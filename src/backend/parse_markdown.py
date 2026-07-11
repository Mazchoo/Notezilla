"""Handles converting markdown into structured objected"""

from typing import Optional
from pathlib import Path
from dataclasses import dataclass

from src.backend.note import NoteData
from src.backend.file_io import (
    read_file_content,
    construct_yaml_header,
    ensure_note_parent_dirs,
    write_file_content,
    get_normalised_path,
)


@dataclass
class MarkdownFile(NoteData):
    """Container that holds markdown information for one file"""

    @staticmethod
    def construct_from_path(path: str) -> Optional["MarkdownFile"]:
        """
        Converters file contents to markdown object
        If header cannot be parsed returns MarkdownData as with file contents in full
        """
        if not (normed_path := get_normalised_path(path)):
            return None

        if not (content := read_file_content(path)):
            return None

        return MarkdownFile.from_payload(content, normed_path)

    @staticmethod
    def construct_from_data(
        path: str, body: str, fields: dict
    ) -> Optional[tuple["MarkdownFile", bool]]:
        """
        Construct note from data and return it if it was successfully created.

        Returns (MarkdownData, new_file_created) where new_file_created is True
        when the path did not exist before writing.

        Side Effect: will write content to file path, i.e. update or add new
        """
        if not (normed_path := get_normalised_path(path)):
            return None

        if not normed_path.endswith(".md"):
            return None

        path_obj = Path(path)
        new_file_created = not path_obj.exists()
        payload = construct_yaml_header(fields) + body
        if not ensure_note_parent_dirs(path):
            return None
        if not write_file_content(path, payload):
            return None

        return (
            MarkdownFile(fields=fields, text=body, filename=normed_path),
            new_file_created,
        )


if __name__ == "__main__":
    print(MarkdownFile.construct_from_path("./notes/new_file.md"))
