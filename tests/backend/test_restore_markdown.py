"""Tests that markdown notes survive Chroma storage and can be fully recovered.

Load file contents via MarkdownFile.construct_from_path (with read mocked),
store in the database, recover NoteData, and assert to_file_string() matches.
"""

from pathlib import Path
from unittest.mock import patch

import pytest

from src.backend.database_adapter import NoteDatabase
from src.backend.database_update import prepate_database_row
from src.backend.note import NoteData
from src.backend.parse_markdown import MarkdownFile
from src.field_enums import FieldTypes, ReservedFields


COLUMN_TYPES = {
    "title": FieldTypes.STRING,
    "date": FieldTypes.DATE,
    "tags": FieldTypes.LIST,
    "meta": FieldTypes.JSON,
    "author": FieldTypes.STRING,
}


def _load_from_path(filename: str, contents: str) -> MarkdownFile:
    """Build a MarkdownFile via construct_from_path with file read mocked."""
    with (
        patch(
            "src.backend.parse_markdown.read_file_content",
            return_value=contents,
        ),
        patch(
            "src.backend.parse_markdown.get_normalised_path",
            return_value="/".join(Path(filename).parts),
        ),
    ):
        markdown = MarkdownFile.construct_from_path(path=filename)
    assert markdown is not None
    return markdown


def _assert_file_contents_recover(
    db: NoteDatabase,
    filename: str,
    contents: str,
    *,
    expected: str | None = None,
) -> NoteData:
    """Load via construct_from_path, store, recover, assert to_file_string().

    By default ``contents`` is the expected recovered markdown. Pass ``expected``
    only when storage transforms fields (dropped keys, etc.).
    """
    original = _load_from_path(filename, contents)
    db.upsert_batch([prepate_database_row(original, COLUMN_TYPES)])
    recovered = db.get_frontmatter_from_path_key(filename, COLUMN_TYPES)
    assert recovered is not None
    assert recovered.filename == filename
    assert recovered.to_file_string() == (
        contents if expected is None else expected
    )
    return recovered


@pytest.fixture()
def temp_db(tmp_path):
    """Isolated Chroma database for one test."""
    return NoteDatabase(path=str(tmp_path / "chroma_db"))


