"""Tests for the rename_template_dir MCP tool."""

import shutil

import pytest

from src.backend.main import rename_template_dir
from tests.backend.helpers import MOCK_TEMPLATES_FOLDER, temporary_notes


class TestRenameTemplateDir:
    """Tests for the rename_template_dir MCP tool using tests/mock_templates."""

    def test_renames_file_preserving_extension(self, mock_templates_folder):
        """Renaming my-template.md to 'renamed' must yield renamed.md."""
        with temporary_notes(
            {"tmp_rename/my-template.md": "hello"},
            folder=MOCK_TEMPLATES_FOLDER,
        ) as paths:
            result = rename_template_dir(
                path="tmp_rename/my-template.md", new_name="renamed"
            )

            src = paths["tmp_rename/my-template.md"]
            dst = mock_templates_folder / "tmp_rename" / "renamed.md"
            assert result.content[0].text == "Success"
            assert not src.exists()
            assert not (mock_templates_folder / "tmp_rename" / "renamed").exists()
            assert dst.is_file()
            assert dst.read_text(encoding="utf-8") == "hello"
            dst.unlink()

    def test_renames_directory(self, mock_templates_folder):
        """rename_template_dir renames a directory and keeps its contents."""
        with temporary_notes(
            {
                "tmp_rename/old-dir/a.md": "a",
                "tmp_rename/old-dir/b.md": "b",
            },
            folder=MOCK_TEMPLATES_FOLDER,
        ):
            result = rename_template_dir(
                path="tmp_rename/old-dir", new_name="new-dir"
            )

            src = mock_templates_folder / "tmp_rename" / "old-dir"
            dst = mock_templates_folder / "tmp_rename" / "new-dir"
            assert result.content[0].text == "Success"
            assert not src.exists()
            assert (dst / "a.md").read_text(encoding="utf-8") == "a"
            assert (dst / "b.md").read_text(encoding="utf-8") == "b"
            shutil.rmtree(dst)

    def test_missing_path_returns_error(self, mock_templates_folder):
        """rename_template_dir returns an error when the path does not exist."""
        result = rename_template_dir(path="tmp_rename/missing.md", new_name="new.md")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"warnings": []}

    def test_does_not_rename_note(
        self, mock_notes_folder, mock_templates_folder
    ):
        """A path inside the note folder is not renamed as a template."""
        note_path = mock_notes_folder / "example.md"
        assert note_path.is_file()

        result = rename_template_dir(path=str(note_path), new_name="renamed.md")

        assert result.content[0].text.startswith("Error")
        assert note_path.is_file()
        assert not (mock_templates_folder / "renamed.md").exists()
        assert not (mock_notes_folder / "renamed.md").exists()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
