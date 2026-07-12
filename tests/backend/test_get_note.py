"""Tests for the get_note MCP tool."""

import pytest

from src.backend.main import get_note, upsert_note
from tests.backend.helpers import clean_up_file_if_created


class TestGetNote:
    """Tests for the get_note MCP tool (filesystem-backed)."""

    def test_returns_matching_document(self, mock_notes_folder):  # pylint: disable=redefined-outer-name
        """get_note returns a single document read from disk."""
        result = get_note(path="example.md")

        assert result.content[0].text == "Success"
        note = result.structured_content["notes"][0]
        assert note["filename"] == "example.md"
        assert "Hello there" in note["text"]
        assert note["metadata"] == {}

    def test_returns_metadata_for_front_matter(self, mock_notes_folder):  # pylint: disable=redefined-outer-name
        """get_note includes metadata fields needed to reconstruct front matter."""
        result = get_note(path="folder/another_example.md")

        note = result.structured_content["notes"][0]
        assert note["filename"] == "folder/another_example.md"
        assert "# Silly Database Integration" in note["text"]
        assert note["metadata"] == {
            "phase": 100,
            "tags": ["rust", "zig"],
            "status": "todo",
        }

    def test_backslash_path_normalises(self, mock_notes_folder):  # pylint: disable=redefined-outer-name
        """Expect key to be universal to get the same result each time"""
        result = get_note(path=r"folder\another_example.md")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"][0]["filename"] == (
            "folder/another_example.md"
        )

    def test_relative_path_normalises(self, mock_notes_folder):  # pylint: disable=redefined-outer-name
        """Expect key to be universal to get the same result each time"""
        result = get_note(path="./folder/another_example.md")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"][0]["filename"] == (
            "folder/another_example.md"
        )

    def test_backtrack_path_normalises(self, mock_notes_folder):  # pylint: disable=redefined-outer-name
        """Expect key to be universal to get the same result each time"""
        result = get_note(path="./folder/../folder/another_example.md")

        assert result.content[0].text == "Success"
        assert result.structured_content["notes"][0]["filename"] == (
            "folder/another_example.md"
        )

    def test_invalid_path_returns_error(self, mock_notes_folder):  # pylint: disable=redefined-outer-name
        """get_note returns an error when the path is outside the note folder."""
        result = get_note(path="../../../etc/passwd")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"notes": []}

    def test_not_found_returns_error(self, mock_notes_folder):  # pylint: disable=redefined-outer-name
        """get_note returns an error when the note file does not exist."""
        result = get_note(path="missing/note.md")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"notes": []}

    def test_reads_latest_content_after_write(self, mock_notes_folder):  # pylint: disable=redefined-outer-name
        """Save then get_note must return the overwritten file contents."""
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "overwrite-me.md"
        ) as note_path:
            first = upsert_note(
                path="2024/01/overwrite-me.md",
                contents="original body",
                fields={"title": "Original"},
            )
            assert first.content[0].text == "Success"

            second = upsert_note(
                path="2024/01/overwrite-me.md",
                contents="updated body",
                fields={"title": "Updated"},
            )
            assert second.content[0].text == "Success"
            assert second.structured_content["newFileCreated"] is False

            result = get_note(path="2024/01/overwrite-me.md")

            assert result.content[0].text == "Success"
            note = result.structured_content["notes"][0]
            assert note["text"].strip() == "updated body"
            assert note["metadata"]["title"] == "Updated"
            assert note_path.exists()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
