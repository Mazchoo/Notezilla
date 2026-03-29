"""Observer class that handles file updates in storage directory"""

import logging
import threading
from typing import List

from rich.logging import RichHandler
from watchdog.observers import Observer
from watchdog.observers.api import BaseObserver
from watchdog.events import FileSystemEventHandler

logging.basicConfig(
    handlers=[RichHandler()],
    level=logging.INFO,
    format="%(message)s",
    datefmt="[%x %X]",
)
logger = logging.getLogger("notezilla")

from src.config import BATCH_SIZE
from src.note_updates.event_handling import (
    FileChangeEvent,
    event_is_valid,
    filter_event_list,
)
from src.note_updates.database_adapter import NoteDatabase
from src.note_updates.parse_markdown import MarkdownData
from src.note_updates.database_update import prepate_database_row
from src.note_updates.file_io import get_normalised_path


class PyFileHandler(FileSystemEventHandler):
    """Watch the note directory for changes with DEBOUNCE (batching)"""

    def __init__(
        self, database: NoteDatabase, column_types: dict, debounce_delay_ms=200
    ):
        self.debounce_delay_s = debounce_delay_ms / 1000.0  # Time to wait for "quiet"
        self.column_types = column_types
        self.database = database

        self._queue: List[FileChangeEvent] = []  # Queue to hold recent events
        self._timer = None
        self._lock = threading.Lock()  # Ensure thread safety

    def on_modified(self, event: FileChangeEvent):
        self._handle_event(event)

    def on_created(self, event: FileChangeEvent):
        self._handle_event(event)

    def on_deleted(self, event: FileChangeEvent):
        self._handle_event(event)

    def _handle_event(self, event: FileChangeEvent):
        if not event_is_valid(event):
            return

        with self._lock:
            # Add to batch
            self._queue.append(event)

            # Cancel the previous timer if it exists
            if self._timer:
                self._timer.cancel()

            # Start a new timer
            self._timer = threading.Timer(self.debounce_delay_s, self._process_batch)
            self._timer.start()

    def _process_batch(self):
        with self._lock:
            if not self._queue:
                return

            queue = filter_event_list(self._queue)

            total_added = 0
            batch = []
            for update in [u for u in queue if u.event_type in ["created", "modified"]]:
                if markdown := MarkdownData.construct_from_path(str(update.src_path)):
                    batch.append(prepate_database_row(markdown, self.column_types))
                    if len(batch) >= BATCH_SIZE:
                        total_added += self.database.upsert_batch(batch)
                        batch = []

            total_added += self.database.upsert_batch(batch)
            logger.info(f"Added {total_added} files to database")

            total_removed = 0
            batch = []
            for update in [u for u in queue if u.event_type == "deleted"]:
                batch.append(get_normalised_path(str(update.src_path)))
                if len(batch) >= BATCH_SIZE:
                    total_removed -= self.database.delete_batch(batch)
                    batch = []

            total_removed -= self.database.delete_batch(batch)
            logger.info(f"Removed {total_removed} files to database")

            # Process and then clear
            self._queue.clear()
            self._timer = None

    @staticmethod
    def construct_observer(
        database: NoteDatabase, column_types: dict, debounce_delay: int
    ) -> BaseObserver:
        """Create a note callback watcher for note directory"""
        handler = PyFileHandler(database, column_types, debounce_delay)
        observer = Observer()
        observer.schedule(handler, path="./notes", recursive=True)
        observer.start()
        return observer
