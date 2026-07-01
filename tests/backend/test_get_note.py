"""Tests for the get_note MCP tool."""

import pytest

from src.backend.main import get_note
from src.backend.mcp_interface import McpResponse
from tests.backend.helpers import _make_query_result


class TestGetNote:
    """Tests for the get_note MCP tool."""

    def test_returns_matching_document(self, mock_db):  # pylint: disable=redefined-outer-name
        """get_note returns a single document from the database."""
        query_result = _make_query_result(
            docs=["note content"], metas=[{"filename": "note.md"}]
        )
        mock_db.query_by_id.return_value = query_result

        result = get_note(path="2024/01/note.md")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == [
            McpResponse.note_item("note content", {"filename": "note.md"})
        ]
        mock_db.query_by_id.assert_called_once_with("2024/01/note.md")

    def test_returns_metadata_for_front_matter(self, mock_db):  # pylint: disable=redefined-outer-name
        """get_note includes metadata fields needed to reconstruct front matter."""
        meta = {
            "filename": "note.md",
            "title": "My Note",
            "tags\twork": True,
            "\npath_depth_0": "2024",
            "text": "ignored body copy",
        }
        query_result = _make_query_result(docs=["note content"], metas=[meta])
        mock_db.query_by_id.return_value = query_result

        result = get_note(path="2024/01/note.md")

        assert result.structured_content["notes"] == [
            {
                "filename": "note.md",
                "text": "note content",
                "metadata": {
                    "filename": "note.md",
                    "title": "My Note",
                    "tags\twork": True,
                },
            }
        ]

    def test_backslash_path_calls_normalised_path(self, mock_db):  # pylint: disable=redefined-outer-name
        """Expect key to be universal to get the same result each time"""
        get_note(path=r"2024\01\note.md")

        mock_db.query_by_id.assert_called_once_with("2024/01/note.md")

    def test_relative_path_calls_normalised_path(self, mock_db):  # pylint: disable=redefined-outer-name
        """Expect key to be universal to get the same result each time"""
        get_note(path="./2024/01/note.md")

        mock_db.query_by_id.assert_called_once_with("2024/01/note.md")

    def test_backtrack_path_calls_normalised_path(self, mock_db):  # pylint: disable=redefined-outer-name
        """Expect key to be universal to get the same result each time"""
        get_note(path="./2024/01/../01/note.md")

        mock_db.query_by_id.assert_called_once_with("2024/01/note.md")

    def test_invalid_path_returns_error(self, mock_db):  # pylint: disable=redefined-outer-name
        """get_note returns an error when the path is outside the note folder."""
        result = get_note(path="../../../etc/passwd")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {}
        mock_db.query_by_id.assert_not_called()

    def test_not_found_returns_error(self, mock_db):  # pylint: disable=redefined-outer-name
        """get_note returns an error when the note is not in the database."""
        mock_db.query_by_id.return_value = _make_query_result(docs=[], metas=[])

        result = get_note(path="missing/note.md")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {}

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """get_note wraps ValueError in an error response."""
        mock_db.query_by_id.side_effect = ValueError("bad id")

        result = get_note(path="note.md")

        assert result.content[0].text.startswith("Error")

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """get_note wraps unexpected exceptions in an error response."""
        mock_db.query_by_id.side_effect = RuntimeError("connection lost")

        result = get_note(path="note.md")

        assert result.content[0].text.startswith("Error")


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
