"""Handle changes to note directory and forward them to database updates"""

import time
import threading
from typing import List

from watchdog.observers import Observer
from watchdog.observers.api import BaseObserver
from watchdog.events import FileSystemEventHandler

from src.config import BATCH_SIZE
from src.note_updates.event_handling import (
    FileChangeEvent,
    event_is_valid,
    filter_event_list,
)
from src.note_updates.database_adapter import NoteDatabase
from src.note_updates.parse_markdown import MarkdownData
from src.note_updates.database_update import prepate_database_row
from src.note_updates.file_io import get_db_column_types


DB = NoteDatabase()
COLUMN_TYPES = get_db_column_types()


class PyFileHandler(FileSystemEventHandler):
    """Watch the note directory for changes with DEBOUNCE (batching)"""

    def __init__(self, debounce_delay_ms=200):
        self.debounce_delay_s = debounce_delay_ms / 1000.0  # Time to wait for "quiet"
        self.queue: List[FileChangeEvent] = []  # Queue to hold recent events
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
            self.queue.append(event)

            # Cancel the previous timer if it exists
            if self._timer:
                self._timer.cancel()

            # Start a new timer
            self._timer = threading.Timer(self.debounce_delay_s, self._process_batch)
            self._timer.start()

    def _process_batch(self):
        with self._lock:
            if not self.queue:
                return

            queue = filter_event_list(self.queue)

            total_added = 0
            batch = []
            for update in [u for u in queue if u.event_type in ["created", "modified"]]:
                if markdown := MarkdownData.construct_from_path(str(update.src_path)):
                    batch.append(prepate_database_row(markdown, COLUMN_TYPES))
                    if len(batch) >= BATCH_SIZE:
                        DB.upsert_batch(batch)
                        batch = []
                        total_added += len(batch)
            DB.upsert_batch(batch)
            total_added += len(batch)
            print(f"Added {total_added} files to database")

            # Process and then clear
            self.queue.clear()
            self._timer = None


def create_note_directory_watcher(debounce_delay: int) -> BaseObserver:
    """Create a note callback watcher for note directory"""
    handler = PyFileHandler(debounce_delay)
    observer = Observer()
    observer.schedule(handler, path="./notes", recursive=True)
    observer.start()
    return observer


if __name__ == "__main__":
    test_observer = create_note_directory_watcher(200)

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        test_observer.stop()
    finally:
        test_observer.join()
