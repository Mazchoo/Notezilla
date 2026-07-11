"""Common note data structures."""

from dataclasses import dataclass
from typing import Any, List

from src.backend.file_io import construct_yaml_header


@dataclass
class NoteData:
    """Container for note content and front matter.

    ``filename`` is the note's relative path within NOTE_FOLDER
    (e.g. ``"2024/01/note.md"``). Basename and folder parts are derived.
    """

    fields: dict
    text: str
    filename: str

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
