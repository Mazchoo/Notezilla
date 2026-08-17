"""Tests for parse_frontmatter YAML → Chroma where conversion."""

from typing import Any, Optional

import pytest

from src.backend.parse_markdown import (
    clean_path_filter,
    normalise_filter,
    parse_frontmatter,
)
from src.field_enums import ColumnTypes, FieldTypes, ReservedFields


COLUMN_TYPES = {
    ReservedFields.FILENAME: FieldTypes.STRING,
    "status": FieldTypes.STRING,
    "phase": FieldTypes.INT,
    "tags": FieldTypes.LIST,
    "published": FieldTypes.BOOL,
}


def _parse(
    raw: str, column_types: ColumnTypes | None = None
) -> tuple[Optional[dict[str, Any]], list[str]]:
    warnings: list[str] = []
    result = parse_frontmatter(raw, column_types or COLUMN_TYPES, warnings)
    return result, warnings


class TestParseFrontmatter:
    """parse_frontmatter builds Chroma where filters from YAML strings."""

    def test_empty_string_returns_none(self):
        """Empty input yields no filter."""
        result, warnings = _parse("")
        assert result is None
        assert warnings == []

    def test_whitespace_only_returns_none(self):
        """Whitespace-only input yields no filter."""
        result, warnings = _parse("   \n  ")
        assert result is None
        assert warnings == []

    def test_scalar_field_equality(self):
        """A single known scalar field becomes a simple where clause."""
        result, warnings = _parse("status: todo")
        assert result == {"status": "todo"}
        assert warnings == []

    def test_fenced_frontmatter(self):
        """YAML delimited by --- fences is parsed the same as bare YAML."""
        result, warnings = _parse("---\nstatus: todo\nphase: 2\n---\n")
        assert result == {"$and": [{"status": "todo"}, {"phase": 2}]}
        assert warnings == []

    def test_unknown_columns_ignored(self):
        """Columns absent from column_types are dropped and warned about."""
        result, warnings = _parse("status: todo\nunknown_field: ignore-me")
        assert result == {"status": "todo"}
        assert warnings

    def test_only_unknown_columns_returns_none(self):
        """When every key is unknown, no where clause is produced."""
        result, warnings = _parse("nope: 1\nalso_nope: x")
        assert result is None
        assert len(warnings) == 2

    def test_list_field_single_value(self):
        """A one-item list becomes field\\titem: True."""
        result, warnings = _parse('tags: ["cheese"]')
        assert result == {"tags\tcheese": True}
        assert warnings == []

    def test_list_field_multiple_values(self):
        """Multiple list items become an $and of field\\titem: True keys."""
        result, warnings = _parse('tags: ["cheese", "bread"]')
        assert result == {"$and": [{"tags\tcheese": True}, {"tags\tbread": True}]}
        assert warnings == []

    def test_list_and_scalar_combined(self):
        """List keys and scalar fields share one $and filter."""
        result, warnings = _parse('status: draft\ntags: ["cheese", "bread"]')
        assert result == {
            "$and": [
                {"status": "draft"},
                {"tags\tcheese": True},
                {"tags\tbread": True},
            ]
        }
        assert warnings == []

    def test_bool_and_int_types(self):
        """Non-string scalars keep their typed values for equality."""
        result, warnings = _parse("published: true\nphase: 100")
        assert result == {"$and": [{"published": True}, {"phase": 100}]}
        assert warnings == []

    def test_malformed_yaml_returns_none(self):
        """Unparseable YAML yields no filter and a warning instead of raising."""
        result, warnings = _parse(": not: valid: [")
        assert result is None
        assert warnings

    def test_yaml_list_root_returns_none(self):
        """A YAML list root is not a valid front matter filter."""
        result, warnings = _parse("- just\n- a\n- list")
        assert result is None
        assert warnings

    def test_yaml_scalar_root_returns_none(self):
        """A YAML scalar root is not a valid front matter filter."""
        result, warnings = _parse("plain string")
        assert result is None
        assert warnings

    def test_non_mapping_semicolon_yaml_returns_none(self):
        """A semicolon scalar is not a YAML mapping of filter fields."""
        result, warnings = _parse("togs; []")
        assert result is None
        assert warnings

    def test_text_reserved_field_ignored(self):
        """Reserved text field is never used as a metadata where key."""
        result, warnings = _parse("text: body\nstatus: todo")
        assert result == {"status": "todo"}
        assert warnings

    def test_empty_field_value_omitted(self):
        """A null field value is omitted from the where clause and warned about."""
        result, warnings = _parse("status:")
        assert result is None
        assert warnings

    def test_empty_field_value_omitted_keeps_other_conditions(self):
        """A null field is dropped while remaining known fields still filter."""
        result, warnings = _parse("status:\nphase: 2")
        assert result == {"phase": 2}
        assert warnings

    def test_filename_allowed_when_in_column_types(self):
        """filename is stored in metadata and may be used as a filter."""
        result, warnings = _parse("filename: note.md")
        assert result == {"filename": "note.md"}
        assert warnings == []


