"""Tests for src.backend.file_io core helpers."""

from pathlib import Path
import shutil
from unittest.mock import mock_open, patch

import pytest

from src.backend.file_io import (
    create_new_folder,
    delete_notes_folder,
    ensure_md_extension,
    ensure_note_parent_dirs,
    extract_yaml_from_file_contents,
    get_normalised_path,
    move_file_or_folder,
    read_file_content,
    split_path_filters,
    rename_basename,
    write_file_content,
)
from src.backend.resolved_folders import ResolvedFolder
from tests.backend.helpers import MOCK_NOTES_FOLDER, temporary_notes


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
# split_path_filters
# ---------------------------------------------------------------------------


class TestSplitPathFilters:
    """Split a comma-separated path filter and trim surrounding spaces."""

    def test_none_returns_empty_list(self):
        """None input yields an empty list."""
        assert split_path_filters(None) == []

    def test_blank_returns_empty_list(self):
        """Blank input yields an empty list."""
        assert split_path_filters("") == []

    def test_single_path_unchanged(self):
        """A single path with no commas is returned as a one-item list."""
        assert split_path_filters("2026/02") == ["2026/02"]

    def test_splits_on_commas(self):
        """Comma-separated paths are returned as separate items."""
        assert split_path_filters("2026/02,folder") == ["2026/02", "folder"]

    def test_trims_spaces_around_paths(self):
        """Leading and trailing spaces on each path are removed."""
        assert split_path_filters(" 2026/02 , folder/sub ") == [
            "2026/02",
            "folder/sub",
        ]

    def test_omits_empty_segments(self):
        """Empty segments after trimming are omitted."""
        assert split_path_filters("keep/,, drop/") == ["keep/", "drop/"]

    def test_only_commas_and_spaces_returns_empty_list(self):
        """A filter of only commas and spaces yields an empty list."""
        assert split_path_filters("  ,  , ") == []


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

    def test_relative_path_resolved_against_note_folder(self, mock_notes_folder):
        assert get_normalised_path("folder/another_example.md") == (
            "folder/another_example.md"
        )

    def test_relative_escape_rejected(self, mock_notes_folder):
        assert get_normalised_path("../../../etc/passwd") is None

    def test_empty_path_is_note_folder_root(self, mock_notes_folder):
        assert get_normalised_path("") == ""

    def test_applied_twice_equals_once(self, mock_notes_folder):
        paths = [
            str(mock_notes_folder / "example.md"),
            str(mock_notes_folder / "folder" / "another_example.md"),
            str(mock_notes_folder / "folder."),
            str(mock_notes_folder / "folder*"),
            "folder/another_example.md",
            "",
        ]
        for path in paths:
            once = get_normalised_path(path)
            assert once is not None
            assert get_normalised_path(once) == once

    def test_relative_path_resolved_against_template_folder(
        self, mock_templates_folder
    ):
        assert (
            get_normalised_path("animal_facts.md", ResolvedFolder.TEMPLATES)
            == "animal_facts.md"
        )

    def test_absolute_path_inside_template_folder(self, mock_templates_folder):
        assert (
            get_normalised_path(
                str(mock_templates_folder / "animal_facts.md"),
                ResolvedFolder.TEMPLATES,
            )
            == "animal_facts.md"
        )

    def test_note_path_rejected_for_template_folder(
        self, mock_notes_folder, mock_templates_folder
    ):
        assert (
            get_normalised_path(
                str(mock_notes_folder / "example.md"), ResolvedFolder.TEMPLATES
            )
            is None
        )


# ---------------------------------------------------------------------------
# ensure_md_extension
# ---------------------------------------------------------------------------


