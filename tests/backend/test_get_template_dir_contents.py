"""Tests for the get_template_dir_contents MCP tool."""

import pytest

from src.backend.main import get_template_dir_contents


class TestGetTemplateDirContents:
    """Tests for get_template_dir_contents using tests/mock_templates."""

    def test_root_lists_md_files(self, mock_templates_folder):
        """Default path lists immediate subfolders and .md files at the template root."""
        result = get_template_dir_contents()

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": ["folder"],
            "files": ["animal_facts.md"],
            "warnings": [],
        }

    def test_subdirectory_lists_children(self, mock_templates_folder):
        """A nested path lists only its immediate child folders and .md files."""
        result = get_template_dir_contents(path="folder")

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": [],
            "files": ["nested.md"],
            "warnings": [],
        }

    def test_invalid_path_outside_template_folder(self, mock_templates_folder):
        """Paths outside the template folder are rejected with an error message."""
        result = get_template_dir_contents(path="../outside")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content["folders"] == []
        assert result.structured_content["files"] == []

    def test_nonexistent_path_returns_error(self, mock_templates_folder):
        """Missing directories return empty lists and a filesystem error message."""
        result = get_template_dir_contents(path="does/not/exist")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content["folders"] == []
        assert result.structured_content["files"] == []


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
