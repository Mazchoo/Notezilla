"""Handles interaction with files"""

from typing import Optional, Tuple, Iterable
import os

import yaml

from src.config import NOTE_FOLDER


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
