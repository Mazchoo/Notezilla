"""Compare note paths in the database against files on disk."""

from pathlib import Path

from src.backend.database_interface import INoteDatabase
from src.backend.file_io import (
    iterate_all_markdowns,
    resolve_note_path,
)
from src.backend.resolved_folders import ResolvedFolder


def get_database_paths(db: INoteDatabase) -> set[str]:
    """Return all normalised note paths stored in the database."""
    paths = set()
    for key in db.get_all_keys():
        if path := resolve_note_path(key, ResolvedFolder.NOTES):
            paths.add(path)
    return paths


def get_directory_paths() -> set[str]:
    """Return all normalised note paths found under the notes folder."""
    paths: set[str] = set()
    for path in iterate_all_markdowns(ResolvedFolder.NOTES):
        paths.add(resolve_note_path(Path(path).resolve(), ResolvedFolder.NOTES))
    return paths


def get_sync_difference(db: INoteDatabase) -> tuple[set[str], set[str]]:
    """
    Compare database and directory paths.

    Returns (db_only, dir_only) where:
    - db_only: in the database but not on disk
    - dir_only: on disk but not in the database
    """
    db_paths = get_database_paths(db)
    dir_paths = get_directory_paths()
    return db_paths - dir_paths, dir_paths - db_paths
