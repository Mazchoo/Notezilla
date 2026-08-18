"""Shared helpers for backend MCP tool tests."""

import os
import shutil
from contextlib import contextmanager
from pathlib import Path
from typing import Dict, FrozenSet, Iterable, Iterator, List, Mapping, Optional

from src.backend.note import NoteData

MOCK_NOTES_FOLDER = Path("./tests/mock_notes").resolve()
MOCK_TEMPLATES_FOLDER = Path("./tests/mock_templates").resolve()

COMMITTED_MOCK_NOTE_FILES: FrozenSet[Path] = frozenset(
    {
        Path("example.md"),
        Path("folder") / "another_example.md",
        Path("folder") / "sub_folder" / ".gitignore",
    }
)
COMMITTED_MOCK_TEMPLATE_FILES: FrozenSet[Path] = frozenset(
    {
        Path("animal_facts.md"),
        Path("folder") / "nested.md",
    }
)


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


def reset_mock_folder(folder: Path, committed_files: FrozenSet[Path]) -> None:
    """Delete files and directories under *folder* that are not in *committed_files*."""
    if not folder.is_dir():
        return
    allowed_dirs = {parent for rel in committed_files for parent in rel.parents}
    allowed_dirs.discard(Path("."))

    for dirpath, _dirnames, filenames in os.walk(folder, topdown=False):
        current = Path(dirpath)
        rel_dir = current.relative_to(folder)
        for name in filenames:
            rel = Path(name) if rel_dir == Path(".") else rel_dir / name
            if rel not in committed_files:
                (current / name).unlink()
        if rel_dir != Path(".") and rel_dir not in allowed_dirs:
            shutil.rmtree(current, ignore_errors=True)


def _path_under_folder(folder: Path, relative_path: str) -> Path:
    """Join *relative_path* onto *folder* as an absolute Path."""
    return folder.joinpath(*Path(relative_path).parts)


def _remove_empty_directories(directories: Iterable[Path], stop_at: Path) -> None:
    """Remove each directory and empty ancestors, stopping at *stop_at*."""
    deepest_first = sorted(directories, key=lambda path: len(path.parts), reverse=True)
    for directory in deepest_first:
        current = directory
        while current != stop_at and current.exists():
            try:
                current.rmdir()
            except OSError:
                break
            current = current.parent


def _create_directories(
    folder: Path, relative_paths: Iterable[str]
) -> Dict[str, Path]:
    created: Dict[str, Path] = {}
    for relative_path in relative_paths:
        directory = _path_under_folder(folder, relative_path)
        directory.mkdir(parents=True, exist_ok=True)
        created[relative_path] = directory
    return created


def _create_files(folder: Path, files: Mapping[str, str]) -> Dict[str, Path]:
    created: Dict[str, Path] = {}
    for relative_path, contents in files.items():
        file_path = _path_under_folder(folder, relative_path)
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.write_text(contents, encoding="utf-8")
        created[relative_path] = file_path
    return created


def _delete_files(created_files: Mapping[str, Path]) -> None:
    for file_path in created_files.values():
        if file_path.is_file():
            file_path.unlink()


def _top_level_name(relative_path: str) -> str:
    return Path(relative_path).parts[0]


@contextmanager
def clean_up_file_if_created(
    file_path: Path, *, folder: Path = MOCK_NOTES_FOLDER
) -> Iterator[Path]:
    """Yield *file_path*, then delete the file and any empty parents under *folder*."""
    try:
        yield file_path
    finally:
        if file_path.is_file():
            file_path.unlink()
        _remove_empty_directories([file_path.parent], folder)


@contextmanager
def temporary_notes(
    files: Optional[Mapping[str, str]] = None,
    *,
    dirs: Iterable[str] = (),
    folder: Path = MOCK_NOTES_FOLDER,
) -> Iterator[Dict[str, Path]]:
    """
    Create temporary files and directories under *folder*.

    *files* maps relative paths to contents.
    *dirs* lists relative directories to create (including empty ones).
    Yields a mapping of relative path -> absolute Path. On exit, deletes
    remaining created files, any newly created top-level entries (including
    rename/move destinations), and then empty parent directories under *folder*.
    """
    relative_paths = list(dirs) + list(files or {})
    top_level_names = {_top_level_name(path) for path in relative_paths}
    preexisting_top_level = {
        name for name in top_level_names if (folder / name).exists()
    }

    created_directories = _create_directories(folder, dirs)
    created_files = _create_files(folder, files or {})
    try:
        yield {**created_directories, **created_files}
    finally:
        _delete_files(created_files)
        for name in top_level_names:
            path = folder / name
            if name in preexisting_top_level:
                continue
            if path.is_file() or path.is_symlink():
                path.unlink()
            elif path.is_dir():
                shutil.rmtree(path)
        _remove_empty_directories(
            set(created_directories.values())
            | {file_path.parent for file_path in created_files.values()},
            folder,
        )
