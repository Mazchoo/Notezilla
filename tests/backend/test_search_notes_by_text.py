"""Tests for the search_notes_by_text MCP tool."""

from unittest.mock import ANY, patch

import pytest

from tests.backend.helpers import make_notes
from src.backend.main import search_notes_by_text
from src.backend.note import NoteData


class TestSearchNotesByText:
    """Tests for the search_notes_by_text MCP tool.

    The NoteDatabase.query_by_text method is mocked directly on the instance.
    """

    def test_returns_matching_documents(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text returns documents from the database."""
        mock_db.query_by_text.return_value = make_notes(
            docs=["semantic match"], metas=[{"filename": "result.md"}]
        )

        result = search_notes_by_text(text="find something")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == [
            NoteData(filename="result.md", text="semantic match", fields={}).to_dict()
        ]

    def test_calls_db_with_correct_args(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text passes text, n_results, and offset to the DB."""
        mock_db.query_by_text.return_value = make_notes()

        search_notes_by_text(text="hello world", n_results=7, offset=10)

        mock_db.query_by_text.assert_called_once_with(
            "hello world", ANY, 7, where=None, offset=10, path_filter=None
        )

    def test_default_n_results_is_10(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text uses n_results=10 by default."""
        mock_db.query_by_text.return_value = make_notes()

        search_notes_by_text(text="query")

        mock_db.query_by_text.assert_called_once_with(
            "query", ANY, 10, where=None, offset=0, path_filter=None
        )

    def test_default_offset_is_zero(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text uses offset=0 by default for pagination."""
        mock_db.query_by_text.return_value = make_notes()

        search_notes_by_text(text="query", n_results=7)

        mock_db.query_by_text.assert_called_once_with(
            "query", ANY, 7, where=None, offset=0, path_filter=None
        )

    def test_passes_pagination_offset(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text forwards pagination offset to the DB."""
        mock_db.query_by_text.return_value = make_notes()

        search_notes_by_text(text="query", n_results=10, offset=10)

        mock_db.query_by_text.assert_called_once_with(
            "query", ANY, 10, where=None, offset=10, path_filter=None
        )

    def test_passes_normalised_path_filter(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text normalises path_filter before calling the DB."""
        mock_db.query_by_text.return_value = make_notes()

        search_notes_by_text(text="query", path_filter=".\\2026\\02*")

        mock_db.query_by_text.assert_called_once_with(
            "query", ANY, 10, where=None, offset=0, path_filter=["2026/02"]
        )

    def test_blank_path_filter_sent_as_none(self, mock_db):  # pylint: disable=redefined-outer-name
        """Blank path_filter is passed as None so the DB ignores it."""
        mock_db.query_by_text.return_value = make_notes()

        result = search_notes_by_text(text="query", path_filter="")

        mock_db.query_by_text.assert_called_once_with(
            "query", ANY, 10, where=None, offset=0, path_filter=None
        )
        assert result.structured_content["warnings"] == []

    def test_passes_comma_separated_path_filters(self, mock_db):  # pylint: disable=redefined-outer-name
        """Comma-separated path_filter values are each normalised into a list."""
        mock_db.query_by_text.return_value = make_notes()

        search_notes_by_text(text="query", path_filter="2026/02,folder")

        mock_db.query_by_text.assert_called_once_with(
            "query", ANY, 10, where=None, offset=0, path_filter=["2026/02", "folder"]
        )

    def test_trims_and_normalises_comma_separated_path_filters(self, mock_db):  # pylint: disable=redefined-outer-name
        """Spaces around comma-separated paths are trimmed before normalisation."""
        mock_db.query_by_text.return_value = make_notes()

        search_notes_by_text(text="query", path_filter=" .\\2026\\02* , ./folder* ")

        mock_db.query_by_text.assert_called_once_with(
            "query",
            ANY,
            10,
            where=None,
            offset=0,
            path_filter=["2026/02", "folder"],
        )

    def test_empty_path_filter_segments_returned_as_warnings(self, mock_db):  # pylint: disable=redefined-outer-name
        """Empty path_filter segments are omitted and listed in warnings."""
        mock_db.query_by_text.return_value = make_notes()

        result = search_notes_by_text(text="query", path_filter="keep/,, drop/")

        mock_db.query_by_text.assert_called_once_with(
            "query", ANY, 10, where=None, offset=0, path_filter=["keep/", "drop/"]
        )
        assert result.structured_content["warnings"]

    def test_only_empty_path_filters_sent_as_none_with_warning(self, mock_db):  # pylint: disable=redefined-outer-name
        """A path_filter of only empty segments is passed as None and warned about."""
        mock_db.query_by_text.return_value = make_notes()

        result = search_notes_by_text(text="query", path_filter=" , , ")

        mock_db.query_by_text.assert_called_once_with(
            "query", ANY, 10, where=None, offset=0, path_filter=None
        )
        assert result.structured_content["warnings"]

    def test_passes_parsed_frontmatter_as_where(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text parses frontmatter YAML into the where filter."""
        mock_db.query_by_text.return_value = make_notes()
        column_types = {"status": "str", "tags": "list"}

        with patch("src.backend.main.init_column_types", return_value=column_types):
            search_notes_by_text(
                text="query",
                frontmatter='status: draft\ntags: ["cheese", "bread"]',
            )

        mock_db.query_by_text.assert_called_once_with(
            "query",
            column_types,
            10,
            where={
                "$and": [
                    {"status": "draft"},
                    {"tags\tcheese": True},
                    {"tags\tbread": True},
                ]
            },
            offset=0,
            path_filter=None,
        )

    def test_unknown_frontmatter_fields_returned_as_warnings(self, mock_db):  # pylint: disable=redefined-outer-name
        """Unknown frontmatter keys are omitted from where and listed in warnings."""
        mock_db.query_by_text.return_value = make_notes()
        column_types = {"status": "str"}

        with patch("src.backend.main.init_column_types", return_value=column_types):
            result = search_notes_by_text(
                text="query",
                frontmatter="status: draft\nunknown_field: ignore-me",
            )

        mock_db.query_by_text.assert_called_once_with(
            "query",
            column_types,
            10,
            where={"status": "draft"},
            offset=0,
            path_filter=None,
        )
        assert result.structured_content["warnings"]

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
        mock_db.query_by_text.return_value = []

        result = search_notes_by_text(text="nothing matches")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"] == []

    def test_multiple_results_returned(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text returns all documents from the DB result."""
        mock_db.query_by_text.return_value = make_notes(
            docs=["doc A", "doc B", "doc C"],
            metas=[{"filename": "a.md"}, {"filename": "b.md"}, {"filename": "c.md"}],
        )

        result = search_notes_by_text(text="broad query", n_results=3)

        assert len(result.structured_content["notes"]) == 3


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
