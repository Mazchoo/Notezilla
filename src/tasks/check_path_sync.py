"""Compare note paths in the database against files on disk."""

from src.config import NOTE_FOLDER
from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_normalised_path, iterate_all_markdowns


def get_database_paths(db: NoteDatabase) -> set[str]:
    """Return all normalised note paths stored in the database."""
    results = db._collection.get(include=[])
    return set(results.get("ids") or [])


def get_directory_paths() -> set[str]:
    """Return all normalised note paths found under the notes folder."""
    paths: set[str] = set()
    for path in iterate_all_markdowns():
        if normed := get_normalised_path(path):
            paths.add(normed)
    return paths


def check_path_sync() -> tuple[set[str], set[str]]:
    """
    Compare database and directory paths.

    Returns (db_only, dir_only) where:
    - db_only: in the database but not on disk
    - dir_only: on disk but not in the database
    """
    db = NoteDatabase()
    db_paths = get_database_paths(db)
    dir_paths = get_directory_paths()
    return db_paths - dir_paths, dir_paths - db_paths


def note_file_path(path: str) -> str:
    """Return the note path as it appears on disk."""
    return f"{NOTE_FOLDER}/{path}"


def print_path_set(title: str, paths: set[str]) -> None:
    """Print a labelled list of file paths, or a none-found message."""
    print(title)
    if not paths:
        print("  (none)")
    else:
        for path in sorted(paths):
            print(f"  {note_file_path(path)}")
    print()


if __name__ == "__main__":
    db_only, dir_only = check_path_sync()

    print("Database vs directory path sync")
    print("=" * 40)
    print_path_set("In database but not in directory:", db_only)
    print_path_set("In directory but not in database:", dir_only)

    if not db_only and not dir_only:
        print("All database paths exist on disk and match the directory.")