class TestDatabaseRoundTrip:
    """Stored file contents round-trip through NoteData.to_file_string()."""

    def test_plain_note_body_survives_without_front_matter_fields(self, temp_db):
        """Assert a note with no YAML header round-trips with empty fields."""
        _assert_file_contents_recover(
            temp_db,
            filename="plain.md",
            contents="No front matter at all.",
        )

    def test_title_field_survives_database_storage(self, temp_db):
        """Assert a single string front matter field round-trips unchanged."""
        _assert_file_contents_recover(
            temp_db,
            filename="projects/readme.md",
            contents="---\ntitle: Example\n---\nBody text",
        )

    def test_tag_list_survives_and_is_sorted_on_recovery(self, temp_db):
        """Assert list tags are recovered in sorted order after Chroma storage."""
        _assert_file_contents_recover(
            temp_db,
            filename="folder/tagged.md",
            contents=(
                "---\n"
                "tags:\n"
                "- python\n"
                "- rust\n"
                "- zig\n"
                "title: Tagged\n"
                "---\n"
                "# Heading\n"
                "\n"
                "Paragraph under heading."
            ),
        )

    def test_nested_json_metadata_survives_database_storage(self, temp_db):
        """Assert nested JSON front matter round-trips through json.dumps/loads."""
        _assert_file_contents_recover(
            temp_db,
            filename="imports/legacy.md",
            contents=(
                "---\n"
                "meta:\n"
                "  nested:\n"
                "  - 1\n"
                "  - 2\n"
                "  source: import\n"
                "  version: 2\n"
                "---\n"
                "Imported note body"
            ),
        )

    def test_date_field_survives_database_storage(self, temp_db):
        """Assert ISO date strings in front matter round-trip unchanged."""
        _assert_file_contents_recover(
            temp_db,
            filename="journal/2024-06-01.md",
            contents="---\ndate: '2024-06-01'\ntitle: Dated\n---\nEntry for the day",
        )

    def test_unicode_and_emoji_in_front_matter_and_body_survive(self, temp_db):
        """Assert non-ASCII characters survive UTF-8 storage and recovery."""
        _assert_file_contents_recover(
            temp_db,
            filename="unicode/notes.md",
            contents=(
                "---\n"
                "author: \"\\xC5sa O'Brien\"\n"
                "title: \"\\u65E5\\u672C\\u8A9E\\u30BF\\u30A4\\u30C8\\u30EB \\U0001F389\"\n"
                "---\n"
                "Body with emoji 🔥 and CJK 漢字\n"
                "Second line."
            ),
        )

    def test_yaml_special_characters_in_field_values_survive(self, temp_db):
        """Assert colons, quotes, and apostrophes in values round-trip correctly."""
        _assert_file_contents_recover(
            temp_db,
            filename="yaml/tricky.md",
            contents=(
                "---\n"
                "tags:\n"
                "- UPPER\n"
                "- has-dash\n"
                "- has.dot\n"
                "title: 'Title: with \"quotes\" and apostrophe''s'\n"
                "---\n"
                "Values that stress YAML quoting."
            ),
        )

    def test_body_containing_yaml_delimiters_does_not_corrupt_recovery(self, temp_db):
        """Assert --- lines inside the body are not mistaken for front matter."""
        _assert_file_contents_recover(
            temp_db,
            filename="body/fake-delimiter.md",
            contents=(
                "---\n"
                "title: Delimiter trap\n"
                "---\n"
                "---\n"
                "This is not front matter\n"
                "---\n"
                "\n"
                "Real body continues."
            ),
        )

    def test_multiline_body_with_blank_lines_survives(self, temp_db):
        """Assert newlines and blank lines in the body are preserved exactly."""
        _assert_file_contents_recover(
            temp_db,
            filename="body/multiline.md",
            contents=(
                "---\n"
                "title: Multiline\n"
                "---\n"
                "Line one\n"
                "\n"
                "Line three after blank\n"
                "\n"
                "\n"
                "Two trailing blanks above"
            ),
        )

    def test_empty_body_survives_database_storage(self, temp_db):
        """Assert a note with front matter but no body text round-trips."""
        _assert_file_contents_recover(
            temp_db,
            filename="body/empty.md",
            contents="---\ntitle: Empty body\n---\n",
        )

    def test_deeply_nested_filename_path_survives(self, temp_db):
        """Assert folder segments in the path key are stored and recovered."""
        _assert_file_contents_recover(
            temp_db,
            filename="deep/nested/folder/note.md",
            contents="---\ntitle: Deep path\n---\nNested folder body",
        )

    def test_empty_tag_list_is_omitted_from_recovered_fields(self, temp_db):
        """Assert an empty tags list is not stored and does not reappear on read."""
        _assert_file_contents_recover(
            temp_db,
            filename="tags/empty-list.md",
            contents="---\ntags: []\n---\nEmpty tag list should not reappear as a field.",
            expected="Empty tag list should not reappear as a field.",
        )

    def test_reserved_field_names_are_stripped_before_storage(self, temp_db):
        """Assert text and filename keys in front matter never enter metadata."""
        _assert_file_contents_recover(
            temp_db,
            filename="fields/reserved.md",
            contents=(
                "---\n"
                "title: Safe title\n"
                f"{ReservedFields.TEXT}: must be ignored\n"
                f"{ReservedFields.FILENAME}: evil.md\n"
                "---\n"
                "Reserved keys must not pollute stored metadata."
            ),
            expected=(
                "---\n"
                "title: Safe title\n"
                "---\n"
                "Reserved keys must not pollute stored metadata."
            ),
        )

    def test_unregistered_front_matter_fields_are_dropped(self, temp_db):
        """Assert fields absent from column_types are not stored in the database."""
        _assert_file_contents_recover(
            temp_db,
            filename="fields/unknown.md",
            contents=(
                "---\n"
                "title: Known\n"
                "unregistered: dropped\n"
                "---\n"
                "Unknown fields are not stored."
            ),
            expected=(
                "---\n"
                "title: Known\n"
                "---\n"
                "Unknown fields are not stored."
            ),
        )

    def test_leading_and_trailing_whitespace_in_body_is_preserved(self, temp_db):
        """Assert insignificant-looking whitespace in the body is not trimmed."""
        _assert_file_contents_recover(
            temp_db,
            filename="body/whitespace.md",
            contents=(
                "---\n"
                "title: Whitespace\n"
                "---\n"
                "  \n"
                "\n"
                "  leading and trailing spaces  \n"
            ),
        )

    def test_multiple_notes_remain_independent_in_one_database(self, temp_db):
        """Assert storing several notes does not mix up their bodies or fields."""
        notes = [
            ("a/first.md", "---\ntitle: First\n---\none"),
            ("b/second.md", "---\ntitle: Second\n---\ntwo"),
            ("c/third.md", "three"),
        ]
        for filename, contents in notes:
            temp_db.upsert_batch(
                [prepate_database_row(_load_from_path(filename, contents), COLUMN_TYPES)]
            )

        for filename, contents in notes:
            recovered = temp_db.get_frontmatter_from_path_key(filename, COLUMN_TYPES)
            assert recovered is not None
            assert recovered.to_file_string() == contents

    def test_reupsert_replaces_previous_note_at_same_path(self, temp_db):
        """Assert writing a second row with the same path overwrites the first."""
        path = "overwrite/me.md"
        temp_db.upsert_batch(
            [
                prepate_database_row(
                    _load_from_path(path, "---\ntitle: v1\n---\noriginal body"),
                    COLUMN_TYPES,
                )
            ]
        )
        replacement_contents = "---\ntitle: v2\n---\nreplacement body"
        temp_db.upsert_batch(
            [
                prepate_database_row(
                    _load_from_path(path, replacement_contents), COLUMN_TYPES
                )
            ]
        )

        recovered = temp_db.get_frontmatter_from_path_key(path, COLUMN_TYPES)
        assert recovered is not None
        assert recovered.to_file_string() == replacement_contents

    def test_missing_path_returns_none(self, temp_db):
        """Assert get_frontmatter_from_path_key returns None for unknown ids."""
        assert (
            temp_db.get_frontmatter_from_path_key("does/not/exist.md", COLUMN_TYPES)
            is None
        )

    def test_tab_inside_tag_value_corrupts_list_encoding(self, temp_db):
        """Assert tabs in tag values break field\\titem storage (known limitation)."""
        filename = "broken/tag-tab.md"
        contents = (
            "---\n"
            "tags:\n"
            "- good\n"
            "- \"bad\\tvalue\"\n"
            "---\n"
            "Tab inside a tag value cannot round-trip."
        )
        temp_db.upsert_batch(
            [prepate_database_row(_load_from_path(filename, contents), COLUMN_TYPES)]
        )

        recovered = temp_db.get_frontmatter_from_path_key(filename, COLUMN_TYPES)
        assert recovered is not None
        assert recovered.to_file_string() != contents

    def test_raw_on_disk_markdown_file_round_trips_through_database(self, temp_db):
        """Assert storing literal file contents recovers the same markdown string."""
        _assert_file_contents_recover(
            temp_db,
            filename="raw/literal.md",
            contents=(
                "---\n"
                "tags:\n"
                "- alpha\n"
                "- beta\n"
                "title: From raw file\n"
                "---\n"
                "# Heading\n"
                "\n"
                "Paragraph with **bold** and a `code` span.\n"
            ),
        )


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
