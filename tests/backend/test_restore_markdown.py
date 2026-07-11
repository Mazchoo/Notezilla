"""Tests that markdown notes survive Chroma storage and can be fully recovered.

prepate_database_row and NoteDatabase.get_frontmatter_from_path_key must be
inverses: store a note, read it back, and reconstruct the same markdown file.
"""

from unittest.mock import patch

import pytest

from src.backend.database_adapter import NoteDatabase
from src.backend.database_update import prepate_database_row
from src.backend.file_io import extract_yaml_from_file_contents
from src.backend.parse_markdown import MarkdownFile
from src.field_enums import FieldTypes, ReservedFields


COLUMN_TYPES = {
    "title": FieldTypes.STRING,
    "date": FieldTypes.DATE,
    "tags": FieldTypes.LIST,
    "meta": FieldTypes.JSON,
    "author": FieldTypes.STRING,
}


def _fake_write_factory(store: dict):
    def _write(path: str, contents: str) -> bool:
        store[path] = contents
        return True

    return _write


def _store_in_database(
    db: NoteDatabase, markdown: MarkdownFile, column_types: dict
):
    row = prepate_database_row(markdown, column_types)
    db.upsert_batch([row])


def _recover_from_database(
    db: NoteDatabase, path_key: str, column_types: dict
) -> MarkdownFile | None:
    note = db.get_frontmatter_from_path_key(path_key, column_types)
    if note is None:
        return None
    return MarkdownFile(
        fields=note.fields,
        text=note.text,
        filename=note.filename,
    )


def _write_recovered_file(path: str, markdown: MarkdownFile) -> str:
    """Serialize recovered note data and return the written file payload."""
    store: dict[str, str] = {}
    with (
        patch(
            "src.backend.parse_markdown.ensure_note_parent_dirs",
            return_value=True,
        ),
        patch(
            "src.backend.parse_markdown.write_file_content",
            side_effect=_fake_write_factory(store),
        ),
    ):
        result = MarkdownFile.construct_from_data(path, markdown.text, markdown.fields)
    assert result is not None
    return next(iter(store.values()))


def _assert_note_round_trips_through_database(
    db: NoteDatabase,
    filename: str,
    fields: dict,
    text: str,
    *,
    expected_fields: dict | None = None,
) -> MarkdownFile:
    """Store a note, read it back, and assert filename/text/fields match."""
    expected = fields if expected_fields is None else expected_fields
    original = MarkdownFile(fields=fields, text=text, filename=filename)
    _store_in_database(db, original, COLUMN_TYPES)

    recovered = _recover_from_database(db, filename, COLUMN_TYPES)
    assert recovered is not None
    assert recovered.filename == filename
    assert recovered.text == text
    assert recovered.fields == expected
    return recovered


def _assert_file_recovers_from_database(
    db: NoteDatabase,
    filename: str,
    fields: dict,
    text: str,
    *,
    expected_fields: dict | None = None,
) -> None:
    """Assert a full markdown file can be rebuilt from stored database content."""
    expected = fields if expected_fields is None else expected_fields
    recovered = _assert_note_round_trips_through_database(
        db, filename, fields, text, expected_fields=expected_fields
    )
    written = _write_recovered_file(filename, recovered)
    parsed_body, parsed_fields = extract_yaml_from_file_contents(written)
    assert parsed_body == text
    assert parsed_fields == expected


@pytest.fixture()
def temp_db(tmp_path):
    """Isolated Chroma database for one test."""
    return NoteDatabase(path=str(tmp_path / "chroma_db"))


