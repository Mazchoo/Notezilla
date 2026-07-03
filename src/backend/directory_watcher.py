"""Observer class that handles file updates in storage directory"""

import threading
from typing import List

from watchdog.observers import Observer
from watchdog.observers.api import BaseObserver
from watchdog.events import FileSystemEventHandler

from src.backend.event_handling import (
    FileChangeEvent,
    event_is_valid,
    filter_event_list,
)
from src.backend.database_adapter import NoteDatabase
from src.backend.parse_markdown import MarkdownData
from src.backend.database_update import prepate_database_row
from src.backend.file_io import get_normalised_path
from src.backend.logger import LOGGER
from src.field_enums import ColumnTypes


class PyFileHandler(FileSystemEventHandler):
    """Watch the note directory for changes with DEBOUNCE (batching)"""

    def __init__(
        self, database: NoteDatabase, column_types: ColumnTypes, debounce_delay_ms=200
    ):
        self.debounce_delay_s = debounce_delay_ms / 1000.0  # Time to wait for "quiet"
        self.column_types = column_types
        self.database = database

        self._queue: List[FileChangeEvent] = []  # Queue to hold recent events
        self._timer = None
        self._event_lock = threading.Lock()  # Ensure thread safety
        self._database_lock = threading.Lock()  # Ensure safe execution on database

    def on_modified(self, event: FileChangeEvent):
        self._handle_event(event)

    def on_created(self, event: FileChangeEvent):
        self._handle_event(event)

    def on_deleted(self, event: FileChangeEvent):
        self._handle_event(event)

    def _handle_event(self, event: FileChangeEvent):
        if not event_is_valid(event):
            return

        with self._event_lock:
            # Add to batch
            self._queue.append(event)

            # Cancel the previous timer if it exists
            if self._timer:
                self._timer.cancel()

            # Start a new timer
            self._timer = threading.Timer(self.debounce_delay_s, self._process_batch)
            self._timer.start()

    def _process_batch(self):
        with self._event_lock:
            if not self._queue:
                return

            queue = filter_event_list(self._queue)

            self._queue.clear()
            self._timer = None

        upsert_batch = []
        total_upserted = 0
        for update in [u for u in queue if u.event_type in ["created", "modified"]]:
            total_upserted += 1
            if markdown := MarkdownData.construct_from_path(str(update.src_path)):
                upsert_batch.append(prepate_database_row(markdown, self.column_types))

        delete_batch = []
        total_removed = 0
        for update in [u for u in queue if u.event_type == "deleted"]:
            total_removed += 1
            if not (normed_path := get_normalised_path(str(update.src_path))):
                continue
            delete_batch.append(normed_path)

        with self._database_lock:
            self.database.upsert_batch(upsert_batch)
            self.database.delete_batch(delete_batch)

        LOGGER.info("Upserted %s files to database", total_upserted)
        LOGGER.info("Command to delete %s files to database", total_removed)


    @staticmethod
    def construct_observer(
        database: NoteDatabase, column_types: ColumnTypes, debounce_delay: int
    ) -> BaseObserver:
        """Create a note callback watcher for note directory"""
        handler = PyFileHandler(database, column_types, debounce_delay)
        observer = Observer()
        observer.schedule(handler, path="./notes", recursive=True)
        observer.start()
        return observer
