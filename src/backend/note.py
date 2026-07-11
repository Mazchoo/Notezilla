"""Common note data structures."""

from dataclasses import dataclass
from typing import Any, List, Self

from src.backend.file_io import construct_yaml_header, extract_yaml_from_file_contents


@dataclass
class NoteData:
    """Container for note content and front matter.

    ``filename`` is the note's relative path within NOTE_FOLDER
    (e.g. ``"2024/01/note.md"``). Basename and folder parts are derived.
    """

    fields: dict
    text: str
    filename: str

    @classmethod
    def from_payload(cls, content: str, filename: str) -> Self:
        """Build from raw markdown file contents and a relative filename."""
        text, fields = extract_yaml_from_file_contents(content)
        return cls(fields=fields, text=text, filename=filename)

    @property
    def path_parts(self) -> List[str]:
        """Folder segments preceding the basename."""
        return self.filename.split("/")[:-1]

    @property
    def basename(self) -> str:
        """Just the file name, no folders."""
        return self.filename.rsplit("/", 1)[-1]

    def to_file_string(self) -> str:
        """Serialise to markdown file contents with YAML front matter."""
        return construct_yaml_header(self.fields) + self.text

    def to_dict(self) -> dict[str, Any]:
        """Serialise for MCP structured content (uses 'metadata' key)."""
        return {
            "filename": self.filename,
            "text": self.text,
            "metadata": self.fields,
        }

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, NoteData):
            return NotImplemented
        return (
            self.fields == other.fields
            and self.text == other.text
            and self.filename == other.filename
        )

    def __repr__(self) -> str:
        return (
            f"NoteData(fields={self.fields!r}, text={self.text!r}, "
            f"filename={self.filename!r})"
        )

    def __str__(self):
        return f"{self.filename} : {self.fields}"
