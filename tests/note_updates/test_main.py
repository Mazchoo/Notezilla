"""Test high level MCP behavior"""

from unittest.mock import patch, MagicMock

import pytest

from src.note_updates.database_adapter import QueryResult
from src.note_updates.main import (
    delete_note,
    search_notes_by_field,
    search_notes_by_path,
    search_notes_by_tag,
    search_notes_by_text,
    upsert_note,
)
from src.note_updates.mcp_interface import init_db


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(autouse=True)
def clear_init_db_cache():
    """Clear the lru_cache on init_db before every test so NoteDatabase() is called fresh."""
    init_db.cache_clear()
    yield
    init_db.cache_clear()


@pytest.fixture()
def mock_db():
    """
    Patch NoteDatabase in mcp_interface so init_db() returns a MagicMock instance.
    Only the individual query methods are mocked; the rest of the adapter is untouched.
    """
    with patch("src.note_updates.mcp_interface.NoteDatabase") as mock_cls:
        db = mock_cls.return_value
        db.query_by_field = MagicMock()
        db.query_field_contains = MagicMock()
        db.query_by_path = MagicMock()
        db.query_by_text = MagicMock()
        yield db


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_query_result(docs=None, metas=None) -> QueryResult:
    return QueryResult(
        documents=docs if docs is not None else ["doc1"],
        metadatas=metas if metas is not None else [{"filename": "note.md"}],
    )


# ---------------------------------------------------------------------------
# upsert_note
# ---------------------------------------------------------------------------


class TestUpsertNote:
    """Tests for the upsert_note MCP tool.

    File I/O is mocked at src.note_updates.parse_markdown so no real files are touched.
    """

    def test_upsert_note_success(self):
        """upsert_note returns a success message when the file is written."""
        with patch(
            "src.note_updates.parse_markdown.write_file_content", return_value=True
        ):
            result = upsert_note(
                path="2024/01/my-note.md",
                contents="Hello world",
                fields={"title": "My Note"},
            )

        assert "2024/01/my-note.md" in result
        assert not result.startswith("Error:")

    def test_upsert_note_failure_returns_error(self):
        """upsert_note returns an error message when the file cannot be written."""
        with patch(
            "src.note_updates.parse_markdown.write_file_content", return_value=False
        ):
            result = upsert_note(
                path="2024/01/my-note.md",
                contents="Hello world",
                fields={},
            )

        assert result.startswith("Error:")

    def test_upsert_note_non_md_path_returns_error(self):
        """upsert_note returns an error for paths that don't end with .md."""
        with patch(
            "src.note_updates.parse_markdown.write_file_content", return_value=True
        ):
            result = upsert_note(
                path="folder/note.txt",
                contents="Body text",
                fields={},
            )

        assert result.startswith("Error:")
        assert "folder/note.txt" in result

    def test_upsert_note_does_not_touch_real_filesystem(self):
        """upsert_note never calls the real open() when parse_markdown is mocked."""
        with patch(
            "src.note_updates.parse_markdown.write_file_content", return_value=True
        ):
            with patch("builtins.open") as mock_open:
                upsert_note(path="a/b.md", contents="text", fields={})
                mock_open.assert_not_called()


# ---------------------------------------------------------------------------
# delete_note
# ---------------------------------------------------------------------------


class TestDeleteNote:
    """Tests for the delete_note MCP tool.

    delete_note_file is mocked at src.note_updates.main (where it is imported)
    so no real files are touched.
    """

    def test_delete_note_success(self):
        """delete_note returns a success message when the file is deleted."""
        with patch("src.note_updates.main.delete_note_file", return_value=True):
            result = delete_note(path="2024/01/my-note.md")

        assert result == "Note deleted at '2024/01/my-note.md'"

    def test_delete_note_failure_returns_error(self):
        """delete_note returns an error message when the file cannot be deleted."""
        with patch("src.note_updates.main.delete_note_file", return_value=False):
            result = delete_note(path="missing/note.md")

        assert result.startswith("Error:")
        assert "missing/note.md" in result

    def test_delete_note_calls_delete_with_correct_path(self):
        """delete_note passes the path argument through to delete_note_file."""
        with patch(
            "src.note_updates.main.delete_note_file", return_value=True
        ) as mock_delete:
            delete_note(path="some/path/note.md")

        mock_delete.assert_called_once_with("some/path/note.md")

    def test_delete_note_does_not_touch_real_filesystem(self):
        """delete_note never calls Path.unlink() when delete_note_file is mocked."""
        with patch("src.note_updates.main.delete_note_file", return_value=True):
            with patch("pathlib.Path.unlink") as mock_unlink:
                delete_note(path="a/b.md")
                mock_unlink.assert_not_called()


