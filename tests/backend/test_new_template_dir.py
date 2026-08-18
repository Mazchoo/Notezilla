"""Tests for the new_template_dir MCP tool."""

import pytest

from src.backend.main import new_template_dir
from tests.backend.helpers import MOCK_TEMPLATES_FOLDER, temporary_notes


class TestNewTemplateDir:
    """Tests for the new_template_dir MCP tool using tests/mock_templates."""

    def test_creates_folder(self, mock_templates_folder):
        """new_template_dir creates a folder under the template folder."""
        with temporary_notes(
            dirs=["tmp_create"], folder=MOCK_TEMPLATES_FOLDER
        ) as paths:
            target = paths["tmp_create"] / "new_folder"
            result = new_template_dir(path="tmp_create/new_folder")

            assert result.content[0].text == "Success"
            assert result.structured_content == {"warnings": []}
            assert target.is_dir()
            target.rmdir()

    def test_creates_nested_folder_with_parents(self, mock_templates_folder):
        """new_template_dir creates missing parent directories."""
        with temporary_notes(dirs=["tmp_create"], folder=MOCK_TEMPLATES_FOLDER):
            target = mock_templates_folder / "tmp_create" / "deep" / "nested"
            result = new_template_dir(path="tmp_create/deep/nested")

            assert result.content[0].text == "Success"
            assert target.is_dir()
            target.rmdir()
            target.parent.rmdir()

    def test_already_exists_returns_error(self, mock_templates_folder):
        """new_template_dir fails when the path already exists."""
        with temporary_notes(
            dirs=["tmp_create/existing"], folder=MOCK_TEMPLATES_FOLDER
        ):
            result = new_template_dir(path="tmp_create/existing")

            assert result.content[0].text.startswith("Error")
            assert result.structured_content == {"warnings": []}

    def test_invalid_path_returns_error(self, mock_templates_folder):
        """new_template_dir rejects paths outside the template folder."""
        result = new_template_dir(path="../../../etc/passwd")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"warnings": []}

    def test_does_not_create_in_note_folder(
        self, mock_notes_folder, mock_templates_folder
    ):
        """new_template_dir writes under the template folder, not the note folder."""
        with temporary_notes(
            dirs=["tmp_create"], folder=MOCK_TEMPLATES_FOLDER
        ) as paths:
            result = new_template_dir(path="tmp_create/isolated")

            assert result.content[0].text == "Success"
            assert (paths["tmp_create"] / "isolated").is_dir()
            assert not (mock_notes_folder / "tmp_create" / "isolated").exists()
            (paths["tmp_create"] / "isolated").rmdir()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
