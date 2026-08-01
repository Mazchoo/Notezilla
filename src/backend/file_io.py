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
    Get standardized path with forward slashes to make path a unique identifier.
    Trailing . and * will be removed.

    Relative paths are resolved against the note folder. Absolute paths must
    already lie inside the note folder.
    """
    if len(path) > 0 and path[-1] in [".", "*"]:
        path = path[:-1]

    candidate = Path(path)
    if candidate.is_absolute():
        resolved_path = candidate.resolve()
    else:
        resolved_path = (RESOLVED_NOTE_FOLDER / candidate).resolve()

    if not resolved_path.is_relative_to(RESOLVED_NOTE_FOLDER):
        return None
    return "/".join(resolved_path.relative_to(RESOLVED_NOTE_FOLDER).parts)


def absolute_note_path(normed_path: str) -> Path:
    """Absolute Path for a normalised (note-folder-relative) path."""
    if not normed_path:
        return RESOLVED_NOTE_FOLDER
    return RESOLVED_NOTE_FOLDER.joinpath(*Path(normed_path).parts)


def resolve_note_path(path: str) -> Optional[str]:
    """
    Resolve a note path to an absolute filesystem path string.

    Accepts note-folder-relative or absolute paths. Returns None when the path
    is outside the note folder.
    """
    if (normed_path := get_normalised_path(path)) is None:
        return None
    return str(absolute_note_path(normed_path))


def ensure_md_extension(path: str) -> str:
    """
    Ensure the path basename ends with lowercase .md.

    Replaces another basename extension, or appends .md when none is present.
    Dots in parent folders are left alone.
    """
    if path.endswith(".md"):
        return path

    slash = max(path.rfind("/"), path.rfind("\\"))
    basename = path[slash + 1 :]
    prefix = path[: slash + 1]
    dot = basename.rfind(".")
    if dot > 0:
        return f"{prefix}{basename[:dot]}.md"
    return f"{path}.md"


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
    path = absolute_note_path(normed_path)

    try:
        with os.scandir(path) as entries:
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

    parent = absolute_note_path(normed_path).parent
    if parent == RESOLVED_NOTE_FOLDER:
        return True

    try:
        parent.mkdir(parents=True, exist_ok=True)
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
        with open(str(absolute_note_path(normed_path)), "w", encoding="utf-8") as f:
            f.write(contents)
    except OSError:
        return False
    return True


def delete_note_file(path: str) -> bool:
    """
    Delete relative path and return True on success
    NB: modifying .md files in NOTE_FOLDER has side effect of updating database
    """
    if not (normed_path := get_normalised_path(path)):
        return False

    try:
        absolute_note_path(normed_path).unlink()
    except OSError:
        return False
    return True


def delete_notes_folder(path: str) -> bool:
    """
    Recursively delete a folder (and its contents) within the note folder.
    Returns True on success

    False if:
    - path is outside note folder
    - not a directory
    - the note folder itself
    - OS returns an error
    NB: modifying .md files in NOTE_FOLDER has side effect of updating database
    """
    normed_path = get_normalised_path(path)
    if not normed_path:
        return False

    target = absolute_note_path(normed_path)
    if not target.is_dir():
        return False

    try:
        shutil.rmtree(target)
    except OSError:
        return False
    return True


def move_file_or_folder(src: str, dst: str) -> bool:
    """
    Move a file or directory into a destination folder within the note folder.
    Returns True on success.

    False if:
    - src or dst is outside note folder
    - src is the note folder itself
    - src does not exist
    - dst is not an existing directory
    - dst is inside src (when src is a directory)
    - OS returns an error
    NB: modifying .md files in NOTE_FOLDER has side effect of updating database
    """
    src_normed = get_normalised_path(src)
    dst_normed = get_normalised_path(dst)
    if not src_normed or dst_normed is None:
        return False

    src_path = absolute_note_path(src_normed)
    dst_path = absolute_note_path(dst_normed)

    if not src_path.exists() or not dst_path.is_dir():
        return False

    if src_path.is_dir() and dst_path.is_relative_to(src_path):
        return False

    try:
        shutil.move(str(src_path), str(dst_path))
    except OSError:
        return False
    return True


def rename_basename(path: str, new_name: str) -> bool:
    """
    Rename a file or directory within the note folder.
    Returns True on success.

    When renaming a file, if new_name has no extension the source file
    extension is preserved (e.g. note.md + "renamed" → renamed.md).
    Directory renames leave new_name unchanged.

    False if:
    - path or resulting destination is outside note folder
    - path is the note folder itself
    - path does not exist
    - destination already exists
    - OS returns an error
    NB: modifying .md files in NOTE_FOLDER has side effect of updating database
    """
    normed_path = get_normalised_path(path)
    if not normed_path:
        return False

    src_path = absolute_note_path(normed_path)
    if not src_path.exists():
        return False

    if src_path.is_file() and src_path.suffix and not Path(new_name).suffix:
        new_name = f"{new_name}{src_path.suffix}"

    dst_normed = get_normalised_path(str(src_path.parent / new_name))
    if not dst_normed:
        return False

    dst_path = absolute_note_path(dst_normed)

    if dst_path.exists():
        return False

    try:
        src_path.rename(dst_path)
    except OSError:
        return False
    return True


def extract_yaml_from_file_contents(content: str) -> Tuple[str, dict]:
    """Return yaml dict in data if it can be parsed else empty data and file contents"""
    if not content.startswith("---"):
        # No yaml header, early return
        return content, {}

    nl = "\r\n" if content.startswith("---\r\n") else "\n"
    sep = f"{nl}---{nl}"

    start = len(f"---{nl}")
    close_pos = content.find(sep, start)

    if close_pos == -1:
        return content, {}

    yaml_block = content[start:close_pos]
    text = content[close_pos + len(sep) :]

    try:
        data = yaml.safe_load(yaml_block)
        if not isinstance(data, dict):
            data = {}
    except yaml.YAMLError as e:
        # Yaml data cannot be parsed, return full content
        print(f"Warning: Malformed yaml data {e}")
        return content, {}

    # New text and yaml dictionary
    return text, data


def construct_yaml_header(data: dict) -> str:
    """Construct a yaml frontmatter header from a dictionary"""
    if not data:
        return ""
    yaml_block = yaml.dump(data, default_flow_style=False, sort_keys=True)
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
    }


def get_db_column_types() -> dict:
    """Save database column schema"""
    try:
        with open(f"{DATABASE_FOLDER}/column_types.json", "r", encoding="utf-8") as f:
            data = json.load(f)
    except FileNotFoundError:
        data = get_default_column_types()

    return data
