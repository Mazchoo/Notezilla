"""Handle changes to note directory and forward them to database updates"""
from watchdog.observers import Observer
from watchdog.observers.api import BaseObserver
from watchdog.events import FileSystemEventHandler


class PyFileHandler(FileSystemEventHandler):
    """Watch the note directory for changes """
    def on_modified(self, event):
        if event.is_directory:
            return
        # Only watch .py files (case-insensitive)
        if str(event.src_path).lower().endswith(".md"):
            print(f"File changed: {event.src_path}")

    def on_created(self, event):
        if event.is_directory:
            return
        if str(event.src_path).lower().endswith(".md"):
            print(f"New file created: {event.src_path}")

    def on_deleted(self, event):
        if event.is_directory:
            return
        # We are only interested in .md files (case-insensitive)
        if str(event.src_path).lower().endswith(".md"):
            print(f"File deleted: {event.src_path}")


def create_note_directory_watcher() -> BaseObserver:
    """Create a note callback watcher for note directory"""
    handler = PyFileHandler()
    observer = Observer()
    observer.schedule(handler, path='./notes', recursive=True)
    observer.start()
    return observer


# Configure the watcher
if __name__ == "__main__":
    test_observer = create_note_directory_watcher()

    try:
        while True:
            pass  # Keep the script running
    except KeyboardInterrupt:
        test_observer.stop()
    finally:
        test_observer.join()
