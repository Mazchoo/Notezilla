"""Tests for template loading MCP tools."""

import pytest

from src.backend.main import get_template, get_template_dir_contents


class TestGetTemplate:
    """Tests for the get_template MCP tool (filesystem-backed)."""

    def test_returns_matching_document(self, mock_templates_folder):
        """get_template returns a single document read from the template folder."""
        result = get_template(path="animal_facts.md")

        assert result.content[0].text == "Success"
        template = result.structured_content["notes"][0]
        assert template["filename"] == "animal_facts.md"
        assert "## Animal name" in template["text"]
        assert template["metadata"] == {"tags": ["animal_facts"]}

    def test_invalid_path_returns_error(self, mock_templates_folder):
        """get_template returns an error when the path is outside the template folder."""
        result = get_template(path="../../../etc/passwd")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"notes": [], "warnings": []}

    def test_not_found_returns_error(self, mock_templates_folder):
        """get_template returns an error when the template file does not exist."""
        result = get_template(path="missing/template.md")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"notes": [], "warnings": []}

    def test_note_path_is_not_a_template(
        self, mock_notes_folder, mock_templates_folder
    ):
        """A path inside the note folder is not loaded as a template."""
        result = get_template(path=str(mock_notes_folder / "example.md"))

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"notes": [], "warnings": []}


class TestGetTemplateDirContents:
    """Tests for get_template_dir_contents using tests/mock_templates."""

    def test_root_lists_md_files(self, mock_templates_folder):
        """Default path lists immediate .md files at the template root."""
        result = get_template_dir_contents()

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": [],
            "files": ["animal_facts.md"],
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
