"""Shared pytest configuration and fixtures for backend MCP tool tests."""

from unittest.mock import MagicMock, patch

import pytest

from src.backend.mcp_interface import init_db


@pytest.fixture(autouse=True)
def clear_init_db_cache():
    """Clear the init_db cache before and after every test."""
    init_db.cache_clear()
    yield
    init_db.cache_clear()


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
