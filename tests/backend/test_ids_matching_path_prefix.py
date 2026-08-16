"""Tests for NoteDatabase._ids_matching_path_prefix."""

from unittest.mock import patch

import pytest


def _get_pages(pages: list[list[str]]):
    """Build a get() side_effect that returns id pages by offset / limit."""

    def fake_get(*, include, limit, offset):
        page_index = offset // limit if limit else 0
        if page_index >= len(pages):
            return {"ids": []}
        return {"ids": pages[page_index]}

    return fake_get


class TestIdsMatchingPathPrefix:
    """Prefix filtering of collection ids."""

    def test_empty_prefix_list_returns_empty_without_get(self, temp_db):
        """An empty prefix list returns [] and does not scan the index."""
        with patch.object(temp_db._collection, "get") as spy:
            ids = temp_db._ids_matching_path_prefix([])

        assert ids == []
        spy.assert_not_called()

    def test_keeps_ids_starting_with_a_single_prefix(self, temp_db):
        """Only ids that start with the prefix are returned."""
        with patch.object(
            temp_db._collection,
            "get",
            side_effect=_get_pages(
                [["keep/a.md", "drop/b.md", "keep/nested/c.md"]]
            ),
        ):
            ids = temp_db._ids_matching_path_prefix(["keep/"])

        assert ids == ["keep/a.md", "keep/nested/c.md"]

    def test_keeps_ids_starting_with_any_of_several_prefixes(self, temp_db):
        """An id is kept when it matches any listed prefix."""
        with patch.object(
            temp_db._collection,
            "get",
            side_effect=_get_pages(
                [["keep/a.md", "drop/b.md", "other/c.md"]]
            ),
        ):
            ids = temp_db._ids_matching_path_prefix(["keep/", "other/"])

        assert ids == ["keep/a.md", "other/c.md"]

    def test_returns_empty_when_no_ids_match(self, temp_db):
        """A prefix that matches no ids returns []."""
        with patch.object(
            temp_db._collection,
            "get",
            side_effect=_get_pages([["drop/a.md", "drop/b.md"]]),
        ):
            ids = temp_db._ids_matching_path_prefix(["keep/"])

        assert ids == []

    def test_returns_empty_when_collection_has_no_ids(self, temp_db):
        """An empty collection returns [] after one get()."""
        with patch.object(
            temp_db._collection, "get", side_effect=_get_pages([[]])
        ) as spy:
            ids = temp_db._ids_matching_path_prefix(["keep/"])

        assert ids == []
        spy.assert_called_once()

    def test_pages_until_a_short_batch(self, temp_db, monkeypatch):
        """A short final page ends the scan before MAX_ID_SCAN_BATCHES."""
        monkeypatch.setattr("src.backend.database_adapter.BATCH_SIZE", 2)
        pages = [["keep/a.md", "drop/b.md"], ["keep/c.md"]]

        with patch.object(
            temp_db._collection, "get", side_effect=_get_pages(pages)
        ) as spy:
            ids = temp_db._ids_matching_path_prefix(["keep/"])

        assert ids == ["keep/a.md", "keep/c.md"]
        assert spy.call_count == 2
        spy.assert_any_call(include=[], limit=2, offset=0)
        spy.assert_any_call(include=[], limit=2, offset=2)

    def test_stops_after_max_id_scan_batches(self, temp_db, monkeypatch):
        """The scan stops after MAX_ID_SCAN_BATCHES full pages."""
        monkeypatch.setattr("src.backend.database_adapter.BATCH_SIZE", 2)
        monkeypatch.setattr("src.backend.database_adapter.MAX_ID_SCAN_BATCHES", 2)

        def always_full(*, include, limit, offset):
            return {"ids": [f"keep/{offset}.md", f"keep/{offset + 1}.md"]}

        with patch.object(
            temp_db._collection, "get", side_effect=always_full
        ) as spy:
            ids = temp_db._ids_matching_path_prefix(["keep/"])

        assert spy.call_count == 2
        assert ids == ["keep/0.md", "keep/1.md", "keep/2.md", "keep/3.md"]


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