def _clean(path_filter: Optional[str]) -> tuple[list[str], list[str]]:
    warnings: list[str] = []
    result = clean_path_filter(path_filter, warnings)
    return result, warnings


class TestNormaliseFilter:
    """Strip and normalise path prefix filters for filename startswith checks."""

    def test_converts_backslashes_to_forward_slashes(self):
        """Backslashes become forward slashes."""
        assert normalise_filter("folder\\sub\\note.md") == "folder/sub/note.md"

    def test_removes_trailing_star(self):
        """A trailing glob star is stripped."""
        assert normalise_filter("folder/sub*") == "folder/sub"

    def test_removes_leading_dot_slash(self):
        """A leading ./ prefix is removed."""
        assert normalise_filter("./folder/sub") == "folder/sub"

    def test_removes_leading_slash(self):
        """A leading / prefix is removed."""
        assert normalise_filter("/folder/sub") == "folder/sub"

    def test_combined_cleanup(self):
        """Backslashes, trailing *, and leading ./ are all normalised."""
        assert normalise_filter(".\\folder\\sub*") == "folder/sub"

    def test_blank_returns_empty(self):
        """Blank input yields an empty string."""
        assert normalise_filter("") == ""

    def test_none_returns_empty(self):
        """None input yields an empty string."""
        assert normalise_filter(None) == ""

    def test_unchanged_relative_prefix(self):
        """A clean relative prefix is returned unchanged."""
        assert normalise_filter("2026/02") == "2026/02"


class TestCleanPathFilter:
    """clean_path_filter splits, normalises, and warns on empty segments."""

    def test_none_returns_empty_without_warning(self):
        """None input yields an empty list and no warning."""
        result, warnings = _clean(None)
        assert result == []
        assert warnings == []

    def test_blank_returns_empty_without_warning(self):
        """Blank input yields an empty list and no warning."""
        result, warnings = _clean("")
        assert result == []
        assert warnings == []

    def test_single_path_normalised(self):
        """A single path prefix is normalised with no warning."""
        result, warnings = _clean(".\\2026\\02*")
        assert result == ["2026/02"]
        assert warnings == []

    def test_comma_separated_paths(self):
        """Comma-separated prefixes are each normalised with no warning."""
        result, warnings = _clean("2026/02,folder")
        assert result == ["2026/02", "folder"]
        assert warnings == []

    def test_empty_segment_omitted_with_warning(self):
        """Empty comma segments are omitted and produce a warning."""
        result, warnings = _clean("keep/,, drop/")
        assert result == ["keep/", "drop/"]
        assert warnings

    def test_only_empty_segments_return_empty_with_warning(self):
        """A filter of only empty segments yields [] and a warning."""
        result, warnings = _clean("  ,  , ")
        assert result == []
        assert warnings

    def test_normalises_to_empty_with_warning(self):
        """A segment that normalises to empty is omitted and warned about."""
        result, warnings = _clean("*")
        assert result == []
        assert warnings

    def test_mixed_empty_segment_omitted_with_warning(self):
        """A segment that normalises to empty is dropped; usable prefixes remain."""
        result, warnings = _clean("folder, ./")
        assert result == ["folder"]
        assert warnings


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
