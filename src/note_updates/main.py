"""Handle changes to note directory and forward them to database updates"""
import time

from src.note_updates.database_adapter import NoteDatabase
from src.note_updates.file_io import get_db_column_types
from src.note_updates.directory_watcher import PyFileHandler


DB = NoteDatabase()
COLUMN_TYPES = get_db_column_types()

if __name__ == "__main__":
    test_observer = PyFileHandler.construct_observer(DB, COLUMN_TYPES, 200)

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        test_observer.stop()
    finally:
        test_observer.join()
