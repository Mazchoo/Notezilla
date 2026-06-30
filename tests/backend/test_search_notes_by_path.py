"""Tests for the search_notes_by_path MCP tool."""

import pytest

from helpers import _make_query_result
from src.backend.main import search_notes_by_path


class TestSearchNotesByPath:
    """Tests for the search_notes_by_path MCP tool.

    The NoteDatabase.query_by_path method is mocked directly on the instance.
    """

    def test_returns_matching_documents(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_path returns documents from the database."""
        query_result = _make_query_result(
            docs=["note in 2024/01"], metas=[{"filename": "note.md"}]
        )
        mock_db.query_by_path.return_value = query_result

        result = search_notes_by_path(path_parts=["2024", "01"])

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == [
            {"filename": "note.md", "text": "note in 2024/01"}
        ]

    def test_calls_db_with_correct_args(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_path passes path_parts and n_results to the DB."""
        mock_db.query_by_path.return_value = _make_query_result()

        search_notes_by_path(path_parts=["2024", "01"], n_results=50)

        mock_db.query_by_path.assert_called_once_with(["2024", "01"], 50)

    def test_default_n_results_is_100(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_path uses n_results=100 by default."""
        mock_db.query_by_path.return_value = _make_query_result()

        search_notes_by_path(path_parts=["2024"])

        mock_db.query_by_path.assert_called_once_with(["2024"], 100)

    def test_empty_path_parts(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_path handles an empty path_parts list."""
        mock_db.query_by_path.return_value = _make_query_result(docs=[], metas=[])

        result = search_notes_by_path(path_parts=[])

        mock_db.query_by_path.assert_called_once_with([], 100)
        assert result.structured_content["notes"] == []

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_path wraps ValueError in an error response."""
        mock_db.query_by_path.side_effect = ValueError("bad path")

        result = search_notes_by_path(path_parts=["x"])

        assert result.content[0].text.startswith("Error")

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_path wraps unexpected exceptions in an error response."""
        mock_db.query_by_path.side_effect = RuntimeError("timeout")

        result = search_notes_by_path(path_parts=["x"])

        assert result.content[0].text.startswith("Error")


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
