"""Handles all database interactions for note storage and retrieval"""

import os
from typing import Any, Dict, List, Optional, cast

import chromadb
from chromadb.utils.embedding_functions import SentenceTransformerEmbeddingFunction

from src.config import (
    BATCH_SIZE,
    COLLECTION_NAME,
    DATABASE_FOLDER,
    EMBEDDING_MODEL,
    MAX_DB_ITERATION,
)
from src.field_enums import ColumnTypes, ReservedFields
from src.backend.chroma_parsing import notes_from_chroma
from src.backend.database_interface import INoteDatabase
from src.backend.file_io import delete_all_old_index_folders
from src.backend.note import NoteData


os.environ.setdefault("HF_HUB_OFFLINE", "1")


class NoteDatabase(INoteDatabase):
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

    def _ids_matching_path_prefix(self, path_filters: List[str]) -> List[str]:
        """Return collection ids whose path starts with any listed prefix.

        IDs are the note filenames. Scans the index in BATCH_SIZE pages so a
        single get() does not load the full id list. Stops after
        MAX_ID_SCAN_BATCHES pages.
        """
        if not path_filters:
            return []

        matching: List[str] = []
        for batch_index in range(MAX_DB_ITERATION):
            batch = self._collection.get(
                include=[],
                limit=BATCH_SIZE,
                offset=batch_index * BATCH_SIZE,
            )
            ids = batch.get("ids") or []
            if not ids:
                break
            matching.extend(
                doc_id
                for doc_id in ids
                if any(doc_id.startswith(prefix) for prefix in path_filters)
            )
            if len(ids) < BATCH_SIZE:
                break
        return matching

    def _search_with_no_text_query(
        self,
        column_types: ColumnTypes,
        n_results: int,
        offset: int,
        where: Optional[dict],
        query_ids: Optional[List[str]],
    ) -> List[NoteData]:
        """Return notes by metadata and path filters, without a similarity query."""
        get_kwargs: Dict[str, Any] = {
            "limit": n_results,
            "offset": offset,
            "include": ["documents", "metadatas"],
        }
        if where is not None:
            get_kwargs["where"] = where
        if query_ids is not None:
            get_kwargs["ids"] = query_ids
        results = self._collection.get(**get_kwargs)
        return notes_from_chroma(
            results["documents"] or [],
            cast(List[Dict[str, Any]], results["metadatas"] or []),
            column_types,
        )

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
        fetch_count = n_results + offset
        if fetch_count < 1:
            return []

        query_ids: Optional[List[str]] = None
        if path_filter:
            query_ids = self._ids_matching_path_prefix(path_filter)
            if not query_ids:
                return []
            fetch_count = min(fetch_count, len(query_ids))

        if not text.strip():
            return self._search_with_no_text_query(
                column_types, n_results, offset, where, query_ids
            )

        query_kwargs: Dict[str, Any] = {
            "query_texts": [text],
            "n_results": fetch_count,
            "where": where,
            "include": ["documents", "metadatas"],
        }
        if query_ids is not None:
            query_kwargs["ids"] = query_ids

        results = self._collection.query(**query_kwargs)
        documents = results["documents"][0] if results["documents"] else []
        metadatas = results["metadatas"][0] if results["metadatas"] else []

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
