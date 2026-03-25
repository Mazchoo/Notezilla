"""Manually build database from all available files"""

from datetime import datetime, date

from src.config import BATCH_SIZE
from src.reserved_fields import ReservedFields
from src.note_updates.file_io import iterate_all_markdowns
from src.note_updates.parse_markdown import MarkdownData
from src.note_updates.database_adapter import NoteDatabase


FIELD_HIERARCHY = ["null", "bool", "int", "float", "date", "list", "str", "json"]


def get_field_type(value):
    """Identifies the specific YAML/Python type for a single value."""
    if isinstance(value, list):
        # Check if the list contains any nested complexity
        is_nested = any(isinstance(i, (list, dict)) for i in value)
        return "json" if is_nested else "list"
    if isinstance(value, dict):
        return "json"
    if isinstance(value, (datetime, date)):
        return "date"
    if isinstance(value, bool):
        return "bool"
    if isinstance(value, int):
        return "int"
    if isinstance(value, float):
        return "float"
    if isinstance(value, str) or value is None:  # none is a corner case
        return "str"
    return "str"  # Catch-all


def discover_field_schemas(markdown: MarkdownData, column_types: dict) -> dict:
    """
    Analyzes markdown fields to find 'Highest Common Factor' type
    for every field (column).
    """
    for key, value in markdown.fields.items():
        if ReservedFields.contains(key):
            continue

        current_type = get_field_type(value)

        if key not in column_types:
            column_types[key] = current_type
        else:
            # Upcast if current value requires a more permissive type
            if FIELD_HIERARCHY.index(current_type) > FIELD_HIERARCHY.index(
                column_types[key]
            ):
                column_types[key] = current_type

    return column_types


def prepate_database_row(markdown: MarkdownData, column_types: dict) -> dict:
    """
    Transforms markdown into a row of data
    """
    row = {}
    for key, target_type in column_types.items():
        val = markdown.fields.get(key)
        row.update(NoteDatabase.cast_value(key, val, target_type))

    row[ReservedFields.PATH.value] = markdown.path
    row[ReservedFields.FILENAME.value] = markdown.filename
    row[ReservedFields.TEXT.value] = markdown.text

    return row


def put_all_markdowns_note_folder_into_database():
    """Freshly parse all markdown files and add them to chroma db index"""
    column_types = {
        ReservedFields.FILENAME.value: "str",
        ReservedFields.TEXT.value: "str",
    }
    max_path_depth = 0

    for path in iterate_all_markdowns():
        if markdown := MarkdownData.construct_from_path(path):
            column_types = discover_field_schemas(markdown, column_types)
            max_path_depth = max(max_path_depth, len(markdown.path))

    print(f"Schema: {column_types}")

    db = NoteDatabase(max_path_depth=max_path_depth)
    db.reset_collection()

    batch = []
    for path in iterate_all_markdowns():
        if markdown := MarkdownData.construct_from_path(path):
            batch.append(prepate_database_row(markdown, column_types))
            if len(batch) >= BATCH_SIZE:
                db.upsert_batch(batch)
                batch = []
    db.upsert_batch(batch)

    print(f"Loaded {len(db)} documents into database")


if __name__ == "__main__":
    put_all_markdowns_note_folder_into_database()
