"""Definitions of interactions between MCP and other objects."""

from functools import cache
from typing import Any, List

from fastmcp.tools.tool import ToolResult
from pydantic import BaseModel, ConfigDict, Field

from src.backend.database_adapter import NoteDatabase
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


class NoteFile(BaseModel):
    """Single note payload in MCP structured content."""

    filename: str = Field(description="Relative path of the note within the notes folder")
    text: str = Field(description="Markdown body of the note without YAML front matter")
    metadata: dict[str, Any] = Field(
        description="YAML front matter fields for the note"
    )

    @classmethod
    def from_note(cls, item: NoteData) -> "NoteFile":
        """Build from a NoteData instance."""
        return cls(filename=item.filename, text=item.text, metadata=item.fields)


class NotesResponse(BaseModel):
    """Structured content for tools that return notes."""

    notes: list[NoteFile] = Field(description="Matching notes")

    @classmethod
    def from_notes(cls, items: List[NoteData]) -> "NotesResponse":
        """Build from NoteData instances."""
        return cls(notes=[NoteFile.from_note(item) for item in items])


class DirectoryResponse(BaseModel):
    """Structured content for directory listings."""

    folders: list[str] = Field(description="Immediate child folder names")
    files: list[str] = Field(description="Immediate child markdown file names")


class UpsertResponse(BaseModel):
    """Structured content for upsert_note."""

    model_config = ConfigDict(populate_by_name=True)

    new_file_created: bool = Field(
        alias="newFileCreated",
        description="True when the note file was newly created",
    )


class EmptyResponse(BaseModel):
    """Structured content with no fields (e.g. delete_note success)."""


def output_schema(model: type[BaseModel]) -> dict[str, Any]:
    """JSON Schema for a response model, used as FastMCP tool output_schema."""
    return model.model_json_schema()


NOTES_OUTPUT_SCHEMA = output_schema(NotesResponse)
DIRECTORY_OUTPUT_SCHEMA = output_schema(DirectoryResponse)
UPSERT_OUTPUT_SCHEMA = output_schema(UpsertResponse)
EMPTY_OUTPUT_SCHEMA = output_schema(EmptyResponse)


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
        return McpResponse.error(
            message, UpsertResponse(new_file_created=False)
        )
