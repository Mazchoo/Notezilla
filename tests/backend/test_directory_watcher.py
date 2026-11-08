"""Tests that the directory watcher keeps the database in step with file moves and deletes."""

from unittest.mock import MagicMock, patch

import pytest
from watchdog.events import DirMovedEvent, FileDeletedEvent, FileMovedEvent

from src.backend.database_adapter import NoteDatabase
from src.backend.directory_watcher import PyFileHandler
from src.field_enums import ReservedFields
from tests.backend.helpers import MOCK_NOTES_FOLDER, temporary_notes


@pytest.fixture()
def database():
    return MagicMock()


@pytest.fixture()
def handler(database, mock_notes_folder):
    with patch("src.backend.directory_watcher.publish"):
        yield PyFileHandler(database, {}, debounce_delay_ms=10_000)


def flush(handler: PyFileHandler):
    """Process queued events now instead of waiting for the debounce timer."""
    if handler._timer:
        handler._timer.cancel()
    handler._process_batch()


def note_path(relative: str) -> str:
    return str(MOCK_NOTES_FOLDER / relative)


def upserted_filenames(database: MagicMock) -> list:
    rows = database.upsert_batch.call_args.args[0]
    return [row[ReservedFields.FILENAME] for row in rows]


def test_deleting_folder_updates_database(handler, database):
    handler.on_deleted(FileDeletedEvent(note_path("old_folder")))
    flush(handler)

    database.delete_folders.assert_called_once_with(["old_folder"])


def test_deleting_note_does_not_delete_folder(handler, database):
    handler.on_deleted(FileDeletedEvent(note_path("folder/note.md")))
    flush(handler)

    database.delete_batch.assert_called_once_with(["folder/note.md"])
    database.delete_folders.assert_called_once_with([])


def test_moving_file_updates_database(handler, database):
    with temporary_notes({"tmp_dst/note.md": "hello"}):
        handler.on_moved(
            FileMovedEvent(note_path("tmp_src/note.md"), note_path("tmp_dst/note.md"))
        )
        flush(handler)

    database.delete_batch.assert_called_once_with(["tmp_src/note.md"])
    assert upserted_filenames(database) == ["tmp_dst/note.md"]


def test_renaming_file_updates_database(handler, database):
    with temporary_notes({"tmp_folder/new.md": "hello"}):
        handler.on_moved(
            FileMovedEvent(note_path("tmp_folder/old.md"), note_path("tmp_folder/new.md"))
        )
        flush(handler)

    database.delete_batch.assert_called_once_with(["tmp_folder/old.md"])
    assert upserted_filenames(database) == ["tmp_folder/new.md"]


def test_renaming_file_away_from_markdown_only_deletes(handler, database):
    handler.on_moved(FileMovedEvent(note_path("note.md"), note_path("note.txt")))
    flush(handler)

    database.delete_batch.assert_called_once_with(["note.md"])
    database.upsert_batch.assert_called_once_with([])


def test_moving_folder_deletes_old_folder_from_database(handler, database):
    handler.on_moved(DirMovedEvent(note_path("old_folder"), note_path("new_folder")))
    flush(handler)

    database.delete_folders.assert_called_once_with(["old_folder"])


def test_folder_deletes_run_before_upserts(handler, database):
    with temporary_notes({"tmp_folder/note.md": "hello"}):
        handler.on_deleted(FileDeletedEvent(note_path("tmp_folder")))
        handler.on_moved(
            FileMovedEvent(note_path("elsewhere.md"), note_path("tmp_folder/note.md"))
        )
        flush(handler)

    call_order = [name for name, _args, _kwargs in database.method_calls]
    assert call_order.index("delete_folders") < call_order.index("upsert_batch")


def test_delete_folders_removes_only_keys_under_folder():
    database = NoteDatabase.__new__(NoteDatabase)
    database._collection = MagicMock()
    keys = ["old/a.md", "old/sub/b.md", "older/c.md", "other/d.md"]

    with patch.object(NoteDatabase, "get_all_keys", return_value=iter(keys)):
        database.delete_folders(["old"])

    database._collection.delete.assert_called_once_with(
        ids=["old/a.md", "old/sub/b.md"]
    )


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
