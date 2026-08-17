"""Tests for template MCP tools."""

from unittest.mock import patch

import pytest

from src.backend.main import (
    delete_template,
    get_template,
    get_template_dir_contents,
    upsert_template,
)
from tests.backend.helpers import MOCK_TEMPLATES_FOLDER, clean_up_file_if_created


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


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
