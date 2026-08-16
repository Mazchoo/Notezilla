"""Pagination tests for NoteDatabase query methods."""

from unittest.mock import patch

import pytest

from src.backend.database_adapter import NoteDatabase
from src.backend.database_update import prepate_database_row
from src.backend.note import NoteData
from src.field_enums import ColumnTypes, FieldTypes


COLUMN_TYPES: ColumnTypes = {}


def _upsert_notes(db: NoteDatabase, notes: list[NoteData]) -> None:
    """Store prepared rows for each note."""
    db.upsert_batch([prepate_database_row(note, COLUMN_TYPES) for note in notes])


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

    def test_path_filter_keeps_only_matching_filenames(self, temp_db):
        """path_filter keeps notes whose filenames start with the prefix."""
        notes = [
            NoteData(
                filename="keep/a.md",
                text="Cats and feline companions in keep",
                fields={},
            ),
            NoteData(
                filename="drop/b.md",
                text="Cats and feline companions in drop",
                fields={},
            ),
            NoteData(
                filename="keep/nested/c.md",
                text="Cats and feline companions nested",
                fields={},
            ),
        ]
        _upsert_notes(temp_db, notes)

        results = temp_db.query_by_text(
            "cats feline", COLUMN_TYPES, n_results=10, path_filter=["keep/"]
        )

        assert {n.filename for n in results} == {"keep/a.md", "keep/nested/c.md"}

    def test_comma_separated_path_filter_matches_any_prefix(self, temp_db):
        """A path_filter list keeps notes matching any prefix."""
        notes = [
            NoteData(
                filename="keep/a.md",
                text="Cats and feline companions in keep",
                fields={},
            ),
            NoteData(
                filename="drop/b.md",
                text="Cats and feline companions in drop",
                fields={},
            ),
            NoteData(
                filename="other/c.md",
                text="Cats and feline companions in other",
                fields={},
            ),
        ]
        _upsert_notes(temp_db, notes)

        results = temp_db.query_by_text(
            "cats feline",
            COLUMN_TYPES,
            n_results=10,
            path_filter=["keep/", "other/"],
        )

        assert {n.filename for n in results} == {"keep/a.md", "other/c.md"}

    def test_blank_path_filter_ignored(self, temp_db):
        """Empty or None path_filter does not restrict results."""
        _upsert_notes(temp_db, _semantic_notes(5))
        unrestricted = temp_db.query_by_text("cats feline", COLUMN_TYPES, n_results=5)
        blank = temp_db.query_by_text(
            "cats feline", COLUMN_TYPES, n_results=5, path_filter=[]
        )
        none_filter = temp_db.query_by_text(
            "cats feline", COLUMN_TYPES, n_results=5, path_filter=None
        )

        assert [n.filename for n in blank] == [n.filename for n in unrestricted]
        assert [n.filename for n in none_filter] == [n.filename for n in unrestricted]

    def test_path_filter_restricts_query_ids_and_n_results(self, temp_db):
        """path_filter passes only matching ids and a page-sized n_results."""
        notes = _semantic_notes(8)
        notes.extend(
            [
                NoteData(
                    filename="keep/a.md",
                    text="Cats and feline companions in keep a",
                    fields={},
                ),
                NoteData(
                    filename="keep/b.md",
                    text="Cats and feline companions in keep b",
                    fields={},
                ),
            ]
        )
        _upsert_notes(temp_db, notes)

        with patch.object(
            temp_db._collection, "query", wraps=temp_db._collection.query
        ) as spy:
            results = temp_db.query_by_text(
                "cats feline", COLUMN_TYPES, n_results=2, path_filter=["keep/"]
            )

        spy.assert_called_once()
        kwargs = spy.call_args.kwargs
        assert kwargs["n_results"] == 2
        assert set(kwargs["ids"]) == {"keep/a.md", "keep/b.md"}
        assert {n.filename for n in results} == {"keep/a.md", "keep/b.md"}

    def test_path_filter_with_no_matching_ids_returns_empty(self, temp_db):
        """A prefix that matches no ids returns [] without querying."""
        _upsert_notes(temp_db, _semantic_notes(3))

        with patch.object(
            temp_db._collection, "query", wraps=temp_db._collection.query
        ) as spy:
            results = temp_db.query_by_text(
                "cats feline",
                COLUMN_TYPES,
                n_results=5,
                path_filter=["missing/"],
            )

        assert results == []
        spy.assert_not_called()