class TestEnsureMdExtension:
    """Coerce path strings to end with lowercase .md without touching folder dots."""

    def test_leaves_md_unchanged(self):
        assert ensure_md_extension("folder/note.md") == "folder/note.md"

    def test_replaces_other_extension(self):
        assert ensure_md_extension("folder/note.txt") == "folder/note.md"

    def test_normalises_uppercase_md(self):
        assert ensure_md_extension("folder/note.MD") == "folder/note.md"

    def test_appends_when_no_extension(self):
        assert ensure_md_extension("folder/note") == "folder/note.md"

    def test_ignores_dots_in_parent_folders(self):
        assert ensure_md_extension("folder.v2/archive.2024/note") == (
            "folder.v2/archive.2024/note.md"
        )

    def test_replaces_extension_with_dotted_parents(self):
        assert ensure_md_extension("folder.v2/note.txt") == "folder.v2/note.md"

    def test_handles_backslash_separators(self):
        assert ensure_md_extension(r"folder.v2\note") == r"folder.v2\note.md"


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
            str(mock_notes_folder / "folder" / "new-note.md"),
            "w",
            encoding="utf-8",
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


# ---------------------------------------------------------------------------
# create_new_folder
# ---------------------------------------------------------------------------


class TestCreateNewFolder:
    """Create a folder within mock_notes."""

    def test_creates_folder(self, mock_notes_folder):
        with temporary_notes(dirs=["tmp_create"]) as paths:
            target = paths["tmp_create"] / "new_folder"
            assert create_new_folder(str(target)) is True
            assert target.is_dir()
            target.rmdir()

    def test_creates_nested_folder_with_parents(self, mock_notes_folder):
        target = mock_notes_folder / "tmp_create" / "deep" / "nested"
        try:
            assert create_new_folder(str(target)) is True
            assert target.is_dir()
        finally:
            if target.exists():
                shutil.rmtree(mock_notes_folder / "tmp_create")

    def test_creates_folder_with_relative_path(self, mock_notes_folder):
        """MCP tools pass note-folder-relative paths, not absolute filesystem paths."""
        target = mock_notes_folder / "tmp_create" / "rel_folder"
        try:
            assert create_new_folder("tmp_create/rel_folder") is True
            assert target.is_dir()
        finally:
            if target.exists():
                shutil.rmtree(mock_notes_folder / "tmp_create")

    def test_rejects_path_outside_note_folder(self, mock_notes_folder):
        assert create_new_folder("/etc/passwd") is False

    def test_rejects_note_folder_root(self, mock_notes_folder):
        assert create_new_folder(str(mock_notes_folder)) is False
        assert create_new_folder("") is False

    def test_rejects_when_folder_already_exists(self, mock_notes_folder):
        with temporary_notes(dirs=["tmp_create/existing"]) as paths:
            assert create_new_folder(str(paths["tmp_create/existing"])) is False

    def test_rejects_when_path_is_existing_file(self, mock_notes_folder):
        with temporary_notes({"tmp_create/solo.md": "x"}) as paths:
            assert create_new_folder(str(paths["tmp_create/solo.md"])) is False
            assert paths["tmp_create/solo.md"].is_file()

    def test_returns_false_when_mkdir_raises(self, mock_notes_folder):
        with temporary_notes(dirs=["tmp_create"]) as paths:
            with patch.object(Path, "mkdir", side_effect=OSError("permission denied")):
                assert create_new_folder(str(paths["tmp_create"] / "blocked")) is False
            assert not (paths["tmp_create"] / "blocked").exists()


# ---------------------------------------------------------------------------
# delete_notes_folder
# ---------------------------------------------------------------------------


