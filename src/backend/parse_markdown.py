"""Handles converting markdown into structured objected"""

from typing import List, Optional
from pathlib import Path
from dataclasses import dataclass

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
class MarkdownData:
    """Container that holds markdown information for one file"""

    fields: dict
    text: str
    filename: str
    path: List[str]

    def __str__(self):
        return "/".join(self.path) + f"/{self.filename} : {self.fields}"

    @property
    def normalised_path(self) -> str:
        """Express normalised path"""
        return "/".join(self.path + [self.filename])

    @staticmethod
    def construct_from_path(path: str) -> Optional["MarkdownData"]:
        """
        Converters file contents to markdown object
        If header cannot be parsed returns MarkdownData as with file contents in full
        """
        path_obj = Path(path)

        if not path_obj.is_relative_to(NOTE_FOLDER_PATH):
            return None

        relative_parts = path_obj.relative_to(NOTE_FOLDER_PATH).parts
        if not relative_parts:
            return None

        filename = relative_parts[-1]
        path_parts = relative_parts[:-1]

        if not (content := read_file_content(path)):
            return None

        text, fields = extract_yaml_from_file_contents(content)

        return MarkdownData(
            fields=fields, text=text, filename=filename, path=list(path_parts)
        )

    @staticmethod
    def construct_from_data(
        path: str, contents: str, fields: dict
    ) -> Optional[tuple["MarkdownData", bool]]:
        """
        Construct note from data and return it if it was successfully created.

        Returns (MarkdownData, new_file_created) where new_file_created is True
        when the path did not exist before writing.

        Side Effect: will write content to file path, i.e. update or add new
        """

        path_obj = Path(path)
        if path_obj.suffix != ".md":
            return None

        new_file_created = not path_obj.exists()
        payload = construct_yaml_header(fields) + contents
        if not ensure_note_parent_dirs(path):
            return None
        if not write_file_content(path, payload):
            return None

        filename = path_obj.name
        path_parts = path_obj.parts[:-1]

        return (
            MarkdownData(
                fields=fields, text=contents, filename=filename, path=list(path_parts)
            ),
            new_file_created,
        )


if __name__ == "__main__":
    print(MarkdownData.construct_from_path("./notes/spam.md"))