# ---------------------------------------------------------------------------
# search_notes_by_field
# ---------------------------------------------------------------------------


class TestSearchNotesByField:
    """Tests for the search_notes_by_field MCP tool.

    The NoteDatabase.query_by_field method is mocked directly on the class;
    NoteDatabase.__init__ is stubbed so no ChromaDB connection is made.
    """

    def test_returns_matching_documents(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field returns documents from the database."""
        query_result = _make_query_result(
            docs=["content of note"], metas=[{"filename": "note.md"}]
        )
        mock_db.query_by_field.return_value = query_result

        result = search_notes_by_field(field="filename", value="note.md")

        assert result["documents"] == ["content of note"]
        assert result["metadatas"] == [{"filename": "note.md"}]
        assert result["error"] is None

    def test_calls_db_with_correct_args(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field passes field, value, and n_results to the DB."""
        mock_db.query_by_field.return_value = _make_query_result()

        search_notes_by_field(field="title", value="hello", n_results=5)

        mock_db.query_by_field.assert_called_once_with("title", "hello", 5)

    def test_default_n_results_is_10(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field uses n_results=10 by default."""
        mock_db.query_by_field.return_value = _make_query_result()

        search_notes_by_field(field="title", value="hello")

        mock_db.query_by_field.assert_called_once_with("title", "hello", 10)

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field wraps ValueError in an error NoteQueryResult."""
        mock_db.query_by_field.side_effect = ValueError("bad type")

        result = search_notes_by_field(field="x", value="y")

        assert result["documents"] == []
        assert result["metadatas"] == []
        assert result["error"] is not None
        assert "Type error" in result["error"]
        assert "bad type" in result["error"]

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_field wraps unexpected exceptions in an error NoteQueryResult."""
        mock_db.query_by_field.side_effect = RuntimeError("connection lost")

        result = search_notes_by_field(field="x", value="y")

        assert result["documents"] == []
        assert result["metadatas"] == []
        assert result["error"] is not None
        assert "DB error" in result["error"]
        assert "connection lost" in result["error"]

    def test_empty_result_from_db(self, mock_db):
        """search_notes_by_field handles an empty result set gracefully."""
        mock_db.query_by_field.return_value = _make_query_result(docs=[], metas=[])

        result = search_notes_by_field(field="title", value="nonexistent")

        assert result["documents"] == []
        assert result["metadatas"] == []
        assert result["error"] is None


# ---------------------------------------------------------------------------
# search_notes_by_tag
# ---------------------------------------------------------------------------


class TestSearchNotesByTag:
    """Tests for the search_notes_by_tag MCP tool.

    The NoteDatabase.query_field_contains method is mocked directly on the instance.
    """

    def test_returns_matching_documents(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag returns documents from the database."""
        query_result = _make_query_result(
            docs=["tagged note"], metas=[{"tags\tpython": True}]
        )
        mock_db.query_field_contains.return_value = query_result

        result = search_notes_by_tag(field="tags", value="python")

        assert result["documents"] == ["tagged note"]
        assert result["error"] is None

    def test_calls_db_with_correct_args(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag passes field, value, and n_results to the DB."""
        mock_db.query_field_contains.return_value = _make_query_result()

        search_notes_by_tag(field="tags", value="python", n_results=3)

        mock_db.query_field_contains.assert_called_once_with("tags", "python", 3)

    def test_default_n_results_is_10(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag uses n_results=10 by default."""
        mock_db.query_field_contains.return_value = _make_query_result()

        search_notes_by_tag(field="tags", value="python")

        mock_db.query_field_contains.assert_called_once_with("tags", "python", 10)

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag wraps ValueError in an error NoteQueryResult."""
        mock_db.query_field_contains.side_effect = ValueError("invalid")

        result = search_notes_by_tag(field="tags", value="x")

        assert result["error"] is not None
        assert "Type error" in result["error"]

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_tag wraps unexpected exceptions in an error NoteQueryResult."""
        mock_db.query_field_contains.side_effect = Exception("boom")

        result = search_notes_by_tag(field="tags", value="x")

        assert result["error"] is not None
        assert "DB error" in result["error"]
        assert "boom" in result["error"]


# ---------------------------------------------------------------------------
# search_notes_by_path
# ---------------------------------------------------------------------------


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

        assert result["documents"] == ["note in 2024/01"]
        assert result["error"] is None

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
        assert result["documents"] == []

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_path wraps ValueError in an error NoteQueryResult."""
        mock_db.query_by_path.side_effect = ValueError("bad path")

        result = search_notes_by_path(path_parts=["x"])

        assert result["error"] is not None
        assert "Type error" in result["error"]
        assert "bad path" in result["error"]

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_path wraps unexpected exceptions in an error NoteQueryResult."""
        mock_db.query_by_path.side_effect = RuntimeError("timeout")

        result = search_notes_by_path(path_parts=["x"])

        assert result["error"] is not None
        assert "DB error" in result["error"]
        assert "timeout" in result["error"]


# ---------------------------------------------------------------------------
# search_notes_by_text
# ---------------------------------------------------------------------------


class TestSearchNotesByText:
    """Tests for the search_notes_by_text MCP tool.

    The NoteDatabase.query_by_text method is mocked directly on the instance.
    """

    def test_returns_matching_documents(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text returns documents from the database."""
        query_result = _make_query_result(
            docs=["semantic match"], metas=[{"filename": "result.md"}]
        )
        mock_db.query_by_text.return_value = query_result

        result = search_notes_by_text(text="find something")

        assert result["documents"] == ["semantic match"]
        assert result["metadatas"] == [{"filename": "result.md"}]
        assert result["error"] is None

    def test_calls_db_with_correct_args(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text passes text and n_results to the DB."""
        mock_db.query_by_text.return_value = _make_query_result()

        search_notes_by_text(text="hello world", n_results=7)

        mock_db.query_by_text.assert_called_once_with("hello world", 7)

    def test_default_n_results_is_10(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text uses n_results=10 by default."""
        mock_db.query_by_text.return_value = _make_query_result()

        search_notes_by_text(text="query")

        mock_db.query_by_text.assert_called_once_with("query", 10)

    def test_value_error_returns_type_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text wraps ValueError in an error NoteQueryResult."""
        mock_db.query_by_text.side_effect = ValueError("invalid text")

        result = search_notes_by_text(text="query")

        assert result["documents"] == []
        assert result["metadatas"] == []
        assert result["error"] is not None
        assert "Type error" in result["error"]
        assert "invalid text" in result["error"]

    def test_generic_exception_returns_db_error_message(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text wraps unexpected exceptions in an error NoteQueryResult."""
        mock_db.query_by_text.side_effect = Exception("embedding failure")

        result = search_notes_by_text(text="query")

        assert result["documents"] == []
        assert result["metadatas"] == []
        assert result["error"] is not None
        assert "DB error" in result["error"]
        assert "embedding failure" in result["error"]

    def test_empty_result_from_db(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text handles an empty result set gracefully."""
        mock_db.query_by_text.return_value = _make_query_result(docs=[], metas=[])

        result = search_notes_by_text(text="nothing matches")

        assert result["documents"] == []
        assert result["metadatas"] == []
        assert result["error"] is None

    def test_multiple_results_returned(self, mock_db):  # pylint: disable=redefined-outer-name
        """search_notes_by_text returns all documents from the DB result."""
        multi_result = _make_query_result(
            docs=["doc A", "doc B", "doc C"],
            metas=[{"filename": "a.md"}, {"filename": "b.md"}, {"filename": "c.md"}],
        )
        mock_db.query_by_text.return_value = multi_result

        result = search_notes_by_text(text="broad query", n_results=3)

        assert len(result["documents"]) == 3
        assert len(result["metadatas"]) == 3


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
