"""Tests for the delete_template MCP tool."""

import pytest

from src.backend.main import delete_template
from tests.backend.helpers import MOCK_TEMPLATES_FOLDER, temporary_notes


class TestDeleteTemplate:
    """Tests for the delete_template MCP tool using tests/mock_templates."""

    def test_delete_template_success(self, mock_templates_folder):
        """delete_template returns success and removes the file from the template folder."""
        with temporary_notes(
            {"tmp_del.md": "# temp\n"}, folder=MOCK_TEMPLATES_FOLDER
        ) as paths:
            result = delete_template(path="tmp_del.md")

            assert result.content[0].text == "Success"
            assert result.structured_content == {"warnings": []}
            assert not paths["tmp_del.md"].is_file()

    def test_delete_template_missing_file_returns_error(self, mock_templates_folder):
        """delete_template returns an error when the template file does not exist."""
        result = delete_template(path="missing/template.md")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"warnings": []}

    def test_delete_template_invalid_path_returns_error(self, mock_templates_folder):
        """delete_template returns an error when the path is outside the template folder."""
        result = delete_template(path="../../../etc/passwd")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"warnings": []}

    def test_delete_template_does_not_delete_note(
        self, mock_notes_folder, mock_templates_folder
    ):
        """A path inside the note folder is not deleted as a template."""
        note_path = mock_notes_folder / "example.md"
        assert note_path.is_file()

        result = delete_template(path=str(note_path))

        assert result.content[0].text.startswith("Error")
        assert note_path.is_file()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
