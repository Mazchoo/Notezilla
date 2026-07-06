"""Compare note content in the database against files on disk."""

from dataclasses import dataclass

from tqdm import tqdm

from src.backend.database_adapter import NoteDatabase
from src.backend.database_update import prepate_database_row
from src.backend.file_io import get_db_column_types
from src.backend.parse_markdown import MarkdownData
from src.tasks.check_path_sync import (
    get_database_paths,
    get_directory_paths,
    note_file_path,
)
from src.tasks.restore_missing_files import metadata_to_fields


@dataclass(frozen=True)
class ContentMismatch:
    """A note whose database and on-disk content differ."""

    path: str
    text_differs: bool
    fields_differs: bool


def check_content_sync() -> list[ContentMismatch]:
    """
    Compare database content with on-disk files for paths present in both.

    Note body is stored as the Chroma document; front matter is stored in
    metadata. On disk both are combined in the markdown file.

    Returns a list of mismatches. An empty list means all shared paths match.
    """
    db = NoteDatabase()
    column_types = get_db_column_types()
    shared_paths = get_database_paths(db) & get_directory_paths()

    mismatches: list[ContentMismatch] = []

    for path in tqdm(sorted(shared_paths), desc="Checking content"):
        result = db.query_by_id(path)
        if not result.documents or not result.metadatas:
            mismatches.append(ContentMismatch(path, True, True))
            continue

        db_text = result.documents[0]
        db_fields = metadata_to_fields(result.metadatas[0], column_types)

        disk = MarkdownData.construct_from_path(note_file_path(path))
        if disk is None:
            mismatches.append(ContentMismatch(path, True, True))
            continue

        disk_fields = metadata_to_fields(
            prepate_database_row(disk, column_types), column_types
        )
        text_differs = db_text != disk.text
        fields_differs = db_fields != disk_fields

        if text_differs or fields_differs:
            mismatches.append(
                ContentMismatch(path, text_differs, fields_differs)
            )

    return mismatches


def print_mismatches(mismatches: list[ContentMismatch]) -> None:
    """Print a labelled list of content mismatches, or a none-found message."""
    print("Content mismatches:")
    if not mismatches:
        print("  (none)")
        return

    for mismatch in mismatches:
        parts = []
        if mismatch.text_differs:
            parts.append("body text")
        if mismatch.fields_differs:
            parts.append("front matter")
        detail = ", ".join(parts) if parts else "unknown"
        print(f"  {note_file_path(mismatch.path)} ({detail})")


if __name__ == "__main__":
    mismatches = check_content_sync()

    print("Database vs directory content sync")
    print("=" * 40)
    print_mismatches(mismatches)

    if not mismatches:
        print()
        print("All shared paths have matching content.")
