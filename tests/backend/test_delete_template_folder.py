"""Tests for the delete_template_folder MCP tool."""

import pytest

from src.backend.main import delete_template_folder
from tests.backend.helpers import MOCK_TEMPLATES_FOLDER, temporary_notes


class TestDeleteTemplateFolder:
    """Tests for the delete_template_folder MCP tool using tests/mock_templates."""

    def test_deletes_folder_recursively(self, mock_templates_folder):
        """delete_template_folder removes the directory and its contents."""
        with temporary_notes(
            {
                "tmp_del/a/one.md": "one",
                "tmp_del/a/b/two.md": "two",
            },
            folder=MOCK_TEMPLATES_FOLDER,
        ) as paths:
            result = delete_template_folder(path="tmp_del/a")

            assert result.content[0].text == "Success"
            assert result.structured_content == {"warnings": []}
            assert not (mock_templates_folder / "tmp_del" / "a").exists()
            assert not paths["tmp_del/a/one.md"].exists()
            assert not paths["tmp_del/a/b/two.md"].exists()

    def test_missing_folder_returns_error(self, mock_templates_folder):
        """delete_template_folder returns an error when the directory does not exist."""
        result = delete_template_folder(path="tmp_del/missing")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"warnings": []}

    def test_file_path_returns_error(self, mock_templates_folder):
        """delete_template_folder does not delete a file path."""
        with temporary_notes(
            {"tmp_del/solo.md": "x"}, folder=MOCK_TEMPLATES_FOLDER
        ) as paths:
            result = delete_template_folder(path="tmp_del/solo.md")

            assert result.content[0].text.startswith("Error")
            assert paths["tmp_del/solo.md"].is_file()

    def test_invalid_path_returns_error(self, mock_templates_folder):
        """delete_template_folder rejects paths outside the template folder."""
        result = delete_template_folder(path="../../../etc/passwd")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"warnings": []}

    def test_does_not_delete_note_folder(
        self, mock_notes_folder, mock_templates_folder
    ):
        """A path inside the note folder is not deleted as a template folder."""
        note_folder = mock_notes_folder / "folder"
        assert note_folder.is_dir()

        result = delete_template_folder(path=str(note_folder))

        assert result.content[0].text.startswith("Error")
        assert note_folder.is_dir()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
