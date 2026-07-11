"""Tests for the upsert_note MCP tool."""

from pathlib import Path
from unittest.mock import patch

import pytest

from src.backend.main import upsert_note
from tests.backend.helpers import clean_up_file_if_created


class TestUpsertNote:
    """Tests for the upsert_note MCP tool using tests/mock_notes."""

    def test_upsert_note_success(self, mock_notes_folder):
        """upsert_note returns a success message when the note is written."""
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "my-note.md"
        ) as note_path:
            result = upsert_note(
                path="2024/01/my-note.md",
                contents="Hello world",
                fields={"title": "My Note"},
            )

            assert result.content[0].text == "Success"
            assert result.structured_content == {"newFileCreated": True}
            assert note_path.is_file()

    def test_upsert_note_existing_file_reports_not_created(self, mock_notes_folder):
        """upsert_note reports newFileCreated=False when the file already exists."""
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "my-note.md"
        ) as note_path:
            upsert_note(
                path="2024/01/my-note.md",
                contents="First body",
                fields={"title": "My Note"},
            )
            result = upsert_note(
                path="2024/01/my-note.md",
                contents="Updated body",
                fields={"title": "My Note"},
            )

            assert result.content[0].text == "Success"
            assert result.structured_content == {"newFileCreated": False}
            assert note_path.read_text(encoding="utf-8").endswith("Updated body")

    def test_upsert_note_failure_returns_error(self, mock_notes_folder):
        """upsert_note returns an error message when open() raises OSError."""
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "my-note.md"
        ) as note_path:
            with patch("src.backend.file_io.open", side_effect=OSError("disk full")):
                result = upsert_note(
                    path="2024/01/my-note.md",
                    contents="Hello world",
                    fields={},
                )

            assert result.content[0].text.startswith("Error")
            assert not note_path.is_file()

    def test_upsert_note_non_md_path_returns_error(self, mock_notes_folder):
        """upsert_note returns an error for paths that don't end with .md."""
        result = upsert_note(
            path="folder/note.txt",
            contents="Body text",
            fields={},
        )

        assert result.content[0].text.startswith("Error")

    def test_upsert_note_writes_correct_content(self, mock_notes_folder):
        """upsert_note writes the YAML header + body to disk."""
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "my-note.md"
        ) as note_path:
            upsert_note(
                path="2024/01/my-note.md",
                contents="Hello world",
                fields={"title": "My Note"},
            )

            written = note_path.read_text(encoding="utf-8")
            assert "Hello world" in written
            assert "My Note" in written


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
