"""Tests for the move_dir MCP tool (relative paths, as clients send them)."""

import pytest

from src.backend.main import move_dir, upsert_note
from tests.backend.helpers import clean_up_file_if_created, temporary_notes


class TestMoveDirTool:
    """MCP move_dir must accept vault-relative paths like make_request.py."""

    def test_moves_upserted_note_into_folder(self, mock_notes_folder):
        """upsert then move with relative paths — the make_request.py scenario."""
        with temporary_notes(dirs=["tmp_move_dst"]) as paths:
            with clean_up_file_if_created(
                mock_notes_folder / "tmp_move_note.md"
            ) as src_path:
                upsert = upsert_note(
                    path="tmp_move_note.md",
                    contents="Hello world",
                    fields={"title": "My Note"},
                )
                assert upsert.content[0].text == "Success"
                assert src_path.is_file()

                result = move_dir(src="tmp_move_note.md", dst="tmp_move_dst")

                assert result.content[0].text == "Success"
                assert not src_path.exists()
                moved = paths["tmp_move_dst"] / "tmp_move_note.md"
                assert moved.is_file()
                assert "Hello world" in moved.read_text(encoding="utf-8")
                moved.unlink()

    def test_move_failure_sets_is_error(self, mock_notes_folder):
        """Failed moves must surface isError=True on the MCP wire result."""
        result = move_dir(src="missing.md", dst="also-missing")

        assert result.content[0].text.startswith("Error")
        wire = result.to_mcp_result()
        assert wire.isError is True


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
