"""MCP tool output schemas."""

from typing import Any, List

from pydantic import BaseModel, ConfigDict, Field

from src.backend.note import NoteData


class NoteFile(BaseModel):
    """Single note payload in MCP structured content."""

    filename: str = Field(
        description="Relative path of the note within the notes folder"
    )
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
