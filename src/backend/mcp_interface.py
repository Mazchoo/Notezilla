"""Definitions of interactions between MCP and other objects."""

from functools import cache
from typing import Any, Dict, List

from fastmcp.tools.tool import ToolResult

from src.backend.database_adapter import NoteDatabase, QueryResult
from src.backend.note import NoteData
from src.backend.file_io import get_db_column_types
from src.field_enums import ColumnTypes


@cache
def init_db() -> NoteDatabase:
    """Lazily initialize the database"""
    return NoteDatabase()


@cache
def init_column_types() -> ColumnTypes:
    """Lazily get the column types"""
    return get_db_column_types()


class McpResponse:
    """Build consistent MCP tool results with a text message and structured payload."""

    @staticmethod
    def success(
        structured_content: Dict[str, Any] | None = None,
    ) -> ToolResult:
        """Return a successful tool result."""
        return ToolResult(
            content="Success",
            structured_content=structured_content or {},
        )

    @staticmethod
    def error(
        message: str,
        structured_content: Dict[str, Any] | None = None,
    ) -> ToolResult:
        """Return a failed tool result."""
        return ToolResult(
            content=f"Error: {message}",
            structured_content=structured_content or {},
        )

    @staticmethod
    def notes(items: List[NoteData]) -> ToolResult:
        """Return note file data as minimal filename/text records."""
        return McpResponse.success({"notes": [item.to_dict() for item in items]})

    @staticmethod
    def _note_metadata(metadata: Dict[str, Any]) -> Dict[str, Any]:
        """Return front-matter metadata only."""
        return {
            key: value for key, value in metadata.items() if not key.startswith("\n")
        }

    @staticmethod
    def note_item(text: str, metadata: Dict[str, Any]) -> NoteData:
        """Build one note record from query document text and raw metadata."""
        return NoteData(
            filename=str(metadata.get("filename", "")),
            text=text,
            fields=McpResponse._note_metadata(metadata),
        )

    @staticmethod
    def notes_from_query(result: QueryResult) -> ToolResult:
        """Convert a database query result into note file records."""
        items = [
            McpResponse.note_item(text, metadata)
            for text, metadata in zip(result.documents, result.metadatas)
        ]
        return McpResponse.notes(items)

    @staticmethod
    def directory(folders: List[str], files: List[str]) -> ToolResult:
        """Return immediate child folders and markdown file names."""
        return McpResponse.success({"folders": folders, "files": files})
