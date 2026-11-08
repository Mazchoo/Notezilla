"""Delete note files on disk that have no corresponding database entry."""

from src.backend.database_adapter import NoteDatabase
from src.backend.database_sync import get_sync_difference
from src.backend.file_io import delete_note_file
from src.backend.resolved_folders import ResolvedFolder


def delete_only_in_dir() -> list[str]:
    """
    Remove note files whose paths are missing from the database.

    Returns the paths that were deleted.
    """
    _, dir_only = get_sync_difference(NoteDatabase())
    if not dir_only:
        return []

    deleted: list[str] = []
    for path in sorted(dir_only):
        if delete_note_file(path, ResolvedFolder.NOTES):
            deleted.append(path)

    return deleted


if __name__ == "__main__":
    deleted = delete_only_in_dir()
    if not deleted:
        print("No directory-only files to delete.")
    else:
        print(f"Deleted {len(deleted)} directory-only file(s):")
        for path in deleted:
            print(path)
