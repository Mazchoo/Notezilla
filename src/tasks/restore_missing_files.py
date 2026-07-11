"""Restore note files that exist in the database but are missing on disk."""

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_db_column_types
from src.backend.parse_markdown import IMarkdownFile
from src.tasks.check_path_sync import check_path_sync, note_file_path


def restore_missing_files() -> int:
    """
    Write note files for database paths that are missing on disk.

    Returns the number of files saved.
    """
    db_only, _ = check_path_sync()
    if not db_only:
        return 0

    db = NoteDatabase()
    column_types = get_db_column_types()
    saved = 0

    for path in sorted(db_only):
        note = db.get_frontmatter_from_path_key(path, column_types)
        if note is None:
            continue

        if IMarkdownFile.construct_from_data(
            note_file_path(path), note.text, note.fields
        ):
            saved += 1

    return saved


if __name__ == "__main__":
    count = restore_missing_files()
    print(f"Saved {count} file(s).")
