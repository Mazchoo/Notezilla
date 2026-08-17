"""Definitions of interactions between MCP and other objects."""

from functools import cache
from typing import Any, List

from fastmcp.tools.tool import ToolResult
from mcp.types import CallToolResult
from pydantic import BaseModel, PrivateAttr

from src.backend.database_adapter import NoteDatabase
from src.backend.database_interface import INoteDatabase
from src.backend.file_io import get_db_column_types
from src.backend.note import NoteData
from src.backend.output_schema import (
    DirectoryResponse,
    EmptyResponse,
    NotesResponse,
    UpsertResponse,
)
from src.field_enums import ColumnTypes


@cache
def init_db() -> INoteDatabase:
    """Lazily initialize the database"""
    return NoteDatabase()


@cache
def init_column_types() -> ColumnTypes:
    """Lazily get the column types"""
    return get_db_column_types()


class _McpToolResult(ToolResult):
    """ToolResult that preserves MCP ``isError`` on the wire."""

    _is_error: bool = PrivateAttr(default=False)

    def __init__(self, *args: Any, is_error: bool = False, **kwargs: Any):
        super().__init__(*args, **kwargs)
        self._is_error = is_error

    def to_mcp_result(
        self,
    ) -> list | tuple[list, dict[str, Any]] | CallToolResult:
        if self._is_error:
            return CallToolResult(
                content=self.content,
                structuredContent=self.structured_content,
                isError=True,
                _meta=self.meta,
            )
        return super().to_mcp_result()


class McpResponse:
    """Build consistent MCP tool results with a text message and structured payload."""

    @staticmethod
    def _dump(payload: BaseModel) -> dict[str, Any]:
        return payload.model_dump(by_alias=True)

    @staticmethod
    def success(payload: BaseModel | None = None) -> ToolResult:
        """Return a successful tool result."""
        return _McpToolResult(
            content="Success",
            structured_content=McpResponse._dump(payload) if payload else {},
        )

    @staticmethod
    def error(message: str, payload: BaseModel | None = None) -> ToolResult:
        """Return a failed tool result."""
        return _McpToolResult(
            content=f"Error: {message}",
            structured_content=McpResponse._dump(payload) if payload else {},
            is_error=True,
        )

    @staticmethod
    def notes(items: List[NoteData], warnings: List[str]) -> ToolResult:
        """Return note file data as MCP structured content."""
        return McpResponse.success(NotesResponse.from_notes(items, warnings))

    @staticmethod
    def notes_error(message: str, warnings: List[str]) -> ToolResult:
        """Return a notes-shaped error payload."""
        return McpResponse.error(message, NotesResponse(notes=[], warnings=warnings))

    @staticmethod
    def directory(
        folders: List[str], files: List[str], warnings: List[str]
    ) -> ToolResult:
        """Return immediate child folders and markdown file names."""
        return McpResponse.success(
            DirectoryResponse(folders=folders, files=files, warnings=warnings)
        )

    @staticmethod
    def directory_error(
        message: str, folders: List[str], files: List[str], warnings: List[str]
    ) -> ToolResult:
        """Return a directory-shaped error payload."""
        return McpResponse.error(
            message,
            DirectoryResponse(folders=folders, files=files, warnings=warnings),
        )

    @staticmethod
    def upsert(new_file_created: bool, warnings: List[str]) -> ToolResult:
        """Return upsert result structured content."""
        return McpResponse.success(
            UpsertResponse(new_file_created=new_file_created, warnings=warnings)
        )

    @staticmethod
    def upsert_error(message: str, warnings: List[str]) -> ToolResult:
        """Return an upsert-shaped error payload."""
        return McpResponse.error(
            message, UpsertResponse(new_file_created=False, warnings=warnings)
        )

    @staticmethod
    def empty(warnings: List[str]) -> ToolResult:
        """Return a success result with no payload fields."""
        return McpResponse.success(EmptyResponse(warnings=warnings))

    @staticmethod
    def empty_error(message: str, warnings: List[str]) -> ToolResult:
        """Return a failed result with no payload fields."""
        return McpResponse.error(message, EmptyResponse(warnings=warnings))
