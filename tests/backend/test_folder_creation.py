"""Tests for note parent directory creation using real files."""

from pathlib import Path
from unittest.mock import patch

import pytest

from src.backend.parse_markdown import IMarkdownFile
from src.backend.file_io import ensure_note_parent_dirs
from tests.backend.helpers import clean_up_file_if_created


class TestEnsureNoteParentDirs:
    """ensure_note_parent_dirs creates real directories under tests/mock_notes."""

    def test_creates_parent_dirs_for_nested_path(self, mock_notes_folder):
        """Nested note paths create missing parent folders on disk."""
        with clean_up_file_if_created(
            mock_notes_folder / "deep" / "nested" / "note.md"
        ) as note_path:
            assert ensure_note_parent_dirs(str(note_path)) is True
            assert note_path.parent.is_dir()

    def test_skips_mkdir_for_root_level_note(self, mock_notes_folder):
        """Notes at the note-folder root need no new directories."""
        note_path = mock_notes_folder / "example.md"
        assert ensure_note_parent_dirs(str(note_path)) is True
        assert not note_path.is_dir()

    def test_rejects_path_outside_note_folder(self, mock_notes_folder):
        """Paths outside the note folder are rejected without creating dirs."""
        assert ensure_note_parent_dirs("/etc/passwd") is False


class TestConstructFromDataCreatesDirs:
    """construct_from_data must create containing folders before writing."""

    def test_creates_containing_folders_and_writes_note(self, mock_notes_folder):
        """Parent directories are created and the note is written to disk."""
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "new-note.md"
        ) as note_path:
            result, new_file_created = IMarkdownFile.construct_from_data(
                path=str(note_path),
                body="body",
                fields={"title": "New"},
            )

            assert result is not None
            assert new_file_created is True
            assert result.filename == "2024/01/new-note.md"
            assert result.project_path == note_path
            assert note_path.is_file()
            assert note_path.parent.is_dir()
            written = note_path.read_text(encoding="utf-8")
            assert "title: New" in written
            assert written.endswith("body")

    def test_new_file_created_false_when_updating_existing_note(
        self, mock_notes_folder
    ):
        """Overwrite uses project_path for exists(), not the raw caller path.

        get_normalised_path strips a trailing '.', so Path(path).exists() would
        be False while the real note under NOTE_FOLDER already exists.
        """
        with clean_up_file_if_created(
            mock_notes_folder / "2024" / "01" / "existing.md"
        ) as note_path:
            first, created = IMarkdownFile.construct_from_data(
                path=str(note_path),
                body="first",
                fields={"title": "Existing"},
            )
            assert first is not None
            assert created is True

            result, new_file_created = IMarkdownFile.construct_from_data(
                path=str(note_path),
                body="updated",
                fields={"title": "Existing"},
            )

            assert result is not None
            assert new_file_created is False
            assert result.project_path == note_path
            assert note_path.read_text(encoding="utf-8").endswith("updated")

    def test_returns_none_for_path_outside_note_folder(self, mock_notes_folder):
        """Paths outside the note folder abort before any write."""
        result = IMarkdownFile.construct_from_data(
            path="/etc/passwd.md",
            body="body",
            fields={},
        )

        assert result is None

    def test_returns_none_when_dir_creation_fails(self, mock_notes_folder):
        """Failed directory creation aborts before any file write."""
        with clean_up_file_if_created(
            mock_notes_folder / "blocked" / "nested" / "note.md"
        ) as note_path:
            with patch.object(Path, "mkdir", side_effect=OSError("permission denied")):
                result = IMarkdownFile.construct_from_data(
                    path=str(note_path),
                    body="body",
                    fields={},
                )

            assert result is None
            assert not note_path.exists()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
