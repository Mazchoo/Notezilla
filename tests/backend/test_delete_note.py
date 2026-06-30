"""Tests for the delete_note MCP tool."""

from unittest.mock import patch

import pytest

from src.backend.main import delete_note
from src.config import NOTE_FOLDER


class TestDeleteNote:
    """Tests for the delete_note MCP tool.

    delete_note_file is mocked at src.backend.main (where it is imported)
    so no real files are touched.
    """

    def test_delete_note_success(self):
        """delete_note returns a success message when the file is deleted."""
        with patch("src.backend.main.delete_note_file", return_value=True):
            result = delete_note(path="2024/01/my-note.md")

        assert result.content[0].text == "Success"

    def test_delete_note_failure_returns_error(self):
        """delete_note returns an error message when the file cannot be deleted."""
        with patch("src.backend.main.delete_note_file", return_value=False):
            result = delete_note(path="missing/note.md")

        assert result.content[0].text.startswith("Error")

    def test_delete_note_calls_delete_with_correct_path(self):
        """delete_note passes the path argument through to delete_note_file."""
        with patch(
            "src.backend.main.delete_note_file", return_value=True
        ) as mock_delete:
            delete_note(path="some/path/note.md")

        mock_delete.assert_called_once_with(f"{NOTE_FOLDER}/some/path/note.md")

    def test_delete_note_does_not_touch_real_filesystem(self):
        """delete_note never calls Path.unlink() when delete_note_file is mocked."""
        with patch("src.backend.main.delete_note_file", return_value=True):
            with patch("pathlib.Path.unlink") as mock_unlink:
                delete_note(path="a/b.md")
                mock_unlink.assert_not_called()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
