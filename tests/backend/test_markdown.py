"""Tests for MarkdownFile construction and parsing using real files."""

import pytest

from src.backend.parse_markdown import IMarkdownFile
from tests.backend.helpers import clean_up_file_if_created


class TestMarkdownDataRoundTrip:
    """Verify that construct_from_data and construct_from_path are inverses."""

    def _run_round_trip(self, note_path, contents: str, fields: dict) -> tuple:
        """Write to file via construct_from_data, read via construct_from_path."""
        path = str(note_path)
        from_data, _new_file_created = IMarkdownFile.construct_from_data(
            path=path, body=contents, fields=fields
        )
        assert from_data is not None, "construct_from_data returned None unexpectedly"

        from_path = IMarkdownFile.construct_from_path(path=path)
        assert from_path is not None, "construct_from_path returned None unexpectedly"

        return from_data, from_path

    def test_round_trip_fields_match(self, mock_notes_folder):
        """Fields survive a write → read cycle unchanged."""
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "my-note.md"
        ) as note_path:
            from_data, from_path = self._run_round_trip(
                note_path,
                contents="Hello world",
                fields={"title": "My Note", "tags": ["python", "testing"]},
            )

            assert from_data.fields == from_path.fields
            assert from_data.text.strip() == from_path.text.strip()
            assert from_data.filename == from_path.filename == "2024/01/my-note.md"

    def test_round_trip_empty_fields(self, mock_notes_folder):
        """A note with no YAML front-matter round-trips correctly."""
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "no-fields.md"
        ) as note_path:
            from_data, from_path = self._run_round_trip(
                note_path,
                contents="Just plain text",
                fields={},
            )

            assert from_data.fields == from_path.fields
            assert from_data.text.strip() == from_path.text.strip()
            assert from_data.filename == from_path.filename == "2024/01/no-fields.md"

    def test_round_trip_multiline_content(self, mock_notes_folder):
        """Multi-line body text is preserved through the round-trip."""
        body = "Line one\nLine two\n\nLine four after blank"
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "06" / "multi.md"
        ) as note_path:
            from_data, from_path = self._run_round_trip(
                note_path,
                contents=body,
                fields={"title": "Multi"},
            )

            assert from_data.text.strip() == from_path.text.strip()


class TestConstructFromPathExistingNotes:
    """construct_from_path against committed files in tests/mock_notes."""

    def test_reads_root_level_note(self, mock_notes_folder):
        result = IMarkdownFile.construct_from_path(
            str(mock_notes_folder / "example.md")
        )

        assert result is not None
        assert result.filename == "example.md"
        assert result.fields == {}
        assert "Hello there" in result.text

    def test_reads_nested_note_with_front_matter(self, mock_notes_folder):
        result = IMarkdownFile.construct_from_path(
            str(mock_notes_folder / "folder" / "another_example.md")
        )

        assert result is not None
        assert result.filename == "folder/another_example.md"
        assert result.fields == {
            "phase": 100,
            "tags": ["rust", "zig"],
            "status": "todo",
        }
        assert "# Silly Database Integration" in result.text

    def test_rejects_path_outside_note_folder(self, mock_notes_folder):
        assert IMarkdownFile.construct_from_path("/etc/passwd") is None

    def test_missing_file_returns_none(self, mock_notes_folder):
        assert (
            IMarkdownFile.construct_from_path(
                str(mock_notes_folder / "does-not-exist.md")
            )
            is None
        )


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
