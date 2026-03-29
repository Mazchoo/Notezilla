"""Handles interaction with files"""

from typing import Optional, Tuple, Iterable
import os
from pathlib import Path
import uuid
import shutil
import json

import yaml

from src.config import NOTE_FOLDER, DATABASE_FOLDER
from src.reserved_fields import ReservedFields


def read_file_content(path: str) -> Optional[str]:
    """Return file contents or None is file cannot be read"""
    try:
        with open(path, "r", encoding="utf-:8") as f:
            return f.read()
    except FileNotFoundError:
        return None


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


def save_db_column_types(column_types: dict):
    """Save database column schema"""
    with open(f"{DATABASE_FOLDER}/column_types.json", "w", encoding="utf-8") as f:
        json.dump(column_types, f)


def get_default_column_types() -> dict:
    """Get the default"""
    return {
        ReservedFields.FILENAME.value: "str",
        ReservedFields.TEXT.value: "str",
    }


def get_db_column_types() -> dict:
    """Save database column schema"""
    try:
        with open(f"{DATABASE_FOLDER}/column_types.json", "r", encoding="utf-8") as f:
            data = json.load(f)
    except FileNotFoundError:
        data = get_default_column_types()

    return data


def get_normalised_path(path: str) -> str:
    """Get standardized path with forward slashes to make path an id"""
    return "/".join(Path(path).relative_to(NOTE_FOLDER).parts)
