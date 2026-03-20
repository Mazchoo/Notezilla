"""Handle changes to note directory and forward them to database updates"""

import time
import threading

from watchdog.observers import Observer
from watchdog.observers.api import BaseObserver
from watchdog.events import FileSystemEventHandler


class PyFileHandler(FileSystemEventHandler):
    """Watch the note directory for changes with DEBOUNCE (batching)"""

    def __init__(self, debounce_delay=200):
        self.debounce_delay = debounce_delay  # Time to wait for "quiet" (ms)
        self.queue = []  # Queue to hold recent events
        self._timer = None
        self._lock = threading.Lock()  # Ensure thread safety

    def on_modified(self, event):
        self._handle_event(event, "modified")

    def on_created(self, event):
        self._handle_event(event, "created")

    def on_deleted(self, event):
        self._handle_event(event, "deleted")

    def _handle_event(self, event, event_type):
        if event.is_directory or not event.src_path.lower().endswith(".md"):
            return

        with self._lock:
            # Add to batch
            self.queue.append({"event": event, "type": event_type})

            # Cancel the previous timer if it exists
            if self._timer:
                self._timer.cancel()

            # Start a new timer
            self._timer = threading.Timer(self.debounce_delay, self._process_batch)
            self._timer.start()

    def _process_batch(self):
        with self._lock:
            if not self.queue:
                return

            # ToDo - This needs to remove duplicate events
            print(f"✅ BATCHED EVENT: {self.queue}")
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
    test_observer = create_note_directory_watcher(2)

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        test_observer.stop()
    finally:
        test_observer.join()
