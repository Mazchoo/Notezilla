"""Tests for the upsert_template MCP tool."""

from unittest.mock import patch

import pytest

from src.backend.main import upsert_template
from tests.backend.helpers import MOCK_TEMPLATES_FOLDER, clean_up_file_if_created


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


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
