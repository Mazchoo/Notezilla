"""Tests for the get_template MCP tool."""

import pytest

from src.backend.main import get_template, upsert_template
from tests.backend.helpers import MOCK_TEMPLATES_FOLDER, clean_up_file_if_created


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

    def test_reads_latest_content_after_write(self, mock_templates_folder):
        """Save then get_template must return the overwritten file contents."""
        with clean_up_file_if_created(
            mock_templates_folder / "overwrite-me.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ) as template_path:
            first = upsert_template(
                path="overwrite-me.md",
                contents="original body",
                fields={"title": "Original"},
            )
            assert first.content[0].text == "Success"

            second = upsert_template(
                path="overwrite-me.md",
                contents="updated body",
                fields={"title": "Updated"},
            )
            assert second.content[0].text == "Success"
            assert second.structured_content["newFileCreated"] is False

            result = get_template(path="overwrite-me.md")

            assert result.content[0].text == "Success"
            template = result.structured_content["notes"][0]
            assert template["text"].strip() == "updated body"
            assert template["metadata"]["title"] == "Updated"
            assert template_path.exists()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