class TestDeleteNotesFolder:
    """Delete a folder tree within mock_notes."""

    def test_deletes_folder_recursively(self, mock_notes_folder):
        with temporary_notes(
            {
                "tmp_del/a/one.md": "one",
                "tmp_del/a/b/two.md": "two",
                "tmp_del/a/keep.txt": "txt",
            }
        ) as paths:
            target = mock_notes_folder / "tmp_del" / "a"
            assert delete_notes_folder(str(target)) is True

            assert not target.exists()
            assert not paths["tmp_del/a/one.md"].exists()
            assert not paths["tmp_del/a/b/two.md"].exists()
            assert not paths["tmp_del/a/keep.txt"].exists()

    def test_does_not_delete_outside_target_folder(self, mock_notes_folder):
        with temporary_notes(
            {
                "tmp_del/keep.md": "keep",
                "tmp_del/target/gone.md": "gone",
            }
        ) as paths:
            target = mock_notes_folder / "tmp_del" / "target"
            assert delete_notes_folder(str(target)) is True

            assert paths["tmp_del/keep.md"].is_file()
            assert not target.exists()

    def test_empty_folder_succeeds(self, mock_notes_folder):
        with temporary_notes(dirs=["tmp_del/empty"]) as paths:
            target = paths["tmp_del/empty"]
            assert delete_notes_folder(str(target)) is True
            assert not target.exists()

    def test_rejects_path_outside_note_folder(self, mock_notes_folder):
        assert delete_notes_folder("/etc/passwd") is False

    def test_returns_false_when_path_is_file(self, mock_notes_folder):
        with temporary_notes({"tmp_del/solo.md": "x"}) as paths:
            assert delete_notes_folder(str(paths["tmp_del/solo.md"])) is False
            assert paths["tmp_del/solo.md"].is_file()

    def test_returns_false_for_missing_folder(self, mock_notes_folder):
        assert (
            delete_notes_folder(str(mock_notes_folder / "tmp_del" / "missing")) is False
        )

    def test_returns_false_when_rmtree_raises(self, mock_notes_folder):
        with temporary_notes({"tmp_del/folder/note.md": "x"}):
            with patch(
                "src.backend.file_io.shutil.rmtree",
                side_effect=OSError("permission denied"),
            ):
                assert (
                    delete_notes_folder(str(mock_notes_folder / "tmp_del" / "folder"))
                    is False
                )


# ---------------------------------------------------------------------------
# move_file_or_folder
# ---------------------------------------------------------------------------


