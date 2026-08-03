"""Handles converting markdown into structured objected"""

from typing import Any, Optional
from dataclasses import dataclass

import yaml

from src.backend.note import NoteData
from src.backend.chroma_parsing import cast_value
from src.backend.file_io import (
    absolute_note_path,
    extract_yaml_from_file_contents,
    read_file_content,
    ensure_note_parent_dirs,
    write_file_content,
    get_normalised_path,
    ensure_md_extension,
)
from src.field_enums import ColumnTypes, ReservedFields


def parse_frontmatter(
    frontmatter: str, column_types: ColumnTypes
) -> Optional[dict[str, Any]]:
    """Parse a YAML front matter string into a Chroma ``where`` filter.

    Unknown columns (absent from ``column_types``) are ignored. List fields
    expand to ``field\\titem: True`` metadata keys (see
    ``NoteDatabase.query_field_contains``). Returns ``None`` when there are
    no usable filter conditions.
    """
    text = frontmatter.strip()
    if not text:
        return None

    if text.startswith("---"):
        _, fields = extract_yaml_from_file_contents(
            text if text.endswith("\n") else f"{text}\n"
        )
    else:
        try:
            loaded = yaml.safe_load(text)
        except yaml.YAMLError:
            return None
        fields = loaded if isinstance(loaded, dict) else {}

    if not fields:
        return None

    conditions: list[dict[str, Any]] = []
    for key, val in fields.items():
        if key == ReservedFields.TEXT:
            continue
        target_type = column_types.get(key)
        if target_type is None:
            continue
        for meta_key, meta_val in cast_value(key, val, target_type).items():
            if meta_val is None:
                continue
            conditions.append({meta_key: meta_val})

    if not conditions:
        return None
    if len(conditions) == 1:
        return conditions[0]
    return {"$and": conditions}


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

        if not (content := read_file_content(str(absolute_note_path(normed_path)))):
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
