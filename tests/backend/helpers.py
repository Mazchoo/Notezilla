"""Shared helpers for backend MCP tool tests."""

from contextlib import contextmanager
from pathlib import Path
from typing import Dict, Iterable, Iterator, List, Mapping, Optional

from src.backend.note import NoteData

MOCK_NOTES_FOLDER = (Path(__file__).resolve().parent.parent / "mock_notes").resolve()


def make_notes(docs=None, metas=None) -> List[NoteData]:
    """Build NoteData list for mocked query returns."""
    documents = docs if docs is not None else ["doc1"]
    metadatas = metas if metas is not None else [{"filename": "note.md"}]
    notes: List[NoteData] = []
    for text, meta in zip(documents, metadatas):
        fields = dict(meta)
        filename = str(fields.pop("filename", ""))
        notes.append(NoteData(text=text, filename=filename, fields=fields))
    return notes


def _vault_path(folder: Path, relative: str) -> Path:
    """Resolve a vault-relative path to an absolute Path under *folder*."""
    return folder.joinpath(*Path(relative).parts)


def _remove_empty_parents(path: Path, folder: Path) -> None:
    """Remove *path* and empty ancestors under *folder* (does not remove *folder*)."""
    current = path
    while current != folder and current.exists():
        try:
            current.rmdir()
        except OSError:
            break
        current = current.parent


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
        _remove_empty_parents(note_path.parent, folder)


@contextmanager
def temporary_notes(
    files: Optional[Mapping[str, str]] = None,
    *,
    dirs: Iterable[str] = (),
    folder: Path = MOCK_NOTES_FOLDER,
) -> Iterator[Dict[str, Path]]:
    """
    Create temporary files and directories under the mock notes folder.

    *files* maps vault-relative paths to contents.
    *dirs* lists vault-relative directories to create (including empty ones).
    Yields a mapping of relative path -> absolute Path. On exit, removes any
    remaining created files and then empty parent directories under *folder*.
    """
    created_files: Dict[str, Path] = {}
    created_dirs: Dict[str, Path] = {}
    try:
        for rel in dirs:
            path = _vault_path(folder, rel)
            path.mkdir(parents=True, exist_ok=True)
            created_dirs[rel] = path

        for rel, content in (files or {}).items():
            path = _vault_path(folder, rel)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
            created_files[rel] = path

        yield {**created_dirs, **created_files}
    finally:
        for path in created_files.values():
            if path.is_file():
                path.unlink()

        cleanup_dirs = sorted(
            set(created_dirs.values()) | {p.parent for p in created_files.values()},
            key=lambda p: len(p.parts),
            reverse=True,
        )
        for directory in cleanup_dirs:
            _remove_empty_parents(directory, folder)
