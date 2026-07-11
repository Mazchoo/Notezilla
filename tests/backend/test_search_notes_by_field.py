"""Tests for the search_notes_by_field MCP tool."""

from unittest.mock import ANY

import pytest

from tests.backend.helpers import make_notes
from src.backend.main import search_notes_by_field
from src.backend.note import NoteData


class TestSearchNotesByField:
    """Tests for the search_notes_by_field MCP tool.

    The NoteDatabase.query_by_field method is mocked directly on the class;
    NoteDatabase.__init__ is stubbed so no ChromaDB connection is made.
    """

    def test_returns_matching_documents(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field returns documents from the database."""
        mock_db.query_by_field.return_value = make_notes(
            docs=["content of note"], metas=[{"filename": "note.md"}]
        )

        result = search_notes_by_field(field="filename", value="note.md")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == [
            NoteData(filename="note.md", text="content of note", fields={}).to_dict()
        ]

    def test_calls_db_with_correct_args(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field passes field, value, and n_results to the DB."""
        mock_db.query_by_field.return_value = make_notes()

        search_notes_by_field(field="title", value="hello", n_results=5)

        mock_db.query_by_field.assert_called_once_with("title", "hello", ANY, 5)

    def test_default_n_results_is_10(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field uses n_results=10 by default."""
        mock_db.query_by_field.return_value = make_notes()

        search_notes_by_field(field="title", value="hello")

        mock_db.query_by_field.assert_called_once_with("title", "hello", ANY, 10)

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field wraps ValueError in an error response."""
        mock_db.query_by_field.side_effect = ValueError("bad type")

        result = search_notes_by_field(field="x", value="y")

        assert result.content[0].text.startswith("Error")

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field wraps unexpected exceptions in an error response."""
        mock_db.query_by_field.side_effect = RuntimeError("connection lost")

        result = search_notes_by_field(field="x", value="y")

        assert result.content[0].text.startswith("Error")

    def test_empty_result_from_db(self, mock_db):
        """search_notes_by_field handles an empty result set gracefully."""
        mock_db.query_by_field.return_value = []

        result = search_notes_by_field(field="title", value="nonexistent")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == []


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
