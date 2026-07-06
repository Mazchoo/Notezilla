"""Compare note content in the database against files on disk."""

from dataclasses import dataclass
from datetime import date, datetime
from typing import Any

from tqdm import tqdm

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_db_column_types
from src.backend.parse_markdown import MarkdownData
from src.field_enums import ReservedFields
from src.tasks.check_path_sync import (
    get_database_paths,
    get_directory_paths,
    note_file_path,
)


@dataclass(frozen=True)
class ContentMismatch:
    """A note whose database and on-disk content differ."""

    path: str
    text_differs: bool
    metadata_differs: bool


def _fields_for_compare(fields: dict[str, Any]) -> dict[str, Any]:
    """Normalise parsed front matter for comparison."""
    aligned: dict[str, Any] = {}
    for key, val in fields.items():
        if isinstance(val, (date, datetime)):
            aligned[key] = val.isoformat()
        else:
            aligned[key] = val
    return aligned


def _expected_from_disk(path: str) -> tuple[str, dict[str, Any], str] | None:
    """Read body text, front matter, and filename from an on-disk note."""
    disk = MarkdownData.construct_from_path(note_file_path(path))
    if disk is None:
        return None

    frontmatter = {
        k: v
        for k, v in _fields_for_compare(disk.fields).items()
        if v is not None and not ReservedFields.contains(k)
    }
    return disk.text, frontmatter, disk.filename


def check_content_sync() -> list[ContentMismatch]:
    """
    Compare database content with on-disk files for paths present in both.

    Returns a list of mismatches. An empty list means all shared paths match.
    """
    db = NoteDatabase()
    column_types = get_db_column_types()
    shared_paths = get_database_paths(db) & get_directory_paths()

    mismatches: list[ContentMismatch] = []

    for path in tqdm(sorted(shared_paths), desc="Checking content"):
        note_from_db = db.get_frontmatter_from_path_key(path, column_types)
        expected = _expected_from_disk(path)

        if note_from_db is None or expected is None:
            mismatches.append(ContentMismatch(path, True, True))
            continue

        expected_text, expected_frontmatter, expected_filename = expected

        text_differs = note_from_db["text"] != expected_text
        metadata_differs = (
            note_from_db["metadata"] != expected_frontmatter
            or note_from_db["filename"] != expected_filename
        )

        if text_differs or metadata_differs:
            mismatches.append(ContentMismatch(path, text_differs, metadata_differs))

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
        if mismatch.metadata_differs:
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
