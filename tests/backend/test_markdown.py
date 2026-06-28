"""Tests for MarkdownData construction and parsing"""

from pathlib import Path
from unittest.mock import patch

import pytest

from src.backend.parse_markdown import MarkdownData


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _fake_write_factory(store: dict):
    """Return a write_file_content replacement that saves payload into *store*."""

    def _write(path: str, contents: str) -> bool:
        store[path] = contents
        return True

    return _write


# ---------------------------------------------------------------------------
# Round-trip tests
# ---------------------------------------------------------------------------


class TestMarkdownDataRoundTrip:
    """Verify that construct_from_data and construct_from_path are inverses.

    File I/O is fully mocked: write_file_content stores the payload in a dict
    and read_file_content returns it from that same dict.  Path existence checks
    are patched so no real filesystem is touched.
    """

    def _run_round_trip(self, path: str, contents: str, fields: dict) -> tuple:
        """
        Execute the full round-trip and return (from_data, from_path).

        1. construct_from_data  - write_file_content is replaced by a dict store.
        2. construct_from_path  - read_file_content returns the stored payload;
           Path predicates are patched to make the path appear valid.
        """
        store = {}

        # --- Phase 1: construct from data ---
        with (
            patch(
                "src.backend.parse_markdown.ensure_note_parent_dirs",
                return_value=True,
            ),
            patch(
                "src.backend.parse_markdown.write_file_content",
                side_effect=_fake_write_factory(store),
            ),
        ):
            from_data = MarkdownData.construct_from_data(
                path=path, contents=contents, fields=fields
            )

        assert from_data is not None, "construct_from_data returned None unexpectedly"
        assert store, "write_file_content was never called"

        # The key written may differ from *path* (write_file_content normalises it),
        # so grab whatever was stored.
        written_payload = next(iter(store.values()))

        # --- Phase 2: construct from path ---
        with (
            patch(
                "src.backend.parse_markdown.read_file_content",
                return_value=written_payload,
            ),
            patch.object(Path, "exists", return_value=True),
            patch.object(Path, "is_dir", return_value=False),
            patch.object(Path, "is_relative_to", return_value=True),
            patch.object(
                Path,
                "relative_to",
                return_value=Path(*Path(path).parts),
            ),
        ):
            from_path = MarkdownData.construct_from_path(path=path)

        assert from_path is not None, "construct_from_path returned None unexpectedly"

        return from_data, from_path

    # ------------------------------------------------------------------
    # Individual test cases
    # ------------------------------------------------------------------

    def test_round_trip_fields_match(self):
        """Fields survive a write → read cycle unchanged."""
        from_data, from_path = self._run_round_trip(
            path="2024/01/my-note.md",
            contents="Hello world",
            fields={"title": "My Note", "tags": ["python", "testing"]},
        )

        assert from_data.fields == from_path.fields
        assert from_data.text.strip() == from_path.text.strip()
        assert from_data.filename == from_path.filename

    def test_round_trip_empty_fields(self):
        """A note with no YAML front-matter round-trips correctly."""
        from_data, from_path = self._run_round_trip(
            path="2024/01/no-fields.md",
            contents="Just plain text",
            fields={},
        )

        assert from_data.fields == from_path.fields
        assert from_data.text.strip() == from_path.text.strip()
        assert from_data.filename == from_path.filename

    def test_round_trip_multiline_content(self):
        """Multi-line body text is preserved through the round-trip."""
        body = "Line one\nLine two\n\nLine four after blank"
        from_data, from_path = self._run_round_trip(
            path="2024/06/multi.md",
            contents=body,
            fields={"title": "Multi"},
        )

        assert from_data.text.strip() == from_path.text.strip()

    def test_round_trip_does_not_touch_real_filesystem(self):
        """Neither construct_from_data nor construct_from_path calls real open()."""
        store = {}

        with patch("builtins.open") as mock_open:
            with (
                patch(
                    "src.backend.parse_markdown.ensure_note_parent_dirs",
                    return_value=True,
                ),
                patch(
                    "src.backend.parse_markdown.write_file_content",
                    side_effect=_fake_write_factory(store),
                ),
            ):
                MarkdownData.construct_from_data(
                    path="2024/01/safe.md",
                    contents="text",
                    fields={"title": "Safe"},
                )

            written_payload = next(iter(store.values()))

            with (
                patch(
                    "src.backend.parse_markdown.read_file_content",
                    return_value=written_payload,
                ),
                patch.object(Path, "exists", return_value=True),
                patch.object(Path, "is_dir", return_value=False),
                patch.object(Path, "is_relative_to", return_value=True),
                patch.object(Path, "relative_to", return_value=Path("2024/01/safe.md")),
            ):
                MarkdownData.construct_from_path(path="2024/01/safe.md")

            mock_open.assert_not_called()


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
