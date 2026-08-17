"""Compare note paths in the database against files on disk."""

from pathlib import Path

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import (
    iterate_all_markdowns,
    resolve_note_path,
)
from src.backend.resolved_folders import ResolvedFolder


def get_database_paths(db: NoteDatabase) -> set[str]:
    """Return all normalised note paths stored in the database."""
    paths = set()
    for key in db._collection.get(include=[]).get("ids", []):
        if path := resolve_note_path(key, ResolvedFolder.NOTES):
            paths.add(path)
    return paths


def get_directory_paths() -> set[str]:
    """Return all normalised note paths found under the notes folder."""
    paths: set[str] = set()
    for path in iterate_all_markdowns(ResolvedFolder.NOTES):
        paths.add(resolve_note_path(Path(path).resolve(), ResolvedFolder.NOTES))
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


def print_path_set(title: str, paths: set[str]) -> None:
    """Print a labelled list of file paths, or a none-found message."""
    print(title)
    if not paths:
        print("  (none)")
    else:
        for path in sorted(paths):
            print(path)
    print()


if __name__ == "__main__":
    db_only, dir_only = check_path_sync()

    print("Database vs directory path sync")
    print("=" * 40)
    print_path_set("In database but not in directory:", db_only)
    print_path_set("In directory but not in database:", dir_only)

    if not db_only and not dir_only:
        print("All database paths exist on disk and match the directory.")