class TestDatabaseRoundTrip:
    """prepate_database_row and get_frontmatter_from_path_key are inverses."""

    def test_plain_note_body_survives_without_front_matter_fields(self, temp_db):
        """Assert a note with no YAML header round-trips with empty fields."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="plain.md",
            fields={},
            text="No front matter at all.",
        )

    def test_title_field_survives_database_storage(self, temp_db):
        """Assert a single string front matter field round-trips unchanged."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="projects/readme.md",
            fields={"title": "Example"},
            text="Body text",
        )

    def test_tag_list_survives_and_is_sorted_on_recovery(self, temp_db):
        """Assert list tags are recovered in sorted order after Chroma storage."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="folder/tagged.md",
            fields={"title": "Tagged", "tags": ["rust", "zig", "python"]},
            text="# Heading\n\nParagraph under heading.",
            expected_fields={"title": "Tagged", "tags": ["python", "rust", "zig"]},
        )

    def test_nested_json_metadata_survives_database_storage(self, temp_db):
        """Assert nested JSON front matter round-trips through json.dumps/loads."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="imports/legacy.md",
            fields={"meta": {"source": "import", "version": 2, "nested": [1, 2]}},
            text="Imported note body",
        )

    def test_date_field_survives_database_storage(self, temp_db):
        """Assert ISO date strings in front matter round-trip unchanged."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="journal/2024-06-01.md",
            fields={"title": "Dated", "date": "2024-06-01"},
            text="Entry for the day",
        )

    def test_unicode_and_emoji_in_front_matter_and_body_survive(self, temp_db):
        """Assert non-ASCII characters survive UTF-8 storage and recovery."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="unicode/notes.md",
            fields={"title": "日本語タイトル 🎉", "author": "Åsa O'Brien"},
            text="Body with emoji 🔥 and CJK 漢字\nSecond line.",
        )

    def test_yaml_special_characters_in_field_values_survive(self, temp_db):
        """Assert colons, quotes, and apostrophes in values round-trip correctly."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="yaml/tricky.md",
            fields={
                "title": 'Title: with "quotes" and apostrophe\'s',
                "tags": ["has-dash", "has.dot", "UPPER"],
            },
            text="Values that stress YAML quoting.",
            expected_fields={
                "title": 'Title: with "quotes" and apostrophe\'s',
                "tags": ["UPPER", "has-dash", "has.dot"],
            },
        )

    def test_body_containing_yaml_delimiters_does_not_corrupt_recovery(self, temp_db):
        """Assert --- lines inside the body are not mistaken for front matter."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="body/fake-delimiter.md",
            fields={"title": "Delimiter trap"},
            text="---\nThis is not front matter\n---\n\nReal body continues.",
        )

    def test_multiline_body_with_blank_lines_survives(self, temp_db):
        """Assert newlines and blank lines in the body are preserved exactly."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="body/multiline.md",
            fields={"title": "Multiline"},
            text="Line one\n\nLine three after blank\n\n\nTwo trailing blanks above",
        )

    def test_empty_body_survives_database_storage(self, temp_db):
        """Assert a note with front matter but no body text round-trips."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="body/empty.md",
            fields={"title": "Empty body"},
            text="",
        )

    def test_deeply_nested_filename_path_survives(self, temp_db):
        """Assert folder segments in the path key are stored and recovered."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="deep/nested/folder/note.md",
            fields={"title": "Deep path"},
            text="Nested folder body",
        )

    def test_empty_tag_list_is_omitted_from_recovered_fields(self, temp_db):
        """Assert an empty tags list is not stored and does not reappear on read."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="tags/empty-list.md",
            fields={"tags": []},
            text="Empty tag list should not reappear as a field.",
            expected_fields={},
        )

    def test_reserved_field_names_are_stripped_before_storage(self, temp_db):
        """Assert text and filename keys in front matter never enter metadata."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="fields/reserved.md",
            fields={
                "title": "Safe title",
                ReservedFields.TEXT: "must be ignored",
                ReservedFields.FILENAME: "evil.md",
            },
            text="Reserved keys must not pollute stored metadata.",
            expected_fields={"title": "Safe title"},
        )

    def test_unregistered_front_matter_fields_are_dropped(self, temp_db):
        """Assert fields absent from column_types are not stored in the database."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="fields/unknown.md",
            fields={"title": "Known", "unregistered": "dropped"},
            text="Unknown fields are not stored.",
            expected_fields={"title": "Known"},
        )

    def test_leading_and_trailing_whitespace_in_body_is_preserved(self, temp_db):
        """Assert insignificant-looking whitespace in the body is not trimmed."""
        _assert_file_recovers_from_database(
            temp_db,
            filename="body/whitespace.md",
            fields={"title": "Whitespace"},
            text="  \n\n  leading and trailing spaces  \n",
        )


