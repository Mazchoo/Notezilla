"""Handle changes to note directory and forward them to database updates"""

from fastmcp import FastMCP

from src.note_updates.database_adapter import NoteDatabase
from src.note_updates.file_io import get_db_column_types, delete_note_file
from src.note_updates.directory_watcher import PyFileHandler
from src.note_updates.parse_markdown import MarkdownData


DB = NoteDatabase()
COLUMN_TYPES = get_db_column_types()

mcp = FastMCP("Notezilla")


@mcp.tool()
def upsert_note(path: str, contents: str, fields: dict) -> str:
    """Create or update a note file with a YAML frontmatter header.

    Args:
        path: Relative path for the note e.g. "folder/filename.md"
        contents: The markdown body of the note
        fields: Dictionary of metadata fields to convert into a YAML header
    """
    note = MarkdownData.construct_from_data(path, contents, fields)
    if note is None:
        return f"Failed to upsert note at '{path}'. Ensure the path ends with .md."
    return f"Note upserted at '{note.normalised_path}'"


@mcp.tool()
def delete_note(path: str) -> str:
    """Delete a note file.

    Args:
        path: Relative path of the note to delete e.g. "folder/filename.md"
    """
    if delete_note_file(path):
        return f"Note deleted at '{path}'"
    return f"Failed to delete note at '{path}'. Ensure the path is valid."


if __name__ == "__main__":
    test_observer = PyFileHandler.construct_observer(DB, COLUMN_TYPES, 200)

    try:
        mcp.run()
    except KeyboardInterrupt:
        test_observer.stop()
    finally:
        test_observer.join()
