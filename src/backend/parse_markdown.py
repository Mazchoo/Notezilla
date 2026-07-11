"""Handles converting markdown into structured objected"""

from typing import Optional
from dataclasses import dataclass

from src.backend.note import NoteData
from src.backend.file_io import (
    read_file_content,
    ensure_note_parent_dirs,
    write_file_content,
    get_normalised_path,
    ensure_md_extension,
)


@dataclass
class IMarkdownFile(NoteData):
    """
    Interface to create NoteData but gives certain guarantees on file existence.
    """

    @staticmethod
    def construct_from_path(path: str) -> Optional["IMarkdownFile"]:
        """Construct note data from existing file."""
        if not (normed_path := get_normalised_path(path)):
            return None

        if not (content := read_file_content(path)):
            return None

        return IMarkdownFile.from_payload(content, normed_path)

    @staticmethod
    def construct_from_data(
        path: str, body: str, fields: dict
    ) -> Optional[tuple["IMarkdownFile", bool]]:
        """
        Construct note from data and return it if it was successfully created.

        Returns (MarkdownData, new_file_created) where new_file_created is True
        when the path did not exist before writing.

        Side Effect: will write content to file path, i.e. update or add new
        """
        path = ensure_md_extension(path)

        if not (normed_path := get_normalised_path(path)):
            return None

        note = IMarkdownFile(fields=fields, text=body, filename=normed_path)
        if not ensure_note_parent_dirs(str(note.project_path)):
            return None
        new_file_created = not note.project_path.exists()
        if not write_file_content(str(note.project_path), note.to_file_string()):
            return None

        return (note, new_file_created)


if __name__ == "__main__":
    print(IMarkdownFile.construct_from_path("./notes/new_file.md"))
