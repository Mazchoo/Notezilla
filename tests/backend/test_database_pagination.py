"""Pagination tests for NoteDatabase query methods."""

import pytest

from src.backend.database_adapter import NoteDatabase
from src.backend.database_update import prepate_database_row
from src.backend.note import NoteData
from src.field_enums import FieldTypes


COLUMN_TYPES = {
    "title": FieldTypes.STRING,
    "tags": FieldTypes.LIST,
}


def _upsert_notes(db: NoteDatabase, notes: list[NoteData]) -> None:
    """Store prepared rows for each note."""
    db.upsert_batch([prepate_database_row(note, COLUMN_TYPES) for note in notes])


def _shared_title_notes(count: int) -> list[NoteData]:
    """Build *count* notes that share title=shared for field queries."""
    return [
        NoteData(
            filename=f"note-{i:02d}.md",
            text=f"Body for note {i}",
            fields={"title": "shared"},
        )
        for i in range(count)
    ]


def _shared_tag_notes(count: int) -> list[NoteData]:
    """Build *count* notes that all contain the tag python."""
    return [
        NoteData(
            filename=f"tagged-{i:02d}.md",
            text=f"Tagged body {i}",
            fields={"tags": ["python"]},
        )
        for i in range(count)
    ]


def _semantic_notes(count: int) -> list[NoteData]:
    """Build *count* notes with related text for semantic search."""
    return [
        NoteData(
            filename=f"semantic-{i:02d}.md",
            text=f"Cats and feline companions number {i}",
            fields={},
        )
        for i in range(count)
    ]


class TestQueryByFieldPagination:
    """Pagination for query_by_field."""

    def test_offset_and_n_results_return_page_slice(self, temp_db):
        """offset=10 and n_results=10 yield results[10:20]."""
        _upsert_notes(temp_db, _shared_title_notes(25))
        full = temp_db.query_by_field("title", "shared", COLUMN_TYPES, n_results=25)
        page = temp_db.query_by_field(
            "title", "shared", COLUMN_TYPES, n_results=10, offset=10
        )

        assert [n.filename for n in page] == [n.filename for n in full[10:20]]
        assert len(page) == 10

    def test_default_offset_is_zero(self, temp_db):
        """With offset omitted, the first page of results is returned."""
        _upsert_notes(temp_db, _shared_title_notes(15))
        first_page = temp_db.query_by_field(
            "title", "shared", COLUMN_TYPES, n_results=10
        )
        explicit = temp_db.query_by_field(
            "title", "shared", COLUMN_TYPES, n_results=10, offset=0
        )

        assert [n.filename for n in first_page] == [n.filename for n in explicit]

    def test_offset_beyond_matches_returns_empty(self, temp_db):
        """An offset past the match count returns no notes."""
        _upsert_notes(temp_db, _shared_title_notes(5))
        page = temp_db.query_by_field(
            "title", "shared", COLUMN_TYPES, n_results=10, offset=10
        )

        assert page == []


class TestQueryFieldContainsPagination:
    """Pagination for query_field_contains."""

    def test_offset_and_n_results_return_page_slice(self, temp_db):
        """offset=10 and n_results=10 yield results[10:20]."""
        _upsert_notes(temp_db, _shared_tag_notes(25))
        full = temp_db.query_field_contains(
            "tags", "python", COLUMN_TYPES, n_results=25
        )
        page = temp_db.query_field_contains(
            "tags", "python", COLUMN_TYPES, n_results=10, offset=10
        )

        assert [n.filename for n in page] == [n.filename for n in full[10:20]]
        assert len(page) == 10

    def test_default_offset_is_zero(self, temp_db):
        """With offset omitted, the first page of results is returned."""
        _upsert_notes(temp_db, _shared_tag_notes(15))
        first_page = temp_db.query_field_contains(
            "tags", "python", COLUMN_TYPES, n_results=10
        )
        explicit = temp_db.query_field_contains(
            "tags", "python", COLUMN_TYPES, n_results=10, offset=0
        )

        assert [n.filename for n in first_page] == [n.filename for n in explicit]


class TestQueryByTextPagination:
    """Pagination for query_by_text."""

    def test_offset_and_n_results_return_page_slice(self, temp_db):
        """offset=10 and n_results=10 yield results[10:20]."""
        _upsert_notes(temp_db, _semantic_notes(25))
        full = temp_db.query_by_text("cats feline", COLUMN_TYPES, n_results=25)
        page = temp_db.query_by_text(
            "cats feline", COLUMN_TYPES, n_results=10, offset=10
        )

        assert [n.filename for n in page] == [n.filename for n in full[10:20]]
        assert len(page) == 10

    def test_default_offset_is_zero(self, temp_db):
        """With offset omitted, the first page of results is returned."""
        _upsert_notes(temp_db, _semantic_notes(15))
        first_page = temp_db.query_by_text("cats feline", COLUMN_TYPES, n_results=10)
        explicit = temp_db.query_by_text(
            "cats feline", COLUMN_TYPES, n_results=10, offset=0
        )

        assert [n.filename for n in first_page] == [n.filename for n in explicit]

    def test_offset_beyond_matches_returns_empty(self, temp_db):
        """An offset past the match count returns no notes."""
        _upsert_notes(temp_db, _semantic_notes(5))
        page = temp_db.query_by_text(
            "cats feline", COLUMN_TYPES, n_results=10, offset=10
        )

        assert page == []


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
