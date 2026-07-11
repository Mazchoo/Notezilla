"""Definitions of interactions between MCP and other objects."""

from functools import cache
from typing import Any, List

from fastmcp.tools.tool import ToolResult
from pydantic import BaseModel

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_db_column_types
from src.backend.note import NoteData
from src.backend.output_schema import (
    DirectoryResponse,
    NotesResponse,
    UpsertResponse,
)
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
    def _dump(payload: BaseModel) -> dict[str, Any]:
        return payload.model_dump(by_alias=True)

    @staticmethod
    def success(payload: BaseModel | None = None) -> ToolResult:
        """Return a successful tool result."""
        return ToolResult(
            content="Success",
            structured_content=McpResponse._dump(payload) if payload else {},
        )

    @staticmethod
    def error(message: str, payload: BaseModel | None = None) -> ToolResult:
        """Return a failed tool result."""
        return ToolResult(
            content=f"Error: {message}",
            structured_content=McpResponse._dump(payload) if payload else {},
        )

    @staticmethod
    def notes(items: List[NoteData]) -> ToolResult:
        """Return note file data as MCP structured content."""
        return McpResponse.success(NotesResponse.from_notes(items))

    @staticmethod
    def notes_error(message: str) -> ToolResult:
        """Return a notes-shaped error payload."""
        return McpResponse.error(message, NotesResponse(notes=[]))

    @staticmethod
    def directory(folders: List[str], files: List[str]) -> ToolResult:
        """Return immediate child folders and markdown file names."""
        return McpResponse.success(DirectoryResponse(folders=folders, files=files))

    @staticmethod
    def directory_error(
        message: str, folders: List[str], files: List[str]
    ) -> ToolResult:
        """Return a directory-shaped error payload."""
        return McpResponse.error(
            message, DirectoryResponse(folders=folders, files=files)
        )

    @staticmethod
    def upsert(new_file_created: bool) -> ToolResult:
        """Return upsert result structured content."""
        return McpResponse.success(UpsertResponse(new_file_created=new_file_created))

    @staticmethod
    def upsert_error(message: str) -> ToolResult:
        """Return an upsert-shaped error payload."""
        return McpResponse.error(message, UpsertResponse(new_file_created=False))
