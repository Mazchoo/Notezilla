"""Tests for get_dir_contents MCP tool."""

import pytest

from src.backend.main import get_dir_contents


class TestGetDirContents:
    """Tests for get_dir_contents using tests/mock_notes as the note folder."""

    def test_root_lists_folders_and_md_files(self, mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents()

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": ["folder"],
            "files": ["example.md"],
        }

    def test_subdirectory_lists_children(self, mock_notes_folder):
        """A nested path lists only its immediate child folders and .md files."""
        result = get_dir_contents(path="folder")

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": ["sub_folder"],
            "files": ["another_example.md"],
        }

    def test_subdirectory_with_no_md_files(self, mock_notes_folder):
        """Directories without .md files still list subfolders but return no files."""
        result = get_dir_contents(path="folder/sub_folder")

        assert result.content[0].text == "Success"
        assert result.structured_content == {"folders": [], "files": []}

    def test_invalid_path_outside_note_folder(self, mock_notes_folder):
        """Paths outside the note folder are rejected with an error message."""
        result = get_dir_contents(path="../outside")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content["folders"] == []
        assert result.structured_content["files"] == []

    def test_nonexistent_path_returns_error(self, mock_notes_folder):
        """Missing directories return empty lists and a filesystem error message."""
        result = get_dir_contents(path="does/not/exist")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content["folders"] == []
        assert result.structured_content["files"] == []

    def test_empty_root_lists_folders_and_md_files(self, mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents(path="")

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": ["folder"],
            "files": ["example.md"],
        }

    def test_dot_root_lists_folders_and_md_files(self, mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents(path=".")

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": ["folder"],
            "files": ["example.md"],
        }

    def test_star_root_lists_folders_and_md_files(self, mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents(path="*")

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": ["folder"],
            "files": ["example.md"],
        }

    def test_relative_dir_root_lists_folders_and_md_files(self, mock_notes_folder):
        """Default path lists immediate subfolders and .md files at the note root."""
        result = get_dir_contents(path="./")

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": ["folder"],
            "files": ["example.md"],
        }


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
