"""Manually build database from all available files"""

from datetime import datetime, date

from src.config import BATCH_SIZE
from src.field_enums import ReservedFields, FieldTypes, ColumnTypes
from src.backend.file_io import (
    iterate_all_markdowns,
    save_db_column_types,
    get_default_column_types,
    save_frontmatter,
)
from src.backend.parse_markdown import MarkdownData
from src.backend.database_adapter import NoteDatabase


def get_field_type(value) -> FieldTypes:
    """Identifies the specific YAML/Python type for a single value."""
    if isinstance(value, list):
        # Check if the list contains any nested complexity
        is_nested = any(isinstance(i, (list, dict)) for i in value)
        return FieldTypes.JSON if is_nested else FieldTypes.LIST
    if isinstance(value, dict):
        return FieldTypes.JSON
    if isinstance(value, (datetime, date)):
        return FieldTypes.DATE
    if isinstance(value, bool):
        return FieldTypes.BOOL
    if isinstance(value, int):
        return FieldTypes.INT
    if isinstance(value, float):
        return FieldTypes.FLOAT
    if isinstance(value, str) or value is None:  # none is a corner case
        return FieldTypes.STRING
    return FieldTypes.STRING  # Catch-all


def discover_field_schemas(
    markdown: MarkdownData, column_types: ColumnTypes
) -> ColumnTypes:
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
            if current_type.get_priority() > column_types[key].get_priority():
                column_types[key] = current_type

    return column_types


def create_default_front_matter(column_types: ColumnTypes) -> str:
    """Caches a default markedown to represent each task"""
    default_fields = "\n".join(
        [
            f"{k}: {v.get_default()}"
            for k, v in column_types.items()
            if not ReservedFields.contains(k)
        ]
    )
    return f"---\n{default_fields}\n---\n\n"


def prepate_database_row(markdown: MarkdownData, column_types: ColumnTypes) -> dict:
    """
    Transforms markdown into a row of data
    """
    row = {}
    for key, target_type in column_types.items():
        val = markdown.fields.get(key)
        row.update(NoteDatabase.cast_value(key, val, target_type))

    row[ReservedFields.PATH] = markdown.path
    row[ReservedFields.FILENAME] = markdown.filename
    row[ReservedFields.TEXT] = markdown.text

    return row


def put_all_markdowns_note_folder_into_database():
    """Freshly parse all markdown files and add them to chroma db index"""
    column_types = get_default_column_types()
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
    save_db_column_types(column_types)
    save_frontmatter(create_default_front_matter(column_types))


if __name__ == "__main__":
    put_all_markdowns_note_folder_into_database()
