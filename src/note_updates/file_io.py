"""Handles interaction with files"""

from typing import Optional, Tuple

import yaml


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
