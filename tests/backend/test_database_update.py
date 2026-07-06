"""Tests for prepate_database_row"""

import pytest

from src.backend.database_update import prepate_database_row
from src.backend.parse_markdown import MarkdownData
from src.field_enums import FieldTypes, ReservedFields
from src.tasks.restore_missing_files import metadata_to_fields


def test_prepate_database_row_only_includes_present_frontmatter_fields():
    """Schema columns absent from the file must not be injected as None."""
    markdown = MarkdownData(
        fields={"title": "Example"},
        text="Body",
        filename="note.md",
        path=["example_folder"],
    )
    column_types = {
        "filename": FieldTypes.STRING,
        "date": FieldTypes.DATE,
        "title": FieldTypes.STRING,
    }

    row = prepate_database_row(markdown, column_types)

    assert "date" not in row
    assert row["title"] == "Example"


def test_empty_tags_are_lost_in_database_row():
    """LIST fields encode as tab keys; an empty list produces no metadata."""
    markdown = MarkdownData(
        fields={"tags": []},
        text="Body",
        filename="note.md",
        path=["example_folder"],
    )
    column_types = {"tags": FieldTypes.LIST}

    row = prepate_database_row(markdown, column_types)
    metadata = {
        k: v
        for k, v in row.items()
        if k not in ReservedFields.excluded_from_metadata()
        and k not in (ReservedFields.FILENAME,)
    }

    assert metadata == {}
    assert metadata_to_fields(metadata, column_types) == {}


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
