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
from src.backend.parse_markdown import MarkdownFile
from src.backend.database_adapter import NoteDatabase
from src.backend.database_update import prepate_database_row


def get_field_type(value) -> FieldTypes:  # pylint: disable=too-many-return-statements
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
    if isinstance(value, str) or value is None:
        return FieldTypes.STRING
    return FieldTypes.STRING  # Catch-all


def discover_field_schemas(
    markdown: MarkdownFile, column_types: ColumnTypes
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


def put_all_markdowns_note_folder_into_database():
    """Freshly parse all markdown files and add them to chroma db index"""
    column_types = get_default_column_types()

    for path in iterate_all_markdowns():
        if markdown := MarkdownFile.construct_from_path(path):
            column_types = discover_field_schemas(markdown, column_types)

    print(f"Schema: {column_types}")

    db = NoteDatabase()
    db.reset_collection()

    batch = []
    for path in iterate_all_markdowns():
        if markdown := MarkdownFile.construct_from_path(path):
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