class TestMoveFileOrFolder:
    """Move a file or directory into a destination folder within mock_notes."""

    def test_moves_file_into_dst_folder(self, mock_notes_folder):
        with temporary_notes(
            {"tmp_move/src/note.md": "hello"},
            dirs=["tmp_move/dst"],
        ) as paths:
            src = paths["tmp_move/src/note.md"]
            dst = paths["tmp_move/dst"]
            assert move_file_or_folder(str(src), str(dst)) is True

            moved = dst / "note.md"
            assert not src.exists()
            assert moved.is_file()
            assert moved.read_text(encoding="utf-8") == "hello"
            moved.unlink()

    def test_moves_file_with_relative_paths(self, mock_notes_folder):
        """MCP tools pass vault-relative paths, not absolute filesystem paths."""
        with temporary_notes(
            {"tmp_move/src/rel-note.md": "hello"},
            dirs=["tmp_move/dst"],
        ) as paths:
            assert (
                move_file_or_folder("tmp_move/src/rel-note.md", "tmp_move/dst") is True
            )

            src = paths["tmp_move/src/rel-note.md"]
            moved = paths["tmp_move/dst"] / "rel-note.md"
            assert not src.exists()
            assert moved.is_file()
            assert moved.read_text(encoding="utf-8") == "hello"
            moved.unlink()

    def test_moves_directory_into_dst_folder(self, mock_notes_folder):
        with temporary_notes(
            {
                "tmp_move/src/nested/a.md": "a",
                "tmp_move/src/nested/b.md": "b",
            },
            dirs=["tmp_move/dst"],
        ) as paths:
            src = mock_notes_folder / "tmp_move" / "src" / "nested"
            dst = paths["tmp_move/dst"]
            assert move_file_or_folder(str(src), str(dst)) is True

            moved = dst / "nested"
            assert not src.exists()
            assert (moved / "a.md").read_text(encoding="utf-8") == "a"
            assert (moved / "b.md").read_text(encoding="utf-8") == "b"
            shutil.rmtree(moved)

    def test_moves_into_note_folder_root(self, mock_notes_folder):
        with temporary_notes({"tmp_move/src/root-move.md": "root"}) as paths:
            src = paths["tmp_move/src/root-move.md"]
            assert move_file_or_folder(str(src), str(mock_notes_folder)) is True

            moved = mock_notes_folder / "root-move.md"
            assert not src.exists()
            assert moved.is_file()
            assert moved.read_text(encoding="utf-8") == "root"
            moved.unlink()

    def test_rejects_src_outside_note_folder(self, mock_notes_folder):
        with temporary_notes(dirs=["tmp_move/dst"]) as paths:
            assert (
                move_file_or_folder("/etc/passwd", str(paths["tmp_move/dst"])) is False
            )

    def test_rejects_dst_outside_note_folder(self, mock_notes_folder):
        with temporary_notes({"tmp_move/src/note.md": "x"}) as paths:
            assert (
                move_file_or_folder(str(paths["tmp_move/src/note.md"]), "/tmp") is False
            )
            assert paths["tmp_move/src/note.md"].is_file()

    def test_rejects_src_note_folder_root(self, mock_notes_folder):
        with temporary_notes(dirs=["tmp_move/dst"]) as paths:
            assert (
                move_file_or_folder(str(mock_notes_folder), str(paths["tmp_move/dst"]))
                is False
            )

    def test_rejects_dst_when_not_a_directory(self, mock_notes_folder):
        with temporary_notes(
            {
                "tmp_move/src/note.md": "x",
                "tmp_move/not-a-dir.md": "y",
            }
        ) as paths:
            assert (
                move_file_or_folder(
                    str(paths["tmp_move/src/note.md"]),
                    str(paths["tmp_move/not-a-dir.md"]),
                )
                is False
            )
            assert paths["tmp_move/src/note.md"].is_file()

    def test_rejects_missing_src(self, mock_notes_folder):
        with temporary_notes(dirs=["tmp_move/dst"]) as paths:
            assert (
                move_file_or_folder(
                    str(mock_notes_folder / "tmp_move" / "missing.md"),
                    str(paths["tmp_move/dst"]),
                )
                is False
            )

    def test_rejects_missing_dst_folder(self, mock_notes_folder):
        with temporary_notes({"tmp_move/src/note.md": "x"}) as paths:
            assert (
                move_file_or_folder(
                    str(paths["tmp_move/src/note.md"]),
                    str(mock_notes_folder / "tmp_move" / "missing_dst"),
                )
                is False
            )
            assert paths["tmp_move/src/note.md"].is_file()

    def test_rejects_moving_directory_into_itself(self, mock_notes_folder):
        with temporary_notes(
            {"tmp_move/src/nested/note.md": "x"},
            dirs=["tmp_move/src/nested/child"],
        ) as paths:
            src = mock_notes_folder / "tmp_move" / "src" / "nested"
            dst = paths["tmp_move/src/nested/child"]
            assert move_file_or_folder(str(src), str(dst)) is False
            assert src.is_dir()
            assert (src / "note.md").is_file()

    def test_returns_false_when_move_raises(self, mock_notes_folder):
        with temporary_notes(
            {"tmp_move/src/note.md": "x"},
            dirs=["tmp_move/dst"],
        ) as paths:
            with patch(
                "src.backend.file_io.shutil.move",
                side_effect=OSError("permission denied"),
            ):
                assert (
                    move_file_or_folder(
                        str(paths["tmp_move/src/note.md"]), str(paths["tmp_move/dst"])
                    )
                    is False
                )
            assert paths["tmp_move/src/note.md"].is_file()


# ---------------------------------------------------------------------------
# rename_basename
# ---------------------------------------------------------------------------


