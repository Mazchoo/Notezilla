"""Handles all database interactions for note storage and retrieval"""

import os
from typing import Any, Dict, List, Optional, cast, Union

import chromadb
from chromadb.utils.embedding_functions import SentenceTransformerEmbeddingFunction

from src.config import DATABASE_FOLDER, COLLECTION_NAME, EMBEDDING_MODEL
from src.field_enums import ColumnTypes, ReservedFields
from src.backend.chroma_parsing import notes_from_chroma
from src.backend.file_io import delete_all_old_index_folders
from src.backend.note import NoteData


os.environ.setdefault("HF_HUB_OFFLINE", "1")

VALID_QUERY_TYPES = (str, int, float, bool)


class NoteDatabase:
    """Manages a ChromaDB collection for markdown notes"""

    def __init__(self, path: str = DATABASE_FOLDER):
        self._client = chromadb.PersistentClient(path=path)
        self._embedding_function = SentenceTransformerEmbeddingFunction(
            model_name=EMBEDDING_MODEL
        )
        self._embedding_function(["warmup"])
        self._collection = self._client.get_or_create_collection(
            name=COLLECTION_NAME,
            metadata={"hnsw:space": "cosine"},
            embedding_function=self._embedding_function,  # type: ignore[arg-type]
        )

    def upsert_batch(self, rows: List[dict]):
        """
        Upserts a list of prepared rows into the collection.
        Each row must contain ReservedFields (path, filename, text).
        Note body is stored as the Chroma document, not in metadata.
        """
        if not rows:
            return 0

        ids = []
        documents = []
        metadatas = []

        for row in rows:
            document = row.get(ReservedFields.TEXT, "")
            doc_id = row.get(ReservedFields.FILENAME, "")
            metadata = {
                k: v
                for k, v in row.items()
                if v is not None and k not in ReservedFields.excluded_from_metadata()
            }

            ids.append(doc_id)
            documents.append(document)
            metadatas.append(metadata)

        self._collection.upsert(ids=ids, documents=documents, metadatas=metadatas)

    def delete_batch(self, paths: List[str]):
        """Delete notes by their path ids."""
        if not paths:
            return 0

        self._collection.delete(ids=paths)

    def __len__(self) -> int:
        return self._collection.count()

    def query_by_id(self, doc_id: str, column_types: ColumnTypes) -> List[NoteData]:
        """Return a single note by its path id"""
        results = self._collection.get(
            ids=[doc_id],
            include=["documents", "metadatas"],
        )
        return notes_from_chroma(
            results["documents"] or [],
            cast(List[Dict[str, Any]], results["metadatas"] or []),
            column_types,
        )

    def get_note_from_path_key(
        self, path_key: str, column_types: ColumnTypes
    ) -> Optional[NoteData]:
        """Load note text and decoded front matter for a path key."""
        notes = self.query_by_id(path_key, column_types)
        return notes[0] if notes else None

    def query_by_field(
        self,
        field: str,
        value: Union[str, bool, int, float],
        column_types: ColumnTypes,
        n_results: int = 10,
        offset: int = 0,
    ) -> List[NoteData]:
        """Return notes where a metadata field equals the given value.

        Pagination: ``offset`` skips that many matches before applying
        ``n_results`` (e.g. offset=10, n_results=10 yields results[10:20]).
        """
        if not isinstance(value, VALID_QUERY_TYPES):
            raise ValueError(
                f"{value}: {type(value)} not in valid query types {VALID_QUERY_TYPES}"
            )

        results = self._collection.get(
            where={field: value},
            limit=n_results,
            offset=offset,
            include=["documents", "metadatas"],
        )
        return notes_from_chroma(
            results["documents"] or [],
            cast(List[Dict[str, Any]], results["metadatas"] or []),
            column_types,
        )

    def query_field_contains(
        self,
        field: str,
        value: str,
        column_types: ColumnTypes,
        n_results: int = 10,
        offset: int = 0,
    ) -> List[NoteData]:
        """Return notes where a list field contains a value.

        List values are stored as ``field.value: True`` metadata keys.
        Pagination: ``offset`` skips that many matches before applying
        ``n_results`` (e.g. offset=10, n_results=10 yields results[10:20]).
        """
        return self.query_by_field(
            f"{field}\t{value}", True, column_types, n_results, offset
        )

    @staticmethod
    def _filter_by_path_prefix(
        documents: List[str],
        metadatas: List[Dict[str, Any]],
        path_filter: str,
    ) -> tuple[List[str], List[Dict[str, Any]]]:
        """Keep only documents whose filename metadata starts with path_filter."""
        filtered_docs: List[str] = []
        filtered_metas: List[Dict[str, Any]] = []
        for doc, meta in zip(documents, metadatas):
            filename = str(meta.get(ReservedFields.FILENAME, ""))
            if filename.startswith(path_filter):
                filtered_docs.append(doc)
                filtered_metas.append(meta)
        return filtered_docs, filtered_metas

    def query_by_text(
        self,
        text: str,
        column_types: ColumnTypes,
        n_results: int = 10,
        where: Optional[dict] = None,
        offset: int = 0,
        path_filter: Optional[str] = None,
    ) -> List[NoteData]:
        """Semantic search — returns matching notes.

        Pagination: ``offset`` skips that many ranked matches before applying
        ``n_results`` (e.g. offset=10, n_results=10 yields results[10:20]).

        When ``path_filter`` is non-empty, only notes whose filenames start
        with that prefix are kept (applied before pagination).
        """
        fetch_count = n_results + offset
        if path_filter:
            fetch_count = max(fetch_count, len(self))
        if fetch_count < 1:
            return []

        results = self._collection.query(
            query_texts=[text],
            n_results=fetch_count,
            where=where,
            include=["documents", "metadatas"],
        )
        documents = results["documents"][0] if results["documents"] else []
        metadatas = results["metadatas"][0] if results["metadatas"] else []

        if path_filter:
            documents, metadatas = self._filter_by_path_prefix(
                documents, cast(List[Dict[str, Any]], metadatas), path_filter
            )

        end = offset + n_results
        return notes_from_chroma(
            documents[offset:end],
            cast(List[Dict[str, Any]], metadatas[offset:end]),
            column_types,
        )

    def reset_collection(self):
        """Drop collection, remove stale index folders, and recreate"""
        self._client.delete_collection(COLLECTION_NAME)
        delete_all_old_index_folders()

        self._collection = self._client.get_or_create_collection(
            name=COLLECTION_NAME,
            metadata={"hnsw:space": "cosine"},
            embedding_function=self._embedding_function,  # type: ignore[arg-type]
        )
