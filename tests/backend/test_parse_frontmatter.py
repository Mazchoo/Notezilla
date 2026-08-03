"""Tests for parse_frontmatter YAML → Chroma where conversion."""

import pytest

from src.backend.parse_markdown import parse_frontmatter
from src.field_enums import FieldTypes, ReservedFields


COLUMN_TYPES = {
    ReservedFields.FILENAME: FieldTypes.STRING,
    "status": FieldTypes.STRING,
    "phase": FieldTypes.INT,
    "tags": FieldTypes.LIST,
    "published": FieldTypes.BOOL,
}


class TestParseFrontmatter:
    """parse_frontmatter builds Chroma where filters from YAML strings."""

    def test_empty_string_returns_none(self):
        """Empty or whitespace-only input yields no filter."""
        assert parse_frontmatter("", COLUMN_TYPES) is None
        assert parse_frontmatter("   \n  ", COLUMN_TYPES) is None

    def test_scalar_field_equality(self):
        """A single known scalar field becomes a simple where clause."""
        assert parse_frontmatter("status: todo", COLUMN_TYPES) == {"status": "todo"}

    def test_fenced_frontmatter(self):
        """YAML delimited by --- fences is parsed the same as bare YAML."""
        raw = "---\nstatus: todo\nphase: 2\n---\n"
        assert parse_frontmatter(raw, COLUMN_TYPES) == {
            "$and": [{"status": "todo"}, {"phase": 2}]
        }

    def test_unknown_columns_ignored(self):
        """Columns absent from column_types are dropped from the filter."""
        assert parse_frontmatter(
            "status: todo\nunknown_field: ignore-me", COLUMN_TYPES
        ) == {"status": "todo"}

    def test_only_unknown_columns_returns_none(self):
        """When every key is unknown, no where clause is produced."""
        assert parse_frontmatter("nope: 1\nalso_nope: x", COLUMN_TYPES) is None

    def test_list_field_single_value(self):
        """A one-item list becomes field\\titem: True."""
        assert parse_frontmatter('tags: ["cheese"]', COLUMN_TYPES) == {
            "tags\tcheese": True
        }

    def test_list_field_multiple_values(self):
        """Multiple list items become an $and of field\\titem: True keys."""
        assert parse_frontmatter('tags: ["cheese", "bread"]', COLUMN_TYPES) == {
            "$and": [{"tags\tcheese": True}, {"tags\tbread": True}]
        }

    def test_list_and_scalar_combined(self):
        """List keys and scalar fields share one $and filter."""
        result = parse_frontmatter(
            'status: draft\ntags: ["cheese", "bread"]', COLUMN_TYPES
        )
        assert result == {
            "$and": [
                {"status": "draft"},
                {"tags\tcheese": True},
                {"tags\tbread": True},
            ]
        }

    def test_bool_and_int_types(self):
        """Non-string scalars keep their typed values for equality."""
        assert parse_frontmatter("published: true\nphase: 100", COLUMN_TYPES) == {
            "$and": [{"published": True}, {"phase": 100}]
        }

    def test_malformed_yaml_returns_none(self):
        """Unparseable YAML yields no filter instead of raising."""
        assert parse_frontmatter(": not: valid: [", COLUMN_TYPES) is None

    def test_non_mapping_yaml_returns_none(self):
        """A YAML list or scalar root is not a valid front matter filter."""
        assert parse_frontmatter("- just\n- a\n- list", COLUMN_TYPES) is None
        assert parse_frontmatter("plain string", COLUMN_TYPES) is None

    def test_text_reserved_field_ignored(self):
        """Reserved text field is never used as a metadata where key."""
        assert parse_frontmatter("text: body\nstatus: todo", COLUMN_TYPES) == {
            "status": "todo"
        }

    def test_filename_allowed_when_in_column_types(self):
        """filename is stored in metadata and may be used as a filter."""
        assert parse_frontmatter("filename: note.md", COLUMN_TYPES) == {
            "filename": "note.md"
        }


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
