"""Handles interaction with files"""

from typing import Optional, Tuple, Iterable
import os
from pathlib import Path
import uuid
import shutil
import json

import yaml

from src.config import NOTE_FOLDER, DATABASE_FOLDER
from src.field_enums import ReservedFields, ColumnTypes

RESOLVED_NOTE_FOLDER = Path(NOTE_FOLDER).resolve()


def read_file_content(path: str) -> Optional[str]:
    """Return file contents or None is file cannot be read"""
    try:
        with open(path, "r", encoding="utf-8") as f:
            return f.read()
    except (FileNotFoundError, IsADirectoryError, OSError):
        return None


def get_normalised_path(path: str) -> Optional[str]:
    """
    Get standardized path with forward slashes to make path a unique identifier
    Trailing . and * will be removed
    """
    if len(path) > 0 and path[-1] in [".", "*"]:
        path = path[:-1]

    resolved_path = Path(path).resolve()
    if not resolved_path.is_relative_to(RESOLVED_NOTE_FOLDER):
        return None
    return "/".join(resolved_path.relative_to(RESOLVED_NOTE_FOLDER).parts)


def get_dirs_and_md_files(
    target_dir: str,
) -> Tuple[list[str], list[str], Optional[str]]:
    """
    List immediate child folders and file names under a note-folder path.
    An error message will return if any error is thrown
    """
    normed_path = get_normalised_path(target_dir)
    if normed_path is None:
        return [], [], f"Path not recognised in note folder {target_dir}"

    folders: list[str] = []
    files: list[str] = []
    path = f"{NOTE_FOLDER}/{normed_path}" if normed_path else NOTE_FOLDER

    try:
        with os.scandir(Path(path)) as entries:
            for entry in entries:
                if entry.is_dir(follow_symlinks=False):
                    folders.append(entry.name)
                elif entry.is_file(follow_symlinks=False) and entry.name.endswith(
                    ".md"
                ):
                    files.append(entry.name)
    except OSError as e:
        return [], [], f"File failed in {path}: {e}"

    return folders, files, None


def ensure_note_parent_dirs(path: str) -> bool:
    """
    Create parent directories for a note path within the note folder.
    Returns True on success, False if path is outside note folder or on error.
    """
    if not (normed_path := get_normalised_path(path)):
        return False

    parent_parts = Path(normed_path).parent
    if parent_parts == Path("."):
        return True

    try:
        (RESOLVED_NOTE_FOLDER / parent_parts).mkdir(parents=True, exist_ok=True)
    except OSError:
        return False
    return True


def write_file_content(path: str, contents: str) -> bool:
    """
    Write file contents to relative path and return True on success
    Will only write to note folder
    NB: writing .md files to NOTE_FOLDER has side effect of updating database
    """
    if not (normed_path := get_normalised_path(path)):
        return False

    try:
        with open(f"{NOTE_FOLDER}/{normed_path}", "w", encoding="utf-8") as f:
            f.write(contents)
    except OSError:
        return False
    return True


def delete_note_file(path: str) -> bool:
    """
    Write file contents to relative path and return True on success
    NB: writing .md files to NOTE_FOLDER has side effect of updating database
    """
    if not (normed_path := get_normalised_path(path)):
        return False

    try:
        Path(f"{NOTE_FOLDER}/{normed_path}").unlink()
    except OSError:
        return False
    return True


def extract_yaml_from_file_contents(content: str) -> Tuple[str, dict]:
    """Return yaml dict in data if it can be parsed else empty data and file contents"""
    split_file = content.split("---")
    if len(split_file) >= 3:
        yaml_block = split_file[1]

        try:
            data = yaml.safe_load(yaml_block)
            text = "".join(split_file[2:])
        except yaml.YAMLError as e:
            print(f"Warning: Malformed yaml data {e}")
            text = content
            data = {}
    else:
        text = content
        data = {}

    return text, data


def construct_yaml_header(data: dict) -> str:
    """Construct a yaml frontmatter header from a dictionary"""
    if not data:
        return ""
    yaml_block = yaml.dump(data, default_flow_style=False, sort_keys=False)
    return f"---\n{yaml_block}---\n"


def iterate_all_markdowns() -> Iterable[str]:
    """Iterate through notes folder and return all markdown paths"""
    for root, _, files in os.walk(NOTE_FOLDER):
        for file in files:
            if file.endswith(".md"):
                yield os.path.join(root, file)


def delete_all_old_index_folders():
    """Chroma db detail, delete all folders in chroma db folder that are uuid's"""
    folder = Path(DATABASE_FOLDER)
    for child in folder.iterdir():
        try:
            uuid.UUID(child.name)
        except ValueError:
            continue
        if child.is_dir():
            shutil.rmtree(child)


def save_db_column_types(column_types: ColumnTypes):
    """Save database column schema"""
    with open(f"{DATABASE_FOLDER}/column_types.json", "w", encoding="utf-8") as f:
        json.dump(column_types, f)


def save_frontmatter(payload: str):
    """Save example front matter"""
    with open(f"{DATABASE_FOLDER}/example_note.md", "w", encoding="utf-8") as f:
        f.write(payload)


def get_default_column_types() -> dict:
    """Get the default"""
    return {
        ReservedFields.FILENAME: "str",
        ReservedFields.TEXT: "str",
    }


def get_db_column_types() -> dict:
    """Save database column schema"""
    try:
        with open(f"{DATABASE_FOLDER}/column_types.json", "r", encoding="utf-8") as f:
            data = json.load(f)
    except FileNotFoundError:
        data = get_default_column_types()

    return data
