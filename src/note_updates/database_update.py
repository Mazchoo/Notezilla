"""Create database updates for note database"""

from src.reserved_fields import ReservedFields
from src.note_updates.parse_markdown import MarkdownData
from src.note_updates.database_adapter import NoteDatabase


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
