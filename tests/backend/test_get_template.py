"""Tests for template MCP tools."""

import shutil
from unittest.mock import patch

import pytest

from src.backend.main import (
    delete_template,
    delete_template_folder,
    get_template,
    get_template_dir_contents,
    move_template_dir,
    new_template_dir,
    rename_template_dir,
    upsert_template,
)
from tests.backend.helpers import (
    MOCK_TEMPLATES_FOLDER,
    clean_up_file_if_created,
    temporary_notes,
)


class TestGetTemplate:
    """Tests for the get_template MCP tool (filesystem-backed)."""

    def test_returns_matching_document(self, mock_templates_folder):
        """get_template returns a single document read from the template folder."""
        result = get_template(path="animal_facts.md")

        assert result.content[0].text == "Success"
        template = result.structured_content["notes"][0]
        assert template["filename"] == "animal_facts.md"
        assert "## Animal name" in template["text"]
        assert template["metadata"] == {"tags": ["animal_facts"]}

    def test_invalid_path_returns_error(self, mock_templates_folder):
        """get_template returns an error when the path is outside the template folder."""
        result = get_template(path="../../../etc/passwd")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"notes": [], "warnings": []}

    def test_not_found_returns_error(self, mock_templates_folder):
        """get_template returns an error when the template file does not exist."""
        result = get_template(path="missing/template.md")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"notes": [], "warnings": []}

    def test_note_path_is_not_a_template(
        self, mock_notes_folder, mock_templates_folder
    ):
        """A path inside the note folder is not loaded as a template."""
        result = get_template(path=str(mock_notes_folder / "example.md"))

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"notes": [], "warnings": []}


class TestGetTemplateDirContents:
    """Tests for get_template_dir_contents using tests/mock_templates."""

    def test_root_lists_md_files(self, mock_templates_folder):
        """Default path lists immediate .md files at the template root."""
        result = get_template_dir_contents()

        assert result.content[0].text == "Success"
        assert result.structured_content == {
            "folders": [],
            "files": ["animal_facts.md"],
            "warnings": [],
        }

    def test_subdirectory_lists_children(self, mock_templates_folder):
        """A nested path lists only its immediate child folders and .md files."""
        nested = mock_templates_folder / "folder" / "nested.md"
        with clean_up_file_if_created(nested, folder=MOCK_TEMPLATES_FOLDER):
            nested.parent.mkdir(parents=True, exist_ok=True)
            nested.write_text("# Nested template\n", encoding="utf-8")

            result = get_template_dir_contents(path="folder")

            assert result.content[0].text == "Success"
            assert result.structured_content == {
                "folders": [],
                "files": ["nested.md"],
                "warnings": [],
            }

            root = get_template_dir_contents()
            assert "folder" in root.structured_content["folders"]
            assert "animal_facts.md" in root.structured_content["files"]

    def test_invalid_path_outside_template_folder(self, mock_templates_folder):
        """Paths outside the template folder are rejected with an error message."""
        result = get_template_dir_contents(path="../outside")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content["folders"] == []
        assert result.structured_content["files"] == []

    def test_nonexistent_path_returns_error(self, mock_templates_folder):
        """Missing directories return empty lists and a filesystem error message."""
        result = get_template_dir_contents(path="does/not/exist")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content["folders"] == []
        assert result.structured_content["files"] == []


