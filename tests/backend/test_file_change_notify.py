"""Tests for filesystem change pings."""

from unittest.mock import MagicMock, patch

import pytest
from watchdog.events import FileModifiedEvent

from src.backend.directory_watcher import PyFileHandler
from src.backend.file_change_notify import (
    clear_subscribers,
    publish,
    subscribe,
)
from src.backend.resolved_folders import ResolvedFolder


@pytest.fixture(autouse=True)
def _isolate_subscribers():
    clear_subscribers()
    yield
    clear_subscribers()


def test_publish_delivers_a_ping():
    received = []
    unsubscribe = subscribe(received.append)

    publish()
    unsubscribe()
    publish()

    assert received == ["{}"]


def test_process_batch_publishes_after_database_update(tmp_path):
    notes = tmp_path / "notes"
    notes.mkdir()
    md = notes / "hello.md"
    md.write_text("hello", encoding="utf-8")
    received = []
    subscribe(received.append)
    database = MagicMock()

    with patch.object(ResolvedFolder.NOTES, "_value_", notes.resolve()):
        handler = PyFileHandler(database, {}, debounce_delay_ms=10)
        handler._queue = [FileModifiedEvent(str(md))]
        handler._process_batch()

    assert received == ["{}"]
    database.upsert_batch.assert_called_once()
    database.delete_batch.assert_called_once_with([])


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
