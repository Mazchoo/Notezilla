"""Tests for the search_notes_by_tag MCP tool."""

from unittest.mock import ANY

import pytest

from tests.backend.helpers import make_notes
from src.backend.main import search_notes_by_tag
from src.backend.note import NoteData


class TestSearchNotesByTag:
    """Tests for the search_notes_by_tag MCP tool.

    The NoteDatabase.query_field_contains method is mocked directly on the instance.
    """

    def test_returns_matching_documents(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag returns documents from the database."""
        mock_db.query_field_contains.return_value = [
            NoteData(
                filename="note.md",
                text="tagged note",
                fields={"tags": ["python"]},
            )
        ]

        result = search_notes_by_tag(field="tags", value="python")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == [
            NoteData(
                filename="note.md",
                text="tagged note",
                fields={"tags": ["python"]},
            ).to_dict()
        ]

    def test_calls_db_with_correct_args(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag passes field, value, and n_results to the DB."""
        mock_db.query_field_contains.return_value = make_notes()

        search_notes_by_tag(field="tags", value="python", n_results=3)

        mock_db.query_field_contains.assert_called_once_with("tags", "python", ANY, 3)

    def test_default_n_results_is_10(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag uses n_results=10 by default."""
        mock_db.query_field_contains.return_value = make_notes()

        search_notes_by_tag(field="tags", value="python")

        mock_db.query_field_contains.assert_called_once_with("tags", "python", ANY, 10)

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag wraps ValueError in an error response."""
        mock_db.query_field_contains.side_effect = ValueError("invalid")

        result = search_notes_by_tag(field="tags", value="x")

        assert result.content[0].text.startswith("Error")

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag wraps unexpected exceptions in an error response."""
        mock_db.query_field_contains.side_effect = Exception("boom")

        result = search_notes_by_tag(field="tags", value="x")

        assert result.content[0].text.startswith("Error")


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
