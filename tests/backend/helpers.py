"""Shared helpers for backend MCP tool tests."""

from contextlib import contextmanager
from pathlib import Path
from typing import Iterator

from src.backend.database_adapter import QueryResult

MOCK_NOTES_FOLDER = (Path(__file__).resolve().parent.parent / "mock_notes").resolve()


def _make_query_result(docs=None, metas=None) -> QueryResult:
    return QueryResult(
        documents=docs if docs is not None else ["doc1"],
        metadatas=metas if metas is not None else [{"filename": "note.md"}],
    )


@contextmanager
def clean_up_file_if_created(
    note_path: Path, *, folder: Path = MOCK_NOTES_FOLDER
) -> Iterator[Path]:
    """Yield *note_path*, then remove the file and any empty parents under *folder*."""
    try:
        yield note_path
    finally:
        if note_path.is_file():
            note_path.unlink()
        parent = note_path.parent
        while parent != folder and parent.exists():
            try:
                parent.rmdir()
            except OSError:
                break
            parent = parent.parent
