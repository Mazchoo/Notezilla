"""Tests for the move_dir MCP tool (relative paths, as clients send them)."""

import pytest

from src.backend.main import move_dir
from tests.backend.helpers import temporary_notes


class TestMoveDirTool:
    """MCP move_dir must accept note-folder-relative paths."""

    def test_moves_file_into_folder(self, mock_notes_folder):
        """move_dir moves an existing file into a destination folder."""
        with temporary_notes(
            {"tmp_move_note.md": "Hello world"},
            dirs=["tmp_move_dst"],
        ) as paths:
            src_path = paths["tmp_move_note.md"]
            result = move_dir(src="tmp_move_note.md", dst="tmp_move_dst")

            assert result.content[0].text == "Success"
            assert not src_path.exists()
            moved = paths["tmp_move_dst"] / "tmp_move_note.md"
            assert moved.is_file()
            assert "Hello world" in moved.read_text(encoding="utf-8")

    def test_move_failure_sets_is_error(self, mock_notes_folder):
        """Failed moves must surface isError=True on the MCP wire result."""
        result = move_dir(src="missing.md", dst="also-missing")

        assert result.content[0].text.startswith("Error")
        wire = result.to_mcp_result()
        assert wire.isError is True


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
