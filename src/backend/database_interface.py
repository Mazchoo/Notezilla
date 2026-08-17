"""Abstract interface for note database storage and retrieval."""

from abc import ABC, abstractmethod
from typing import List, Optional

from src.backend.note import NoteData
from src.field_enums import ColumnTypes


class INoteDatabase(ABC):
    """Public interface for a note database."""

    @abstractmethod
    def upsert_batch(self, rows: List[dict]):
        """
        Upserts a list of prepared rows into the collection.
        Each row must contain ReservedFields (path, filename, text).
        Note body is stored as the Chroma document, not in metadata.
        """
        raise NotImplementedError

    @abstractmethod
    def delete_batch(self, paths: List[str]):
        """Delete notes by their path ids."""
        raise NotImplementedError

    @abstractmethod
    def __len__(self) -> int:
        """Return the number of notes in the collection."""
        raise NotImplementedError

    @abstractmethod
    def query_by_id(self, doc_id: str, column_types: ColumnTypes) -> List[NoteData]:
        """Return a single note by its path id"""
        raise NotImplementedError

    @abstractmethod
    def get_note_from_path_key(
        self, path_key: str, column_types: ColumnTypes
    ) -> Optional[NoteData]:
        """Load note text and decoded front matter for a path key."""
        raise NotImplementedError

    @abstractmethod
    def query_by_text(
        self,
        text: str,
        column_types: ColumnTypes,
        n_results: int = 10,
        where: Optional[dict] = None,
        offset: int = 0,
        path_filter: Optional[List[str]] = None,
    ) -> List[NoteData]:
        """Return notes matching filters, with optional semantic ranking.

        Pagination: ``offset`` skips that many matches before applying
        ``n_results`` (e.g. offset=10, n_results=10 yields results[10:20]).

        When ``path_filter`` is non-empty, the index is restricted to ids
        whose filenames start with any listed prefix before the similarity
        query and pagination.

        When ``text`` is blank, notes are returned without a similarity
        query. Path and metadata filters still apply.
        """
        raise NotImplementedError

    @abstractmethod
    def reset_collection(self):
        """Drop collection, remove stale index folders, and recreate"""
        raise NotImplementedError
