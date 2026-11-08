"""Restore note files that exist in the database but are missing on disk."""

from src.backend.database_adapter import NoteDatabase
from src.backend.database_sync import restore_missing_files_from_db
from src.backend.file_io import get_db_column_types


if __name__ == "__main__":
    if restore_missing_files_from_db(NoteDatabase(), get_db_column_types()):
        print("Restored all database-only files to disk.")
    else:
        print("Failed to restore one or more database-only files.")
