"""Delete database entries that have no corresponding file on disk."""

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_normalised_path
from src.backend.resolved_folders import ResolvedFolder
from src.tasks.check_path_sync import check_path_sync


def delete_only_in_db() -> list[str]:
    """
    Remove database entries whose paths are missing on disk.

    Returns the path keys that were deleted.
    """
    db_only, _ = check_path_sync()
    if not db_only:
        return []

    delete_ids: list[str] = []
    for path in sorted(db_only):
        if path_key := get_normalised_path(path, ResolvedFolder.NOTES):
            delete_ids.append(path_key)

    if delete_ids:
        db = NoteDatabase()
        db.delete_batch(delete_ids)

    return delete_ids


if __name__ == "__main__":
    deleted = delete_only_in_db()
    if not deleted:
        print("No database-only entries to delete.")
    else:
        print(f"Deleted {len(deleted)} database-only entry/entries:")
        for path_key in deleted:
            print(path_key)
