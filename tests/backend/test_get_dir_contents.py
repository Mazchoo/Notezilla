"""Tests for get_dir_contents MCP tool."""

from pathlib import Path
from unittest.mock import patch

import pytest

from src.backend.main import get_dir_contents


MOCK_NOTES_FOLDER = Path(__file__).resolve().parent.parent / "mock_notes"


@pytest.fixture()
def _mock_notes_folder():
    """Point NOTE_FOLDER at tests/mock_notes for filesystem-backed tests."""
    resolved = MOCK_NOTES_FOLDER.resolve()
    with (
        patch("src.backend.main.NOTE_FOLDER", str(resolved)),
        patch("src.backend.file_io.NOTE_FOLDER", str(resolved)),
        patch("src.backend.file_io.RESOLVED_NOTE_FOLDER", resolved),
    ):
        yield resolved


class TestGetDirContents:
    """Tests for get_dir_contents using tests/mock_notes as the note folder."""

    def test_root_lists_folders_and_md_files(self, _mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents()

        assert result == {
            "folders": ["folder"],
            "files": ["example.md"],
            "error": None,
        }

    def test_subdirectory_lists_children(self, _mock_notes_folder):
        """A nested path lists only its immediate child folders and .md files."""
        result = get_dir_contents(path="folder")

        assert result == {
            "folders": ["sub_folder"],
            "files": ["another_example.md"],
            "error": None,
        }

    def test_subdirectory_with_no_md_files(self, _mock_notes_folder):
        """Directories without .md files still list subfolders but return no files."""
        result = get_dir_contents(path="folder/sub_folder")

        assert result == {"folders": [], "files": [], "error": None}

    def test_invalid_path_outside_note_folder(self, _mock_notes_folder):
        """Paths outside the note folder are rejected with an error message."""
        result = get_dir_contents(path="../outside")

        assert result["folders"] == []
        assert result["files"] == []
        assert result["error"] is not None

    def test_nonexistent_path_returns_error(self, _mock_notes_folder):
        """Missing directories return empty lists and a filesystem error message."""
        result = get_dir_contents(path="does/not/exist")

        assert result["folders"] == []
        assert result["files"] == []
        assert result["error"] is not None

    def test_empty_root_lists_folders_and_md_files(self, _mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents(path="")

        assert result == {
            "folders": ["folder"],
            "files": ["example.md"],
            "error": None,
        }

    def test_dot_root_lists_folders_and_md_files(self, _mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents(path=".")

        assert result == {
            "folders": ["folder"],
            "files": ["example.md"],
            "error": None,
        }

    def test_star_root_lists_folders_and_md_files(self, _mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents(path="*")

        assert result == {
            "folders": ["folder"],
            "files": ["example.md"],
            "error": None,
        }

    def test_relative_dir_root_lists_folders_and_md_files(self, _mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents(path="./")

        assert result == {
            "folders": ["folder"],
            "files": ["example.md"],
            "error": None,
        }


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
