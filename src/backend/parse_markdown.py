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
from src.backend.resolved_folders import ResolvedFolder
from src.field_enums import ColumnTypes, ReservedFields


def normalise_filter(path_filter: Optional[str]) -> str:
    """Normalise a path prefix filter for note filename matching.

    Converts backslashes to forward slashes, strips a trailing ``*``,
    and removes a leading ``./`` or ``/`` when present. Blank or None
    yields an empty string.
    """
    if not path_filter:
        return ""

    normalised = path_filter.replace("\\", "/")
    if normalised.endswith("*"):
        normalised = normalised[:-1]
    if normalised.startswith("./"):
        normalised = normalised[2:]
    elif normalised.startswith("/"):
        normalised = normalised[1:]
    return normalised


def parse_frontmatter(
    frontmatter: str, column_types: ColumnTypes, warnings: list[str]
) -> Optional[dict[str, Any]]:
    """Parse a YAML front matter string into a Chroma ``where`` filter.

    Unknown columns (absent from ``column_types``) and the reserved text
    field are omitted from the filter and appended to ``warnings``. A
    non-mapping or empty YAML root is also warned about. List fields
    expand to ``field\\titem: True`` metadata keys. Returns ``None`` when
    there are no usable filter conditions.

    warnings appended in place when invalid data is encountered.
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
            warnings.append("Frontmatter is not valid YAML")
            return None
        fields = loaded if isinstance(loaded, dict) else {}

    if not fields:
        warnings.append("Frontmatter has no filter fields")
        return None

    conditions: list[dict[str, Any]] = []
    for key, val in fields.items():
        target_type = None if key == ReservedFields.TEXT else column_types.get(key)
        if target_type is None:
            warnings.append(f"Invalid frontmatter field '{key}'")
            continue
        for meta_key, meta_val in cast_value(key, val, target_type).items():
            if meta_val is None:
                warnings.append(f"Empty frontmatter condition for '{meta_key}'")
                continue
            conditions.append({meta_key: meta_val})

    if not conditions:
        return None
    if len(conditions) == 1:
        return conditions[0]
    return {"$and": conditions}


def clean_path_filter(path_filter: Optional[str], warnings: list[str]) -> list[str]:
    """Split and normalise a comma-separated path prefix filter.

    Each segment is trimmed and passed through ``normalise_filter``.
    Empty segments are omitted and appended to ``warnings``. Returns an
    empty list when there are no usable prefixes. Blank or None input
    yields an empty list with no warning.
    """
    if not path_filter:
        return []

    cleaned_parts = []
    for part in path_filter.split(","):
        normalised = normalise_filter(part.strip())
        if normalised:
            cleaned_parts.append(normalised)
        else:
            warnings.append(f"Empty path filter '{part.strip()}'")
    return cleaned_parts


@dataclass
class IMarkdownFile(NoteData):
    """
    Interface to create NoteData but gives certain guarantees on file existence.
    """

    @staticmethod
    def construct_from_path(
        path: str, folder: ResolvedFolder = ResolvedFolder.NOTES
    ) -> Optional["IMarkdownFile"]:
        """Construct note data from existing file in the given folder."""
        if not (normed_path := get_normalised_path(path, folder)):
            return None

        if not (
            content := read_file_content(str(absolute_note_path(normed_path, folder)))
        ):
            return None

        return IMarkdownFile.from_payload(content, normed_path)

    @staticmethod
    def construct_from_data(
        path: str,
        body: str,
        fields: dict,
        folder: ResolvedFolder = ResolvedFolder.NOTES,
    ) -> Optional[tuple["IMarkdownFile", bool]]:
        """
        Construct note from data and return it if it was successfully created.

        Returns (MarkdownData, new_file_created) where new_file_created is True
        when the path did not exist before writing.

        Side Effect: will write content to file path, i.e. update or add new
        """
        path = ensure_md_extension(path)

        if not (normed_path := get_normalised_path(path, folder)):
            return None

        note = IMarkdownFile(fields=fields, text=body, filename=normed_path)
        target = absolute_note_path(normed_path, folder)
        if not ensure_note_parent_dirs(str(target), folder):
            return None
        new_file_created = not target.exists()
        if not write_file_content(str(target), note.to_file_string(), folder):
            return None

        return (note, new_file_created)


if __name__ == "__main__":
    print(IMarkdownFile.construct_from_path("./notes/new_file.md"))
