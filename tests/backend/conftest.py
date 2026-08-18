"""Shared pytest configuration and fixtures for backend MCP tool tests."""

from unittest.mock import MagicMock, patch

import pytest

from src.backend.database_adapter import NoteDatabase
from src.backend.mcp_interface import init_db
from src.backend.resolved_folders import ResolvedFolder
from tests.backend.helpers import (
    COMMITTED_MOCK_NOTE_FILES,
    COMMITTED_MOCK_TEMPLATE_FILES,
    MOCK_NOTES_FOLDER,
    MOCK_TEMPLATES_FOLDER,
    reset_mock_folder,
)


@pytest.fixture(autouse=True)
def clear_init_db_cache():
    """Clear the init_db cache before and after every test."""
    init_db.cache_clear()
    yield
    init_db.cache_clear()


@pytest.fixture(autouse=True)
def isolate_mock_folders():
    """Remove leftover files so listing tests see only committed mock contents."""
    reset_mock_folder(MOCK_NOTES_FOLDER, COMMITTED_MOCK_NOTE_FILES)
    reset_mock_folder(MOCK_TEMPLATES_FOLDER, COMMITTED_MOCK_TEMPLATE_FILES)
    yield
    reset_mock_folder(MOCK_NOTES_FOLDER, COMMITTED_MOCK_NOTE_FILES)
    reset_mock_folder(MOCK_TEMPLATES_FOLDER, COMMITTED_MOCK_TEMPLATE_FILES)


@pytest.fixture()
def mock_notes_folder():
    """Point ResolvedFolder.NOTES at tests/mock_notes for filesystem-backed tests."""
    with patch.object(ResolvedFolder.NOTES, "_value_", MOCK_NOTES_FOLDER):
        yield MOCK_NOTES_FOLDER


@pytest.fixture()
def mock_templates_folder():
    """Point ResolvedFolder.TEMPLATES at tests/mock_templates."""
    with patch.object(ResolvedFolder.TEMPLATES, "_value_", MOCK_TEMPLATES_FOLDER):
        yield MOCK_TEMPLATES_FOLDER


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
        db.query_by_text = MagicMock()
        yield db
    init_db.cache_clear()
