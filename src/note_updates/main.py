"""Handle changes to note directory and forward them to database updates"""

import time
import threading
from typing import List

from watchdog.observers import Observer
from watchdog.observers.api import BaseObserver
from watchdog.events import FileSystemEventHandler

from src.note_updates.event_handling import (
    FILE_CHANGE_EVENT,
    event_is_valid,
    filter_event_list,
)


class PyFileHandler(FileSystemEventHandler):
    """Watch the note directory for changes with DEBOUNCE (batching)"""

    def __init__(self, debounce_delay_ms=200):
        self.debounce_delay_s = debounce_delay_ms / 1000.0  # Time to wait for "quiet"
        self.queue: List[FILE_CHANGE_EVENT] = []  # Queue to hold recent events
        self._timer = None
        self._lock = threading.Lock()  # Ensure thread safety

    def on_modified(self, event: FILE_CHANGE_EVENT):
        self._handle_event(event)

    def on_created(self, event: FILE_CHANGE_EVENT):
        self._handle_event(event)

    def on_deleted(self, event: FILE_CHANGE_EVENT):
        self._handle_event(event)

    def _handle_event(self, event: FILE_CHANGE_EVENT):
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
            print(f"✅ BATCHED EVENT: {queue}")

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
