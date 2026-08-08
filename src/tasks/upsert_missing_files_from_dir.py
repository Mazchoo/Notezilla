"""Upsert note files that exist on disk but are missing from the database."""

from src.backend.database_adapter import NoteDatabase
from src.backend.database_update import prepate_database_row
from src.backend.file_io import get_db_column_types, get_normalised_path
from src.backend.parse_markdown import IMarkdownFile
from src.tasks.check_path_sync import check_path_sync


def upsert_missing_files_from_dir() -> tuple[list[str], list[str]]:
    """
    Upsert database entries for directory paths that are missing from the database.

    Returns (uploaded, failures) as absolute path strings.
    """
    _, dir_only = check_path_sync()
    if not dir_only:
        return [], []

    db = NoteDatabase()
    column_types = get_db_column_types()
    uploaded: list[str] = []
    failures: list[str] = []
    rows: list[dict] = []

    for path in sorted(dir_only):
        if get_normalised_path(path) is None:
            failures.append(path)
            continue

        markdown = IMarkdownFile.construct_from_path(path)
        if markdown is None:
            failures.append(path)
            continue

        rows.append(prepate_database_row(markdown, column_types))
        uploaded.append(path)

    if rows:
        db.upsert_batch(rows)

    return uploaded, failures


if __name__ == "__main__":
    uploaded_paths, failed_paths = upsert_missing_files_from_dir()

    if not uploaded_paths:
        print("No directory-only files upserted.")
    else:
        print(f"Upserted {len(uploaded_paths)} file(s) from disk to database:")
        for uploaded_path in uploaded_paths:
            print(uploaded_path)

    if failed_paths:
        print(f"Failed to upsert {len(failed_paths)} file(s):")
        for failed_path in failed_paths:
            print(failed_path)
