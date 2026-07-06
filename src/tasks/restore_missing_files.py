"""Restore note files that exist in the database but are missing on disk."""

import json
from typing import Any

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_db_column_types
from src.backend.parse_markdown import MarkdownData
from src.field_enums import ReservedFields, FieldTypes
from src.tasks.check_path_sync import check_path_sync, note_file_path


def metadata_to_fields(metadata: dict, column_types: dict) -> dict[str, Any]:
    """Reconstruct front-matter fields from Chroma metadata."""
    fields: dict[str, Any] = {}
    list_items: dict[str, list[str]] = {}

    for key, val in metadata.items():
        if (
            key.startswith("\n")
            or key == ReservedFields.FILENAME
            or key in ReservedFields.excluded_from_metadata()
        ):
            continue
        if "\t" in key:
            field, item = key.split("\t", 1)
            if val is True:
                list_items.setdefault(field, []).append(item)
            continue

        field_type = column_types.get(key)
        if field_type in (FieldTypes.JSON, FieldTypes.JSON.value):
            fields[key] = json.loads(val) if isinstance(val, str) else val
        else:
            fields[key] = val

    for field, items in list_items.items():
        fields[field] = sorted(items)

    return fields


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
        result = db.query_by_id(path)
        if not result.documents:
            continue

        text = result.documents[0]
        fields = metadata_to_fields(result.metadatas[0], column_types)
        if MarkdownData.construct_from_data(note_file_path(path), text, fields):
            saved += 1

    return saved


if __name__ == "__main__":
    count = restore_missing_files()
    print(f"Saved {count} file(s).")
