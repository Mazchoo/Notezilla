"""Tests for the get_template MCP tool."""

import pytest

from src.backend.main import get_template
from tests.backend.helpers import MOCK_TEMPLATES_FOLDER, temporary_notes


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

    def test_reads_file_written_on_disk(self, mock_templates_folder):
        """get_template returns the current file contents from the template folder."""
        with temporary_notes(
            {"tmp_get.md": "---\ntitle: Fresh\n---\nfresh body"},
            folder=MOCK_TEMPLATES_FOLDER,
        ):
            result = get_template(path="tmp_get.md")

            assert result.content[0].text == "Success"
            template = result.structured_content["notes"][0]
            assert template["filename"] == "tmp_get.md"
            assert template["text"].strip() == "fresh body"
            assert template["metadata"]["title"] == "Fresh"


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