class TestQueryByTextBlankQuery:
    """Blank query returns notes without a similarity search."""

    def test_blank_query_returns_notes_without_query(self, temp_db):
        """A blank query returns stored notes and does not call query()."""
        notes = [
            NoteData(filename="a.md", text="alpha", fields={}),
            NoteData(filename="b.md", text="beta", fields={}),
        ]
        _upsert_notes(temp_db, notes)

        with patch.object(
            temp_db._collection, "query", wraps=temp_db._collection.query
        ) as spy:
            results = temp_db.query_by_text("", COLUMN_TYPES, n_results=10)

        spy.assert_not_called()
        assert {n.filename for n in results} == {"a.md", "b.md"}

    def test_whitespace_query_treated_as_blank(self, temp_db):
        """Whitespace-only text is treated as a blank query."""
        _upsert_notes(
            temp_db, [NoteData(filename="a.md", text="alpha", fields={})]
        )

        with patch.object(
            temp_db._collection, "query", wraps=temp_db._collection.query
        ) as spy:
            results = temp_db.query_by_text("   ", COLUMN_TYPES, n_results=10)

        spy.assert_not_called()
        assert [n.filename for n in results] == ["a.md"]

    def test_blank_query_with_path_filter(self, temp_db):
        """A blank query still keeps notes whose filenames match path_filter."""
        notes = [
            NoteData(filename="keep/a.md", text="in keep", fields={}),
            NoteData(filename="drop/b.md", text="in drop", fields={}),
            NoteData(filename="keep/nested/c.md", text="nested keep", fields={}),
        ]
        _upsert_notes(temp_db, notes)

        with patch.object(
            temp_db._collection, "query", wraps=temp_db._collection.query
        ) as spy:
            results = temp_db.query_by_text(
                "", COLUMN_TYPES, n_results=10, path_filter=["keep/"]
            )

        spy.assert_not_called()
        assert {n.filename for n in results} == {"keep/a.md", "keep/nested/c.md"}

    def test_blank_query_with_where_filter(self, temp_db):
        """A blank query still applies the metadata where filter."""
        column_types: ColumnTypes = {"status": FieldTypes.STRING}
        notes = [
            NoteData(
                filename="draft.md", text="draft note", fields={"status": "draft"}
            ),
            NoteData(
                filename="pub.md",
                text="published note",
                fields={"status": "published"},
            ),
        ]
        temp_db.upsert_batch(
            [prepate_database_row(note, column_types) for note in notes]
        )

        with patch.object(
            temp_db._collection, "query", wraps=temp_db._collection.query
        ) as spy:
            results = temp_db.query_by_text(
                "",
                column_types,
                n_results=10,
                where={"status": "draft"},
            )

        spy.assert_not_called()
        assert [n.filename for n in results] == ["draft.md"]

    def test_blank_query_paginates(self, temp_db):
        """offset and n_results page blank-query results without query()."""
        notes = [
            NoteData(filename=f"note-{i:02d}.md", text=f"body {i}", fields={})
            for i in range(15)
        ]
        _upsert_notes(temp_db, notes)

        with patch.object(
            temp_db._collection, "query", wraps=temp_db._collection.query
        ) as spy:
            first = temp_db.query_by_text("", COLUMN_TYPES, n_results=10, offset=0)
            second = temp_db.query_by_text(
                "", COLUMN_TYPES, n_results=10, offset=10
            )

        spy.assert_not_called()
        assert len(first) == 10
        assert len(second) == 5
        assert {n.filename for n in first}.isdisjoint({n.filename for n in second})
        assert {n.filename for n in first + second} == {
            f"note-{i:02d}.md" for i in range(15)
        }


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
