"""Restore note files that exist in the database but are missing on disk."""

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_db_column_types, get_normalised_path
from src.backend.parse_markdown import IMarkdownFile
from src.backend.resolved_folders import ResolvedFolder
from src.tasks.check_path_sync import check_path_sync


def restore_missing_files_from_db() -> list[str]:
    """
    Write note files for database paths that are missing on disk.

    Returns the absolute path strings that were saved.
    """
    db_only, _ = check_path_sync()
    if not db_only:
        return []

    db = NoteDatabase()
    column_types = get_db_column_types()
    saved: list[str] = []

    for path in sorted(db_only):
        path_key = get_normalised_path(path, ResolvedFolder.NOTES)
        if path_key is None:
            continue
        note = db.get_note_from_path_key(path_key, column_types)
        if note is None:
            continue

        if IMarkdownFile.construct_from_data(path_key, note.text, note.fields):
            saved.append(path)

    return saved


if __name__ == "__main__":
    saved_paths = restore_missing_files_from_db()
    if not saved_paths:
        print("No database-only files restored.")
    else:
        print(f"Saved {len(saved_paths)} file(s) from database to disk:")
        for saved_path in saved_paths:
            print(saved_path)
