"""Shared pytest configuration and fixtures for backend MCP tool tests."""

from unittest.mock import MagicMock, patch

import pytest

from src.backend.database_adapter import NoteDatabase
from src.backend.mcp_interface import init_db
from tests.backend.helpers import MOCK_NOTES_FOLDER


@pytest.fixture(autouse=True)
def clear_init_db_cache():
    """Clear the init_db cache before and after every test."""
    init_db.cache_clear()
    yield
    init_db.cache_clear()


@pytest.fixture()
def mock_notes_folder():
    """Point NOTE_FOLDER at tests/mock_notes for filesystem-backed tests."""
    with (
        patch("src.backend.main.NOTE_FOLDER", str(MOCK_NOTES_FOLDER)),
        patch("src.backend.file_io.NOTE_FOLDER", str(MOCK_NOTES_FOLDER)),
        patch("src.backend.file_io.RESOLVED_NOTE_FOLDER", MOCK_NOTES_FOLDER),
    ):
        yield MOCK_NOTES_FOLDER


@pytest.fixture()
def temp_db(tmp_path):
    """Isolated Chroma database for one test."""
    return NoteDatabase(path=str(tmp_path / "chroma_db"))


@pytest.fixture()
def mock_db():
    """
    Patch NoteDatabase in mcp_interface so init_db() returns a MagicMock instance.
    Query methods are mocked individually; the rest of the adapter is untouched.
    """
    init_db.cache_clear()
    with patch("src.backend.mcp_interface.NoteDatabase") as mock_cls:
        db = mock_cls.return_value
        db.query_by_id = MagicMock()
        db.query_by_field = MagicMock()
        db.query_field_contains = MagicMock()
        db.query_by_text = MagicMock()
        yield db
    init_db.cache_clear()
