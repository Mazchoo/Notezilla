"""Handles converting markdown into structured objected"""

from typing import Optional
from pathlib import Path
from dataclasses import dataclass

from src.backend.note import NoteData
from src.backend.file_io import (
    read_file_content,
    extract_yaml_from_file_contents,
    construct_yaml_header,
    ensure_note_parent_dirs,
    write_file_content,
)
from src.config import NOTE_FOLDER

NOTE_FOLDER_PATH = Path(NOTE_FOLDER)


@dataclass
class MarkdownFile(NoteData):
    """Container that holds markdown information for one file"""

    @staticmethod
    def construct_from_path(path: str) -> Optional["MarkdownFile"]:
        """
        Converters file contents to markdown object
        If header cannot be parsed returns MarkdownData as with file contents in full
        """
        # ToDo - use normalize here, don't reinvent wheel
        path_obj = Path(path)

        if not path_obj.is_relative_to(NOTE_FOLDER_PATH):
            return None

        relative_parts = path_obj.relative_to(NOTE_FOLDER_PATH).parts
        if not relative_parts:
            return None

        if not (content := read_file_content(path)):
            return None

        text, fields = extract_yaml_from_file_contents(content)

        return MarkdownFile(fields=fields, text=text, filename="/".join(relative_parts))

    @staticmethod
    def construct_from_data(
        path: str, contents: str, fields: dict
    ) -> Optional[tuple["MarkdownFile", bool]]:
        """
        Construct note from data and return it if it was successfully created.

        Returns (MarkdownData, new_file_created) where new_file_created is True
        when the path did not exist before writing.

        Side Effect: will write content to file path, i.e. update or add new
        """

        # ToDo - use normalize file path here
        path_obj = Path(path)
        if path_obj.suffix != ".md":
            return None

        new_file_created = not path_obj.exists()
        payload = construct_yaml_header(fields) + contents
        if not ensure_note_parent_dirs(path):
            return None
        if not write_file_content(path, payload):
            return None

        try:
            relative_parts = path_obj.relative_to(NOTE_FOLDER_PATH).parts
        except ValueError:
            relative_parts = path_obj.parts
        return (
            MarkdownFile(
                fields=fields, text=contents, filename="/".join(relative_parts)
            ),
            new_file_created,
        )


if __name__ == "__main__":
    print(MarkdownFile.construct_from_path("./notes/new_file.md"))
