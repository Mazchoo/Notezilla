"""Tests for the upsert_note MCP tool."""

from pathlib import Path
from unittest.mock import patch, mock_open

import pytest

from src.backend.main import upsert_note


class TestUpsertNote:
    """Tests for the upsert_note MCP tool.

    File I/O is mocked at src.backend.parse_markdown so no real files are touched.
    """

    def test_upsert_note_success(self):
        """upsert_note returns a success message when open() succeeds."""
        with patch("src.backend.file_io.open", mock_open()):
            result = upsert_note(
                path="2024/01/my-note.md",
                contents="Hello world",
                fields={"title": "My Note"},
            )

        assert result.content[0].text == "Success"
        assert result.structured_content == {"newFileCreated": True}

    def test_upsert_note_existing_file_reports_not_created(self):
        """upsert_note reports newFileCreated=False when the file already exists."""
        with (
            patch("src.backend.file_io.open", mock_open()),
            patch.object(Path, "exists", return_value=True),
        ):
            result = upsert_note(
                path="2024/01/my-note.md",
                contents="Updated body",
                fields={"title": "My Note"},
            )

        assert result.content[0].text == "Success"
        assert result.structured_content == {"newFileCreated": False}

    def test_upsert_note_failure_returns_error(self):
        """upsert_note returns an error message when open() raises OSError."""
        with patch("src.backend.file_io.open", side_effect=OSError("disk full")):
            result = upsert_note(
                path="notes/2024/01/my-note.md",
                contents="Hello world",
                fields={},
            )

        assert result.content[0].text.startswith("Error")

    def test_upsert_note_non_md_path_returns_error(self):
        """upsert_note returns an error for paths that don't end with .md.

        MarkdownData.construct_from_data rejects non-.md paths before open() is
        ever called, so no file-I/O mock is needed here.
        """
        result = upsert_note(
            path="folder/note.txt",
            contents="Body text",
            fields={},
        )

        assert result.content[0].text.startswith("Error")

    def test_upsert_note_writes_correct_content(self):
        """upsert_note writes the YAML header + body to open()."""
        m = mock_open()
        with patch("src.backend.file_io.open", m):
            upsert_note(
                path="2024/01/my-note.md",
                contents="Hello world",
                fields={"title": "My Note"},
            )

        written = "".join(call.args[0] for call in m().write.call_args_list)
        assert "Hello world" in written
        assert "My Note" in written


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
