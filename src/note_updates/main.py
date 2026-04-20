"""Handle changes to note directory and forward them to database updates"""

from typing import Any, Dict, List, Optional, TypedDict

from fastmcp import FastMCP

from src.note_updates.database_adapter import NoteDatabase
from src.note_updates.file_io import get_db_column_types, delete_note_file
from src.note_updates.directory_watcher import PyFileHandler
from src.note_updates.parse_markdown import MarkdownData


DB = NoteDatabase()
COLUMN_TYPES = get_db_column_types()

mcp = FastMCP("Notezilla")


class NoteQueryResult(TypedDict):
    documents: List[str]
    metadatas: List[Dict[str, Any]]
    error: Optional[str]


@mcp.tool()
def upsert_note(path: str, contents: str, fields: dict) -> str:
    """Create or update a note file with a YAML frontmatter header.

    Args:
        path: Relative path for the note e.g. "folder/filename.md"
        contents: The markdown body of the note
        fields: Dictionary of metadata fields to convert into a YAML header
    """
    if note := MarkdownData.construct_from_data(path, contents, fields):
        return f"Note upserted at '{note.normalised_path}'"
    return f"Error: Failed to upsert note at '{path}'. Ensure the path ends with .md."


@mcp.tool()
def delete_note(path: str) -> str:
    """Delete a note file.

    Args:
        path: Relative path of the note to delete e.g. "folder/filename.md"
    """
    if delete_note_file(path):
        return f"Note deleted at '{path}'"
    return f"Error: Failed to delete note at '{path}'. Ensure the path is valid."


@mcp.tool()
def search_notes_by_field(
    field: str, value: str, n_results: int = 10
) -> NoteQueryResult:
    """Find notes where a metadata field exactly matches a value.

    Args:
        field: Metadata field name to filter on e.g. "filename"
        value: Exact value the field must equal
        n_results: Maximum number of results to return
    """
    try:
        result = DB.query_by_field(field, value, n_results)
        return NoteQueryResult(
            documents=result.documents, metadatas=result.metadatas, error=None
        )
    except ValueError as e:
        return NoteQueryResult(documents=[], metadatas=[], error=f"Type error: {e}")
    except Exception as e:  # pylint: disable=broad-except
        return NoteQueryResult(documents=[], metadatas=[], error=f"DB error: {e}")


@mcp.tool()
def search_notes_by_tag(field: str, value: str, n_results: int = 10) -> NoteQueryResult:
    """Find notes where a list metadata field contains a given value.

    Args:
        field: List metadata field name e.g. "tags"
        value: Value that the list must contain
        n_results: Maximum number of results to return
    """
    try:
        result = DB.query_field_contains(field, value, n_results)
        return NoteQueryResult(
            documents=result.documents, metadatas=result.metadatas, error=None
        )
    except ValueError as e:
        return NoteQueryResult(documents=[], metadatas=[], error=f"Type error: {e}")
    except Exception as e:  # pylint: disable=broad-except
        return NoteQueryResult(documents=[], metadatas=[], error=f"DB error: {e}")


@mcp.tool()
def search_notes_by_path(
    path_parts: List[str], n_results: int = 100
) -> NoteQueryResult:
    """Find notes under a given folder path.

    Args:
        path_parts: Ordered list of path segments e.g. ["2018", "01", "14"]
        n_results: Maximum number of results to return
    """
    try:
        result = DB.query_by_path(path_parts, n_results)
        return NoteQueryResult(
            documents=result.documents, metadatas=result.metadatas, error=None
        )
    except ValueError as e:
        return NoteQueryResult(documents=[], metadatas=[], error=f"Type error: {e}")
    except Exception as e:  # pylint: disable=broad-except
        return NoteQueryResult(documents=[], metadatas=[], error=f"DB error: {e}")


@mcp.tool()
def search_notes_by_text(text: str, n_results: int = 10) -> NoteQueryResult:
    """Semantically search notes by their content.

    Args:
        text: Natural language query to search for
        n_results: Maximum number of results to return
    """
    try:
        result = DB.query_by_text(text, n_results)
        return NoteQueryResult(
            documents=result.documents, metadatas=result.metadatas, error=None
        )
    except ValueError as e:
        return NoteQueryResult(documents=[], metadatas=[], error=f"Type error: {e}")
    except Exception as e:  # pylint: disable=broad-except
        return NoteQueryResult(documents=[], metadatas=[], error=f"DB error: {e}")


if __name__ == "__main__":
    test_observer = PyFileHandler.construct_observer(DB, COLUMN_TYPES, 200)

    try:
        mcp.run()
    except KeyboardInterrupt:
        test_observer.stop()
    finally:
        test_observer.join()
