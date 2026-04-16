"""Handles all database interactions for note storage and retrieval"""

import os
import json
from dataclasses import dataclass
from datetime import datetime, date
from typing import Any, Dict, List, Optional, cast

import chromadb
from chromadb.utils.embedding_functions import SentenceTransformerEmbeddingFunction

from src.config import DATABASE_FOLDER, COLLECTION_NAME, EMBEDDING_MODEL
from src.reserved_fields import ReservedFields
from src.note_updates.file_io import delete_all_old_index_folders


os.environ.setdefault("HF_HUB_OFFLINE", "1")


@dataclass
class QueryResult:
    documents: List[str]
    metadatas: List[Dict[str, Any]]
    distances: Optional[List[float]] = None


class NoteDatabase:
    """Manages a ChromaDB collection for markdown notes"""

    PATH_DEPTH_PREFIX = "\npath_depth_"

    def __init__(self, max_path_depth: int = 0, path: str = DATABASE_FOLDER):
        self._client = chromadb.PersistentClient(path=path)
        self._max_path_depth = max_path_depth
        self._embedding_function = SentenceTransformerEmbeddingFunction(
            model_name=EMBEDDING_MODEL
        )
        self._collection = self._client.get_or_create_collection(
            name=COLLECTION_NAME,
            metadata={"hnsw:space": "cosine"},
            embedding_function=self._embedding_function,  # type: ignore[arg-type]
        )

    @staticmethod
    def cast_value(key: str, val, target_type: str) -> dict:
        """Cast a value to the target type and return as dict entries for a row"""
        if val is None:
            return {key: None}
        if target_type == "json":
            return {key: json.dumps(val, default=str)}
        if target_type == "list":
            parsed_list = val if isinstance(val, list) else [val]
            return {f"{key}\t{item}": True for item in parsed_list}
        if target_type == "date":
            return {
                key: val.isoformat() if isinstance(val, (datetime, date)) else str(val)
            }
        if target_type == "str":
            return {key: str(val)}
        if target_type == "float":
            return {key: float(val)}
        if target_type == "int":
            return {key: int(val)}
        if target_type == "bool":
            return {key: bool(val)}
        return {key: val}

    def upsert_batch(self, rows: List[dict]) -> int:
        """
        Upserts a list of prepared rows into the collection.
        Each row must contain ReservedFields (path, filename, text).
        Returns change in files (+ve)
        """
        if not rows:
            return 0

        ids = []
        documents = []
        metadatas = []

        for row in rows:
            text = row.get(ReservedFields.TEXT.value, "")
            path_parts = row.get(ReservedFields.PATH.value, [])
            filename = row.get(ReservedFields.FILENAME.value, "")

            doc_id = "/".join(path_parts + [filename])
            for i in range(self._max_path_depth):
                key = f"{self.PATH_DEPTH_PREFIX}{i}"
                row[key] = path_parts[i] if i < len(path_parts) else None
            # The path field is ignored as it is already searchable in path depth parts
            metadata = {
                k: v
                for k, v in row.items()
                if v is not None and k != ReservedFields.PATH.value
            }

            ids.append(doc_id)
            documents.append(text)
            metadatas.append(metadata)

        count_before = len(self)
        self._collection.upsert(ids=ids, documents=documents, metadatas=metadatas)
        return len(self) - count_before

    def delete_batch(self, paths: List[str]) -> int:
        """
        Upserts a list of prepared rows into the collection.
        Each row must contain ReservedFields (path, filename, text).
        Returns change in files (-ve)
        """
        if not paths:
            return 0

        count_before = len(self)
        self._collection.delete(ids=paths)
        return len(self) - count_before

    def __len__(self) -> int:
        return self._collection.count()

    def query_by_field(self, field: str, value, n_results: int = 10) -> QueryResult:
        """Return documents and metadatas where a metadata field equals the given value"""
        results = self._collection.get(where={field: value}, limit=n_results)
        return QueryResult(
            documents=results["documents"] or [],
            metadatas=cast(List[Dict[str, Any]], results["metadatas"] or []),
        )

    def query_field_contains(self, field: str, value: str, n_results: int = 10) -> QueryResult:
        """Return documents and metadatas where a list field contains a value (stored as field.value: True)"""
        return self.query_by_field(f"{field}\t{value}", True, n_results)

    def query_by_path(self, path_parts: List[str], n_results: int = 100) -> QueryResult:
        """Return all documents and metadatas under a given folder path"""
        if not path_parts:
            results = self._collection.get(limit=n_results)
        else:
            conditions = [
                {f"{self.PATH_DEPTH_PREFIX}{i}": part}
                for i, part in enumerate(path_parts)
            ]
            where = conditions[0] if len(conditions) == 1 else {"$and": conditions}
            results = self._collection.get(where=where, limit=n_results)  # type: ignore[arg-type]
        return QueryResult(
            documents=results["documents"] or [],
            metadatas=cast(List[Dict[str, Any]], results["metadatas"] or []),
        )

    def query_by_text(
        self, text: str, n_results: int = 10, where: Optional[dict] = None
    ) -> QueryResult:
        """Semantic search — returns documents, metadatas, and distances"""
        results = self._collection.query(
            query_texts=[text],
            n_results=n_results,
            where=where,
        )
        raw_distances = results.get("distances")
        return QueryResult(
            documents=results["documents"][0] if results["documents"] else [],
            metadatas=cast(List[Dict[str, Any]], results["metadatas"][0] if results["metadatas"] else []),
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