class TestRenameBasename:
    """Rename a file or directory within mock_notes."""

    def test_renames_file(self, mock_notes_folder):
        with temporary_notes({"tmp_rename/old-name.md": "hello"}) as paths:
            src = paths["tmp_rename/old-name.md"]
            assert rename_basename(str(src), "new-name.md") is True

            dst = mock_notes_folder / "tmp_rename" / "new-name.md"
            assert not src.exists()
            assert dst.is_file()
            assert dst.read_text(encoding="utf-8") == "hello"
            dst.unlink()

    def test_renames_file_with_relative_path(self, mock_notes_folder):
        """MCP tools pass note-folder-relative paths, not absolute filesystem paths."""
        with temporary_notes({"tmp_rename/rel-old.md": "hello"}) as paths:
            assert rename_basename("tmp_rename/rel-old.md", "rel-new.md") is True

            src = paths["tmp_rename/rel-old.md"]
            dst = mock_notes_folder / "tmp_rename" / "rel-new.md"
            assert not src.exists()
            assert dst.is_file()
            assert dst.read_text(encoding="utf-8") == "hello"
            dst.unlink()

    def test_preserves_file_extension_when_new_name_omits_it(self, mock_notes_folder):
        """Renaming my-note.md to 'some-note' must yield some-note.md."""
        with temporary_notes({"tmp_rename/my-note.md": "hello"}) as paths:
            assert rename_basename("tmp_rename/my-note.md", "some-note") is True

            src = paths["tmp_rename/my-note.md"]
            dst = mock_notes_folder / "tmp_rename" / "some-note.md"
            assert not src.exists()
            assert not (mock_notes_folder / "tmp_rename" / "some-note").exists()
            assert dst.is_file()
            assert dst.read_text(encoding="utf-8") == "hello"
            dst.unlink()

    def test_renames_directory(self, mock_notes_folder):
        with temporary_notes(
            {
                "tmp_rename/old-dir/a.md": "a",
                "tmp_rename/old-dir/b.md": "b",
            }
        ):
            src = mock_notes_folder / "tmp_rename" / "old-dir"
            assert rename_basename(str(src), "new-dir") is True

            dst = mock_notes_folder / "tmp_rename" / "new-dir"
            assert not src.exists()
            assert (dst / "a.md").read_text(encoding="utf-8") == "a"
            assert (dst / "b.md").read_text(encoding="utf-8") == "b"
            shutil.rmtree(dst)

    def test_rejects_path_outside_note_folder(self, mock_notes_folder):
        assert rename_basename("/etc/passwd", "renamed") is False

    def test_rejects_note_folder_root(self, mock_notes_folder):
        assert rename_basename(str(mock_notes_folder), "renamed") is False

    def test_rejects_missing_path(self, mock_notes_folder):
        assert (
            rename_basename(
                str(mock_notes_folder / "tmp_rename" / "missing.md"), "new.md"
            )
            is False
        )

    def test_rejects_new_name_outside_note_folder(self, mock_notes_folder):
        with temporary_notes({"tmp_rename/note.md": "x"}) as paths:
            assert (
                rename_basename(str(paths["tmp_rename/note.md"]), "../../../etc/passwd")
                is False
            )
            assert paths["tmp_rename/note.md"].is_file()

    def test_rejects_when_sibling_already_exists(self, mock_notes_folder):
        with temporary_notes(
            {
                "tmp_rename/src.md": "src",
                "tmp_rename/dst.md": "dst",
            }
        ) as paths:
            assert rename_basename(str(paths["tmp_rename/src.md"]), "dst.md") is False
            assert paths["tmp_rename/src.md"].is_file()
            assert paths["tmp_rename/dst.md"].read_text(encoding="utf-8") == "dst"

    def test_returns_false_when_rename_raises(self, mock_notes_folder):
        with temporary_notes({"tmp_rename/src/note.md": "x"}) as paths:
            with patch.object(
                Path,
                "rename",
                side_effect=OSError("permission denied"),
            ):
                assert (
                    rename_basename(str(paths["tmp_rename/src/note.md"]), "renamed.md")
                    is False
                )
            assert paths["tmp_rename/src/note.md"].is_file()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
