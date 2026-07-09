"""Tests for note parent directory creation"""

from pathlib import Path
from unittest.mock import patch, MagicMock

import pytest

from src.backend.parse_markdown import MarkdownFile
from src.backend.file_io import ensure_note_parent_dirs


class TestEnsureNoteParentDirs:
    """Unit tests for ensure_note_parent_dirs with mocked filesystem operations."""

    def test_creates_parent_dirs_for_nested_path(self):
        """Nested note paths trigger mkdir on the resolved parent folder."""
        mock_parent = MagicMock()

        with (
            patch(
                "src.backend.file_io.get_normalised_path",
                return_value="2024/01/note.md",
            ),
            patch(
                "src.backend.file_io.RESOLVED_NOTE_FOLDER",
                MagicMock(__truediv__=MagicMock(return_value=mock_parent)),
            ),
        ):
            assert ensure_note_parent_dirs("2024/01/note.md") is True

        mock_parent.mkdir.assert_called_once_with(parents=True, exist_ok=True)

    def test_skips_mkdir_for_root_level_note(self):
        """Notes at the note-folder root have no parent directories to create."""
        with (
            patch(
                "src.backend.file_io.get_normalised_path",
                return_value="note.md",
            ),
            patch.object(Path, "mkdir") as mock_mkdir,
        ):
            assert ensure_note_parent_dirs("note.md") is True

        mock_mkdir.assert_not_called()

    def test_rejects_path_outside_note_folder(self):
        """Paths outside RESOLVED_NOTE_FOLDER are rejected without touching disk."""
        with (
            patch("src.backend.file_io.get_normalised_path", return_value=None),
            patch.object(Path, "mkdir") as mock_mkdir,
        ):
            assert ensure_note_parent_dirs("/etc/passwd") is False

        mock_mkdir.assert_not_called()

    def test_returns_false_when_mkdir_raises(self):
        """OSError from mkdir is surfaced as False."""
        with (
            patch(
                "src.backend.file_io.get_normalised_path",
                return_value="deep/nested/note.md",
            ),
            patch.object(Path, "mkdir", side_effect=OSError("permission denied")),
        ):
            assert ensure_note_parent_dirs("deep/nested/note.md") is False


class TestConstructFromDataCreatesDirs:
    """construct_from_data must create containing folders before writing."""

    def test_calls_ensure_note_parent_dirs_before_write(self):
        """Parent directories are ensured before write_file_content runs."""
        store = {}
        call_order = []

        def _ensure(path: str) -> bool:
            call_order.append(("ensure", path))
            return True

        def _write(path: str, contents: str) -> bool:
            call_order.append(("write", path))
            store[path] = contents
            return True

        with (
            patch(
                "src.backend.parse_markdown.ensure_note_parent_dirs",
                side_effect=_ensure,
            ),
            patch(
                "src.backend.parse_markdown.write_file_content",
                side_effect=_write,
            ),
        ):
            result, _new_file_created = MarkdownFile.construct_from_data(
                path="2024/01/new-note.md",
                contents="body",
                fields={"title": "New"},
            )

        assert result is not None
        assert call_order == [
            ("ensure", "2024/01/new-note.md"),
            ("write", "2024/01/new-note.md"),
        ]

    def test_returns_none_when_dir_creation_fails(self):
        """Failed directory creation aborts before any file write."""
        with (
            patch(
                "src.backend.parse_markdown.ensure_note_parent_dirs",
                return_value=False,
            ),
            patch("src.backend.parse_markdown.write_file_content") as mock_write,
        ):
            result = MarkdownFile.construct_from_data(
                path="2024/01/new-note.md",
                contents="body",
                fields={},
            )

        assert result is None
        mock_write.assert_not_called()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