class TestUpsertTemplate:
    """Tests for the upsert_template MCP tool using tests/mock_templates."""

    def test_upsert_template_success(self, mock_templates_folder):
        """upsert_template returns success when the template is written."""
        with clean_up_file_if_created(
            mock_templates_folder / "folder" / "my-template.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ) as template_path:
            result = upsert_template(
                path="folder/my-template.md",
                contents="Hello template",
                fields={"title": "My Template"},
            )

            assert result.content[0].text == "Success"
            assert result.structured_content == {
                "newFileCreated": True,
                "warnings": [],
            }
            assert template_path.is_file()

    def test_upsert_template_existing_file_reports_not_created(
        self, mock_templates_folder
    ):
        """upsert_template reports newFileCreated=False when the file already exists."""
        with clean_up_file_if_created(
            mock_templates_folder / "folder" / "my-template.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ) as template_path:
            upsert_template(
                path="folder/my-template.md",
                contents="First body",
                fields={"title": "My Template"},
            )
            result = upsert_template(
                path="folder/my-template.md",
                contents="Updated body",
                fields={"title": "My Template"},
            )

            assert result.content[0].text == "Success"
            assert result.structured_content == {
                "newFileCreated": False,
                "warnings": [],
            }
            assert template_path.read_text(encoding="utf-8").endswith("Updated body")

    def test_upsert_template_failure_returns_error(self, mock_templates_folder):
        """upsert_template returns an error message when open() raises OSError."""
        with clean_up_file_if_created(
            mock_templates_folder / "folder" / "my-template.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ) as template_path:
            with patch("src.backend.file_io.open", side_effect=OSError("disk full")):
                result = upsert_template(
                    path="folder/my-template.md",
                    contents="Hello template",
                    fields={},
                )

            assert result.content[0].text.startswith("Error")
            assert not template_path.is_file()

    def test_upsert_template_non_md_path_writes_md(self, mock_templates_folder):
        """upsert_template coerces non-.md paths to .md and writes the template."""
        with clean_up_file_if_created(
            mock_templates_folder / "folder" / "note.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ) as template_path:
            result = upsert_template(
                path="folder/note.txt",
                contents="Body text",
                fields={},
            )

            assert result.content[0].text == "Success"
            assert result.structured_content == {
                "newFileCreated": True,
                "warnings": [],
            }
            assert template_path.is_file()
            assert not (mock_templates_folder / "folder" / "note.txt").exists()

    def test_upsert_template_writes_correct_content(self, mock_templates_folder):
        """upsert_template writes the YAML header + body to the template folder."""
        with clean_up_file_if_created(
            mock_templates_folder / "folder" / "my-template.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ) as template_path:
            upsert_template(
                path="folder/my-template.md",
                contents="Hello template",
                fields={"title": "My Template"},
            )

            written = template_path.read_text(encoding="utf-8")
            assert "Hello template" in written
            assert "My Template" in written

    def test_upsert_template_does_not_write_to_note_folder(
        self, mock_notes_folder, mock_templates_folder
    ):
        """upsert_template writes under the template folder, not the note folder."""
        with clean_up_file_if_created(
            mock_templates_folder / "isolated.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ):
            result = upsert_template(
                path="isolated.md",
                contents="Template only",
                fields={},
            )

            assert result.content[0].text == "Success"
            assert (mock_templates_folder / "isolated.md").is_file()
            assert not (mock_notes_folder / "isolated.md").exists()

    def test_get_template_reads_latest_content_after_write(
        self, mock_templates_folder
    ):
        """Save then get_template must return the overwritten file contents."""
        with clean_up_file_if_created(
            mock_templates_folder / "overwrite-me.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ) as template_path:
            first = upsert_template(
                path="overwrite-me.md",
                contents="original body",
                fields={"title": "Original"},
            )
            assert first.content[0].text == "Success"

            second = upsert_template(
                path="overwrite-me.md",
                contents="updated body",
                fields={"title": "Updated"},
            )
            assert second.content[0].text == "Success"
            assert second.structured_content["newFileCreated"] is False

            result = get_template(path="overwrite-me.md")

            assert result.content[0].text == "Success"
            template = result.structured_content["notes"][0]
            assert template["text"].strip() == "updated body"
            assert template["metadata"]["title"] == "Updated"
            assert template_path.exists()


class TestDeleteTemplate:
    """Tests for the delete_template MCP tool using tests/mock_templates."""

    def test_delete_template_success(self, mock_templates_folder):
        """delete_template returns success and removes the file from the template folder."""
        with clean_up_file_if_created(
            mock_templates_folder / "tmp_del.md",
            folder=MOCK_TEMPLATES_FOLDER,
        ) as template_path:
            template_path.write_text("# temp\n", encoding="utf-8")
            result = delete_template(path="tmp_del.md")

            assert result.content[0].text == "Success"
            assert result.structured_content == {"warnings": []}
            assert not template_path.is_file()

    def test_delete_template_missing_file_returns_error(self, mock_templates_folder):
        """delete_template returns an error when the template file does not exist."""
        result = delete_template(path="missing/template.md")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"warnings": []}

    def test_delete_template_invalid_path_returns_error(self, mock_templates_folder):
        """delete_template returns an error when the path is outside the template folder."""
        result = delete_template(path="../../../etc/passwd")

        assert result.content[0].text.startswith("Error")
        assert result.structured_content == {"warnings": []}

    def test_delete_template_does_not_delete_note(
        self, mock_notes_folder, mock_templates_folder
    ):
        """A path inside the note folder is not deleted as a template."""
        note_path = mock_notes_folder / "example.md"
        assert note_path.is_file()

        result = delete_template(path=str(note_path))

        assert result.content[0].text.startswith("Error")
        assert note_path.is_file()


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
        target = mock_templates_folder / "tmp_create" / "deep" / "nested"
        try:
            result = new_template_dir(path="tmp_create/deep/nested")

            assert result.content[0].text == "Success"
            assert target.is_dir()
        finally:
            created = mock_templates_folder / "tmp_create"
            if created.exists():
                shutil.rmtree(created)

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
