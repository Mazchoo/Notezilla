"""Create database updates for note database"""

from src.field_enums import ReservedFields, ColumnTypes
from src.backend.parse_markdown import MarkdownData
from src.backend.database_adapter import NoteDatabase


def prepate_database_row(markdown: MarkdownData, column_types: ColumnTypes) -> dict:
    """
    Transforms markdown into a row of data
    """
    row = {}
    for key, val in markdown.fields.items():
        if ReservedFields.contains(key):
            continue
        target_type = column_types.get(key)
        if target_type is None:
            continue
        row.update(NoteDatabase.cast_value(key, val, target_type))

    row[ReservedFields.PATH] = markdown.path
    row[ReservedFields.FILENAME] = markdown.filename
    row[ReservedFields.TEXT] = markdown.text

    return row
