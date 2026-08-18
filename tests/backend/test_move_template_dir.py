"""Tests for the move_template_dir MCP tool."""

import pytest

from src.backend.main import move_template_dir, upsert_template
from tests.backend.helpers import (
    MOCK_TEMPLATES_FOLDER,
    clean_up_file_if_created,
    temporary_notes,
)


class TestMoveTemplateDir:
    """Tests for the move_template_dir MCP tool using tests/mock_templates."""

    def test_moves_upserted_template_into_folder(self, mock_templates_folder):
        """upsert then move with relative paths under the template folder."""
        with temporary_notes(
            dirs=["tmp_move_dst"], folder=MOCK_TEMPLATES_FOLDER
        ) as paths:
            with clean_up_file_if_created(
                mock_templates_folder / "tmp_move_note.md",
                folder=MOCK_TEMPLATES_FOLDER,
            ) as src_path:
                upsert = upsert_template(
                    path="tmp_move_note.md",
                    contents="Hello template",
                    fields={"title": "My Template"},
                )
                assert upsert.content[0].text == "Success"
                assert src_path.is_file()

                result = move_template_dir(
                    src="tmp_move_note.md", dst="tmp_move_dst"
                )

                assert result.content[0].text == "Success"
                assert not src_path.exists()
                moved = paths["tmp_move_dst"] / "tmp_move_note.md"
                assert moved.is_file()
                assert "Hello template" in moved.read_text(encoding="utf-8")
                moved.unlink()

    def test_move_failure_sets_is_error(self, mock_templates_folder):
        """Failed moves must surface isError=True on the MCP wire result."""
        result = move_template_dir(src="missing.md", dst="also-missing")

        assert result.content[0].text.startswith("Error")
        wire = result.to_mcp_result()
        assert wire.isError is True

    def test_does_not_move_note(
        self, mock_notes_folder, mock_templates_folder
    ):
        """A path inside the note folder is not moved as a template."""
        with temporary_notes(
            dirs=["tmp_move_dst"], folder=MOCK_TEMPLATES_FOLDER
        ):
            note_path = mock_notes_folder / "example.md"
            assert note_path.is_file()

            result = move_template_dir(
                src=str(note_path), dst="tmp_move_dst"
            )

            assert result.content[0].text.startswith("Error")
            assert note_path.is_file()
            assert not (mock_templates_folder / "tmp_move_dst" / "example.md").exists()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
