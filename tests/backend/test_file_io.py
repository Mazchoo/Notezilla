"""Tests for src.backend.file_io core helpers."""

from pathlib import Path
from unittest.mock import mock_open, patch

import pytest

from src.backend.file_io import (
    ensure_note_parent_dirs,
    extract_yaml_from_file_contents,
    get_normalised_path,
    read_file_content,
    write_file_content,
)
from tests.backend.helpers import MOCK_NOTES_FOLDER


EXAMPLE_MD = MOCK_NOTES_FOLDER / "example.md"
ANOTHER_EXAMPLE_MD = MOCK_NOTES_FOLDER / "folder" / "another_example.md"


# ---------------------------------------------------------------------------
# extract_yaml_from_file_contents
# ---------------------------------------------------------------------------


class TestExtractYamlFromFileContents:
    """Parse front matter from real mock_notes payloads."""

    def test_plain_markdown_without_front_matter(self):
        """Notes without --- blocks keep full content as body text."""
        content = EXAMPLE_MD.read_text(encoding="utf-8")

        text, fields = extract_yaml_from_file_contents(content)

        assert fields == {}
        assert text == content

    def test_yaml_front_matter_and_body(self):
        """Front matter fields are extracted; body follows the closing ---."""
        content = ANOTHER_EXAMPLE_MD.read_text(encoding="utf-8")

        text, fields = extract_yaml_from_file_contents(content)

        assert fields == {
            "phase": 100,
            "tags": ["rust", "zig"],
            "status": "todo",
        }
        assert "# Silly Database Integration" in text
        assert "Just add some random content" in text
        assert text.startswith("\n#")

    def test_body_immediately_after_front_matter_has_no_leading_newline(self):
        content = (
            "---\n"
            "date: 2018-04-13\n"
            "tags: [journal, paragraph]\n"
            "---\n"
            "There was a polar bear made out of used toilet paper."
        )

        text, fields = extract_yaml_from_file_contents(content)

        assert fields["tags"] == ["journal", "paragraph"]
        assert "date" in fields
        assert text == "There was a polar bear made out of used toilet paper."
        assert not text.startswith("\n")

    def test_malformed_yaml_returns_full_content(self):
        """Invalid YAML yields empty fields and preserves the raw file."""
        content = "---\nphase: [unclosed\n---\nBody after bad yaml"

        text, fields = extract_yaml_from_file_contents(content)

        assert fields == {}
        assert text == content


# ---------------------------------------------------------------------------
# get_normalised_path
# ---------------------------------------------------------------------------


class TestGetNormalisedPath:
    """Normalise vault-relative paths using tests/mock_notes."""

    def test_root_level_note(self, mock_notes_folder):
        assert get_normalised_path(str(mock_notes_folder / "example.md")) == (
            "example.md"
        )

    def test_nested_note(self, mock_notes_folder):
        assert (
            get_normalised_path(
                str(mock_notes_folder / "folder" / "another_example.md")
            )
            == "folder/another_example.md"
        )

    def test_strips_trailing_dot(self, mock_notes_folder):
        assert get_normalised_path(str(mock_notes_folder / "folder.")) == "folder"

    def test_strips_trailing_star(self, mock_notes_folder):
        assert get_normalised_path(str(mock_notes_folder / "folder*")) == "folder"

    def test_rejects_path_outside_note_folder(self, mock_notes_folder):
        assert get_normalised_path("/etc/passwd") is None


# ---------------------------------------------------------------------------
# read_file_content
# ---------------------------------------------------------------------------


class TestReadFileContent:
    """Read note files from tests/mock_notes; mock open() for error paths."""

    def test_reads_existing_note(self):
        content = read_file_content(str(EXAMPLE_MD))

        assert content == EXAMPLE_MD.read_text(encoding="utf-8")

    def test_reads_note_with_front_matter(self):
        content = read_file_content(str(ANOTHER_EXAMPLE_MD))

        assert "phase: 100" in content
        assert "# Silly Database Integration" in content

    def test_missing_file_returns_none(self):
        assert read_file_content(str(MOCK_NOTES_FOLDER / "missing.md")) is None

    def test_directory_path_returns_none(self):
        assert read_file_content(str(MOCK_NOTES_FOLDER / "folder")) is None

    def test_os_error_returns_none(self):
        with patch(
            "src.backend.file_io.open", side_effect=OSError("permission denied")
        ):
            assert read_file_content(str(EXAMPLE_MD)) is None


# ---------------------------------------------------------------------------
# ensure_note_parent_dirs
# ---------------------------------------------------------------------------


class TestEnsureNoteParentDirs:
    """Directory creation with mkdir mocked at the filesystem boundary."""

    def test_root_level_note_needs_no_mkdir(self, mock_notes_folder):
        with patch.object(Path, "mkdir") as mock_mkdir:
            assert (
                ensure_note_parent_dirs(str(mock_notes_folder / "example.md")) is True
            )

        mock_mkdir.assert_not_called()

    def test_nested_path_creates_parent_dirs(self, mock_notes_folder):
        with patch.object(Path, "mkdir") as mock_mkdir:
            assert (
                ensure_note_parent_dirs(
                    str(mock_notes_folder / "folder" / "new" / "note.md")
                )
                is True
            )

        mock_mkdir.assert_called_once_with(parents=True, exist_ok=True)

    def test_rejects_path_outside_note_folder(self, mock_notes_folder):
        with patch.object(Path, "mkdir") as mock_mkdir:
            assert ensure_note_parent_dirs("/etc/passwd") is False

        mock_mkdir.assert_not_called()

    def test_returns_false_when_mkdir_raises(self, mock_notes_folder):
        with patch.object(Path, "mkdir", side_effect=OSError("permission denied")):
            assert (
                ensure_note_parent_dirs(
                    str(mock_notes_folder / "folder" / "new" / "note.md")
                )
                is False
            )


# ---------------------------------------------------------------------------
# write_file_content
# ---------------------------------------------------------------------------


class TestWriteFileContent:
    """File writes mock open() at the file_io module boundary."""

    def test_writes_content_to_normalised_path(self, mock_notes_folder):
        m = mock_open()
        with patch("src.backend.file_io.open", m):
            assert (
                write_file_content(
                    str(mock_notes_folder / "folder" / "new-note.md"),
                    "Hello world",
                )
                is True
            )

        m.assert_called_once_with(
            f"{mock_notes_folder}/folder/new-note.md", "w", encoding="utf-8"
        )
        m().write.assert_called_once_with("Hello world")

    def test_rejects_path_outside_note_folder(self, mock_notes_folder):
        with patch("src.backend.file_io.open", mock_open()) as mock_file:
            assert write_file_content("/etc/passwd", "secret") is False

        mock_file.assert_not_called()

    def test_returns_false_when_open_raises(self, mock_notes_folder):
        with patch("src.backend.file_io.open", side_effect=OSError("disk full")):
            assert (
                write_file_content(str(mock_notes_folder / "new-note.md"), "body")
                is False
            )


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
