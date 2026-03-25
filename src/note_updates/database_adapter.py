"""Handles all database interactions for note storage and retrieval"""

import json
from datetime import datetime, date
from typing import List, Optional

import chromadb
from chromadb.utils.embedding_functions import SentenceTransformerEmbeddingFunction

from src.config import DATABASE_FOLDER, COLLECTION_NAME
from src.reserved_fields import ReservedFields
from src.note_updates.file_io import delete_all_old_index_folders


class NoteDatabase:
    """Manages a ChromaDB collection for markdown notes"""

    PATH_DEPTH_PREFIX = "\npath_depth_"
    EMBEDDING_MODEL = "all-MiniLM-L6-v2"

    def __init__(self, max_path_depth: int = 0, path: str = DATABASE_FOLDER):
        self._client = chromadb.PersistentClient(path=path)
        self._next_id = 0
        self._max_path_depth = max_path_depth
        self._embedding_function = SentenceTransformerEmbeddingFunction(
            model_name=self.EMBEDDING_MODEL
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

    def upsert_batch(self, rows: List[dict]):
        """
        Upserts a list of prepared rows into the collection.
        Each row must contain ReservedFields (path, filename, text).
        """
        if not rows:
            return

        ids = []
        documents = []
        metadatas = []

        for row in rows:
            text = row.pop(ReservedFields.TEXT.value, "")
            path_parts = row.pop(ReservedFields.PATH.value, [])
            for i in range(self._max_path_depth):
                key = f"{self.PATH_DEPTH_PREFIX}{i}"
                row[key] = path_parts[i] if i < len(path_parts) else None
            metadata = {k: v for k, v in row.items() if v is not None}

            ids.append(str(self._next_id))
            self._next_id += 1
            documents.append(text)
            metadatas.append(metadata)

        self._collection.upsert(ids=ids, documents=documents, metadatas=metadatas)

    def __len__(self) -> int:
        return self._collection.count()

    def query_by_field(self, field: str, value, n_results: int = 10):
        """Return documents where a metadata field equals the given value"""
        return self._collection.get(
            where={field: value},
            limit=n_results,
        )

    def query_field_contains(self, field: str, value: str, n_results: int = 10):
        """Return documents where a list field contains a value (stored as field.value: True)"""
        return self.query_by_field(f"{field}\t{value}", True, n_results)

    def query_by_path(self, path_parts: List[str], n_results: int = 100):
        """Return all documents under a given folder path"""
        if not path_parts:
            return self._collection.get(limit=n_results)

        conditions = [
            {f"{self.PATH_DEPTH_PREFIX}{i}": part} for i, part in enumerate(path_parts)
        ]
        where = conditions[0] if len(conditions) == 1 else {"$and": conditions}
        return self._collection.get(where=where, limit=n_results)  # type: ignore[arg-type]

    def query_by_text(
        self, text: str, n_results: int = 10, where: Optional[dict] = None
    ):
        """Semantic search over document text, with optional metadata filter"""
        return self._collection.query(
            query_texts=[text],
            n_results=n_results,
            where=where,
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
        self._next_id = 0
