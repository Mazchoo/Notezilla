"""Manually build database from all available files"""

import json
from datetime import datetime, date

from src.reserved_fields import ReservedFields
from src.note_updates.file_io import iterate_all_markdowns
from src.note_updates.parse_markdown import MarkdownData


FIELD_HIERARCHY = ["null", "bool", "int", "float", "date", "list", "str", "json"]


def get_chroma_type(value):
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

        current_type = get_chroma_type(value)

        if key not in column_types:
            column_types[key] = current_type
        else:
            # Upcast if current value requires a more permissive type
            if FIELD_HIERARCHY.index(current_type) > FIELD_HIERARCHY.index(
                column_types[key]
            ):
                column_types[key] = current_type

    return column_types


def prepare_batch_for_chroma(markdown: MarkdownData, column_types: dict) -> dict:
    """
    Transforms markdown into a row of data
    """
    row = {}
    for key, target_type in column_types.items():
        val = markdown.fields.get(key)

        if val is None:
            row[key] = None
            continue

        # Cast logic based on the 'Leader' type of the column
        if target_type == "json":
            row[key] = json.dumps(val, default=str)
        elif target_type == "list":
            # Ensure it's a list (handles cases where one row was a single str)
            row[key] = val if isinstance(val, list) else [val]
        elif target_type == "date":
            row[key] = val.isoformat() if hasattr(val, "isoformat") else str(val)
        elif target_type == "str":
            row[key] = str(val)
        elif target_type == "float":
            row[key] = float(val)
        elif target_type == "int":
            row[key] = int(val)
        elif target_type == "bool":
            row[key] = bool(val)
        else:
            row[key] = val

    row[ReservedFields.PATH.value] = markdown.path
    row[ReservedFields.FILENAME.value] = markdown.filename
    row[ReservedFields.TEXT.value] = markdown.text

    return row


def put_all_markdowns_note_folder_into_database():
    """Freshly parse all markdown files and add them to chroma db index"""
    column_types = {
        ReservedFields.PATH.value: "str",
        ReservedFields.FILENAME.value: "str",
        ReservedFields.TEXT.value: "str",
    }

    for path in iterate_all_markdowns():
        if markdown := MarkdownData.construct_from_path(path):
            column_types = discover_field_schemas(markdown, column_types)

    print(column_types)

    for path in iterate_all_markdowns():
        if markdown := MarkdownData.construct_from_path(path):
            row = prepare_batch_for_chroma(markdown, column_types)
            print(row)


if __name__ == "__main__":
    put_all_markdowns_note_folder_into_database()
