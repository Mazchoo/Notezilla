"""Compare note paths in the database against files on disk."""

from src.backend.database_adapter import NoteDatabase
from src.backend.database_sync import get_sync_difference


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
    db_only, dir_only = get_sync_difference(NoteDatabase())

    print("Database vs directory path sync")
    print("=" * 40)
    print_path_set("In database but not in directory:", db_only)
    print_path_set("In directory but not in database:", dir_only)

    if not db_only and not dir_only:
        print("All database paths exist on disk and match the directory.")
