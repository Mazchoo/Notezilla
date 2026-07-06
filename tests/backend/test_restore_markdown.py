"""Tests for restoring markdown notes from database rows."""

from pathlib import Path
from unittest.mock import patch

import pytest

from src.backend.database_adapter import McpNoteItem, NoteDatabase
from src.backend.database_update import prepate_database_row
from src.backend.file_io import extract_yaml_from_file_contents
from src.backend.parse_markdown import MarkdownData
from src.field_enums import FieldTypes, ReservedFields
from src.tasks.restore_missing_files import restore_missing_files


COLUMN_TYPES = {
    "title": FieldTypes.STRING,
    "date": FieldTypes.DATE,
    "tags": FieldTypes.LIST,
    "meta": FieldTypes.JSON,
}


def _chroma_metadata_from_row(row: dict) -> dict:
    """Metadata as stored by NoteDatabase.upsert_batch."""
    return {
        key: value
        for key, value in row.items()
        if value is not None and key not in ReservedFields.excluded_from_metadata()
    }


def _note_item_from_row(row: dict) -> McpNoteItem:
    metadata = _chroma_metadata_from_row(row)
    return {
        "text": row[ReservedFields.TEXT],
        "metadata": NoteDatabase._decode_frontmatter(metadata, COLUMN_TYPES),
        "filename": row[ReservedFields.FILENAME],
    }


def _fake_write_factory(store: dict):
    def _write(path: str, contents: str) -> bool:
        store[path] = contents
        return True

    return _write


class TestFrontmatterRoundTrip:
    """Front matter survives prepate_database_row -> Chroma metadata -> McpNoteItem."""

    @pytest.mark.parametrize(
        "fields",
        [
            {"title": "Example"},
            {"title": "Tagged", "tags": ["python", "testing"]},
            {"meta": {"source": "import", "version": 2}},
            {"title": "Dated", "date": "2024-06-01"},
        ],
        ids=["title_only", "tags_list", "json_field", "date_field"],
    )
    def test_row_round_trips_to_frontmatter(self, fields):
        markdown = MarkdownData(
            fields=fields,
            text="Body text",
            filename="note.md",
            path=["example_folder"],
        )
        row = prepate_database_row(markdown, COLUMN_TYPES)
        note = _note_item_from_row(row)

        assert note["metadata"] == fields
        assert note["text"] == "Body text"
        assert note["filename"] == "note.md"

    def test_empty_tags_are_omitted_from_frontmatter(self):
        markdown = MarkdownData(
            fields={"tags": []},
            text="Body",
            filename="note.md",
            path=["example_folder"],
        )
        row = prepate_database_row(markdown, COLUMN_TYPES)
        note = _note_item_from_row(row)

        assert note["metadata"] == {}


class TestRestoreMissingFiles:
    """restore_missing_files writes notes using database front matter."""

    def _run_restore(
        self,
        path_key: str,
        markdown: MarkdownData,
    ) -> tuple[dict[str, str], int]:
        row = prepate_database_row(markdown, COLUMN_TYPES)
        note = _note_item_from_row(row)
        store: dict[str, str] = {}

        with (
            patch(
                "src.tasks.restore_missing_files.check_path_sync",
                return_value=({path_key}, set()),
            ),
            patch("src.tasks.restore_missing_files.NoteDatabase") as mock_db_cls,
            patch(
                "src.tasks.restore_missing_files.note_file_path",
                side_effect=lambda key: key,
            ),
            patch(
                "src.backend.parse_markdown.ensure_note_parent_dirs",
                return_value=True,
            ),
            patch(
                "src.backend.parse_markdown.write_file_content",
                side_effect=_fake_write_factory(store),
            ),
            patch.object(Path, "exists", return_value=False),
        ):
            mock_db_cls.return_value.get_frontmatter_from_path_key.return_value = note
            count = restore_missing_files()

        return store, count

    @pytest.mark.parametrize(
        ("path_key", "fields", "body"),
        [
            (
                "example_folder/note.md",
                {"title": "Example"},
                "Body text",
            ),
            (
                "folder/another_example.md",
                {"title": "Tagged", "tags": ["rust", "zig"]},
                "# Silly Database Integration\n",
            ),
            (
                "plain.md",
                {},
                "Just plain text",
            ),
        ],
        ids=["title_only", "tags_list", "no_frontmatter"],
    )
    def test_restores_file_from_database_row(
        self, path_key: str, fields: dict, body: str
    ):
        markdown = MarkdownData(
            fields=fields,
            text=body,
            filename=Path(path_key).name,
            path=list(Path(path_key).parent.parts),
        )
        store, count = self._run_restore(path_key, markdown)

        assert count == 1
        assert len(store) == 1
        written = next(iter(store.values()))
        parsed_body, parsed_fields = extract_yaml_from_file_contents(written)
        assert parsed_body == body
        assert parsed_fields == fields

    def test_get_frontmatter_from_path_key_returns_same_row(self):
        """Mocked database method returns the McpNoteItem built from the row."""
        markdown = MarkdownData(
            fields={"title": "From DB", "tags": ["restore"]},
            text="Recovered body",
            filename="note.md",
            path=["restore"],
        )
        row = prepate_database_row(markdown, COLUMN_TYPES)
        expected = _note_item_from_row(row)

        store, count = self._run_restore("restore/note.md", markdown)

        assert count == 1
        assert expected["metadata"] == {"title": "From DB", "tags": ["restore"]}
        written = next(iter(store.values()))
        assert "title: From DB" in written
        assert "Recovered body" in written


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
