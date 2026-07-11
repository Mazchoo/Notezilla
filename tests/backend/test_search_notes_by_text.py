"""Tests for the search_notes_by_text MCP tool."""

import pytest

from tests.backend.helpers import make_query_result
from src.backend.main import search_notes_by_text
from src.backend.mcp_interface import McpResponse


class TestSearchNotesByText:
    """Tests for the search_notes_by_text MCP tool.

    The NoteDatabase.query_by_text method is mocked directly on the instance.
    """

    def test_returns_matching_documents(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text returns documents from the database."""
        query_result = make_query_result(
            docs=["semantic match"], metas=[{"filename": "result.md"}]
        )
        mock_db.query_by_text.return_value = query_result

        result = search_notes_by_text(text="find something")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == [
            McpResponse.note_item("semantic match", {"filename": "result.md"}).to_dict()
        ]

    def test_calls_db_with_correct_args(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text passes text and n_results to the DB."""
        mock_db.query_by_text.return_value = make_query_result()

        search_notes_by_text(text="hello world", n_results=7)

        mock_db.query_by_text.assert_called_once_with("hello world", 7)

    def test_default_n_results_is_10(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text uses n_results=10 by default."""
        mock_db.query_by_text.return_value = make_query_result()

        search_notes_by_text(text="query")

        mock_db.query_by_text.assert_called_once_with("query", 10)

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text wraps ValueError in an error response."""
        mock_db.query_by_text.side_effect = ValueError("invalid text")

        result = search_notes_by_text(text="query")

        assert result.content[0].text.startswith("Error")

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text wraps unexpected exceptions in an error response."""
        mock_db.query_by_text.side_effect = Exception("embedding failure")

        result = search_notes_by_text(text="query")

        assert result.content[0].text.startswith("Error")

    def test_empty_result_from_db(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text handles an empty result set gracefully."""
        mock_db.query_by_text.return_value = make_query_result(docs=[], metas=[])

        result = search_notes_by_text(text="nothing matches")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == []

    def test_multiple_results_returned(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text returns all documents from the DB result."""
        multi_result = make_query_result(
            docs=["doc A", "doc B", "doc C"],
            metas=[{"filename": "a.md"}, {"filename": "b.md"}, {"filename": "c.md"}],
        )
        mock_db.query_by_text.return_value = multi_result

        result = search_notes_by_text(text="broad query", n_results=3)

        assert len(result.structured_content["notes"]) == 3


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
