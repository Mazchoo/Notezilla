"""Upsert note files that exist on disk but are missing from the database."""

from src.backend.database_adapter import NoteDatabase
from src.backend.database_sync import upsert_missing_files_from_dir
from src.backend.file_io import get_db_column_types


if __name__ == "__main__":
    if upsert_missing_files_from_dir(NoteDatabase(), get_db_column_types()):
        print("Upserted all directory-only files to the database.")
    else:
        print("Failed to upsert one or more directory-only files.")
