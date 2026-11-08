"""Compare note paths in the database against files on disk."""

from pathlib import Path

from src.backend.database_interface import INoteDatabase
from src.backend.database_update import prepate_database_row
from src.backend.file_io import (
    delete_note_file,
    get_normalised_path,
    iterate_all_markdowns,
    resolve_note_path,
)
from src.backend.parse_markdown import IMarkdownFile
from src.backend.resolved_folders import ResolvedFolder
from src.field_enums import ColumnTypes


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


def restore_note_from_db(
    db: INoteDatabase, column_types: ColumnTypes, path: str
) -> bool:
    """Write the database copy of a note to disk. True on success."""
    path_key = get_normalised_path(path, ResolvedFolder.NOTES)
    if path_key is None:
        return False
    note = db.get_note_from_path_key(path_key, column_types)
    if note is None:
        return False
    return bool(IMarkdownFile.construct_from_data(path_key, note.text, note.fields))


def upsert_note_from_dir(
    db: INoteDatabase, column_types: ColumnTypes, path: str
) -> bool:
    """Upsert the database entry for a note file on disk. True on success."""
    markdown = IMarkdownFile.construct_from_path(path)
    if markdown is None:
        return False
    db.upsert_batch([prepate_database_row(markdown, column_types)])
    return True


def delete_note_from_db(db: INoteDatabase, path: str) -> bool:
    """Remove a note's database entry without touching disk. True on success."""
    path_key = get_normalised_path(path, ResolvedFolder.NOTES)
    if path_key is None:
        return False
    db.delete_batch([path_key])
    return True


def restore_missing_files_from_db(db: INoteDatabase, column_types: ColumnTypes) -> bool:
    """
    Write note files for database paths that are missing on disk.

    Returns True if every database-only note was written to disk.
    """
    db_only, _ = get_sync_difference(db)
    success = True

    for path in db_only:
        if not restore_note_from_db(db, column_types, path):
            success = False

    return success


def upsert_missing_files_from_dir(db: INoteDatabase, column_types: ColumnTypes) -> bool:
    """
    Upsert database entries for directory paths that are missing from the database.

    Returns True if every directory-only note was upserted.
    """
    _, dir_only = get_sync_difference(db)
    success = True
    rows: list[dict] = []

    for path in dir_only:
        markdown = IMarkdownFile.construct_from_path(path)
        if markdown is None:
            success = False
            continue
        rows.append(prepate_database_row(markdown, column_types))

    if rows:
        db.upsert_batch(rows)

    return success


def delete_db_only_entries(db: INoteDatabase) -> bool:
    """
    Remove database entries for notes that are missing on disk.

    Returns True if every database-only path resolved to a key.
    """
    db_only, _ = get_sync_difference(db)
    keys: list[str] = []
    success = True

    for path in db_only:
        path_key = get_normalised_path(path, ResolvedFolder.NOTES)
        if path_key is None:
            success = False
            continue
        keys.append(path_key)

    db.delete_batch(keys)
    return success


def delete_dir_only_files(db: INoteDatabase) -> bool:
    """
    Delete note files on disk that are missing from the database.

    Returns True if every directory-only file was deleted.
    """
    _, dir_only = get_sync_difference(db)
    success = True

    for path in dir_only:
        if not delete_note_file(path, ResolvedFolder.NOTES):
            success = False

    return success