class TestDatabaseEdgeCases:
    """Scenarios that are easy to get wrong when splitting body and metadata."""

    def test_multiple_notes_remain_independent_in_one_database(self, temp_db):
        """Assert storing several notes does not mix up their bodies or fields."""
        notes = [
            ("a/first.md", {"title": "First"}, "one"),
            ("b/second.md", {"title": "Second"}, "two"),
            ("c/third.md", {}, "three"),
        ]
        for filename, fields, text in notes:
            _store_in_database(
                temp_db,
                MarkdownFile(fields=fields, text=text, filename=filename),
                COLUMN_TYPES,
            )

        for filename, fields, text in notes:
            recovered = _recover_from_database(temp_db, filename, COLUMN_TYPES)
            assert recovered is not None
            assert recovered.text == text
            assert recovered.fields == fields

    def test_reupsert_replaces_previous_note_at_same_path(self, temp_db):
        """Assert writing a second row with the same path overwrites the first."""
        path = "overwrite/me.md"
        _store_in_database(
            temp_db,
            MarkdownFile(
                fields={"title": "v1"}, text="original body", filename=path
            ),
            COLUMN_TYPES,
        )
        _store_in_database(
            temp_db,
            MarkdownFile(
                fields={"title": "v2"}, text="replacement body", filename=path
            ),
            COLUMN_TYPES,
        )

        recovered = _recover_from_database(temp_db, path, COLUMN_TYPES)
        assert recovered is not None
        assert recovered.text == "replacement body"
        assert recovered.fields == {"title": "v2"}

    def test_missing_path_returns_none(self, temp_db):
        """Assert get_frontmatter_from_path_key returns None for unknown ids."""
        assert (
            _recover_from_database(temp_db, "does/not/exist.md", COLUMN_TYPES) is None
        )

    def test_tab_inside_tag_value_corrupts_list_encoding(self, temp_db):
        """Assert tabs in tag values break field\\titem storage (known limitation)."""
        filename = "broken/tag-tab.md"
        fields = {"tags": ["good", "bad\tvalue"]}
        original = MarkdownFile(
            fields=fields,
            text="Tab inside a tag value cannot round-trip.",
            filename=filename,
        )
        _store_in_database(temp_db, original, COLUMN_TYPES)

        recovered = _recover_from_database(temp_db, filename, COLUMN_TYPES)
        assert recovered is not None
        assert recovered.fields.get("tags") != fields["tags"]

    def test_raw_on_disk_markdown_file_round_trips_through_database(self, temp_db):
        """Assert parsing a literal file, storing it, and rebuilding yields same content."""
        filename = "raw/literal.md"
        raw_content = (
            "---\n"
            "title: From raw file\n"
            "tags:\n"
            "  - alpha\n"
            "  - beta\n"
            "---\n"
            "# Heading\n"
            "\n"
            "Paragraph with **bold** and a `code` span.\n"
        )
        body, fields = extract_yaml_from_file_contents(raw_content)
        original = MarkdownFile(fields=fields, text=body, filename=filename)

        _store_in_database(temp_db, original, COLUMN_TYPES)
        recovered = _recover_from_database(temp_db, filename, COLUMN_TYPES)
        assert recovered is not None

        written = _write_recovered_file(filename, recovered)
        reparsed_body, reparsed_fields = extract_yaml_from_file_contents(written)

        assert reparsed_body == body
        assert reparsed_fields == {
            "title": "From raw file",
            "tags": ["alpha", "beta"],
        }


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
