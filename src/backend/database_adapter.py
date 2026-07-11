"""Handles all database interactions for note storage and retrieval"""

import os
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, cast, Union

import chromadb
from chromadb.utils.embedding_functions import SentenceTransformerEmbeddingFunction

from src.config import DATABASE_FOLDER, COLLECTION_NAME, EMBEDDING_MODEL
from src.field_enums import ColumnTypes, ReservedFields
from src.backend.chroma_frontmatter_parsing import decode_frontmatter
from src.backend.file_io import delete_all_old_index_folders
from src.backend.note import NoteData


os.environ.setdefault("HF_HUB_OFFLINE", "1")

VALID_QUERY_TYPES = (str, int, float, bool)


@dataclass
class QueryResult:
    """Documents and metadata returned from a database query."""

    documents: List[str]
    metadatas: List[Dict[str, Any]]
    distances: Optional[List[float]] = None


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

    def query_by_id(self, doc_id: str) -> QueryResult:
        """Return a single document and metadata by its path id"""
        results = self._collection.get(
            ids=[doc_id],
            include=["documents", "metadatas"],
        )
        return QueryResult(
            documents=results["documents"] or [],
            metadatas=cast(List[Dict[str, Any]], results["metadatas"] or []),
        )

    def get_frontmatter_from_path_key(
        self, path_key: str, column_types: ColumnTypes
    ) -> Optional[NoteData]:
        """Load note text and decoded front matter for a path key."""
        result = self.query_by_id(path_key)
        if not result.documents:
            return None

        metadata = result.metadatas[0] if result.metadatas else {}
        return NoteData(
            text=result.documents[0],
            fields=decode_frontmatter(metadata, column_types),
            filename=str(metadata.get(ReservedFields.FILENAME, "")),
        )

    def query_by_field(
        self, field: str, value: Union[str, bool, int, float], n_results: int = 10
    ) -> QueryResult:
        """Return documents and metadatas where a metadata field equals the given value"""
        if not isinstance(value, VALID_QUERY_TYPES):
            raise ValueError(
                f"{value}: {type(value)} not in valid query types {VALID_QUERY_TYPES}"
            )

        results = self._collection.get(
            where={field: value},
            limit=n_results,
            include=["documents", "metadatas"],
        )
        return QueryResult(
            documents=results["documents"] or [],
            metadatas=cast(List[Dict[str, Any]], results["metadatas"] or []),
        )

    def query_field_contains(
        self, field: str, value: str, n_results: int = 10
    ) -> QueryResult:
        """Return documents where a list field contains a value.

        List values are stored as ``field.value: True`` metadata keys.
        """
        return self.query_by_field(f"{field}\t{value}", True, n_results)

    def query_by_text(
        self, text: str, n_results: int = 10, where: Optional[dict] = None
    ) -> QueryResult:
        """Semantic search — returns documents, metadatas, and distances"""
        results = self._collection.query(
            query_texts=[text],
            n_results=n_results,
            where=where,
            include=["documents", "metadatas", "distances"],
        )
        raw_distances = results.get("distances")
        return QueryResult(
            documents=results["documents"][0] if results["documents"] else [],
            metadatas=cast(
                List[Dict[str, Any]],
                results["metadatas"][0] if results["metadatas"] else [],
            ),
            distances=raw_distances[0] if raw_distances else [],
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
