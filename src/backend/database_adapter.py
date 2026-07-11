"""Handles all database interactions for note storage and retrieval"""

import os
import json
from dataclasses import dataclass
from datetime import datetime, date
from typing import Any, Dict, List, Optional, cast, Union

import chromadb
from chromadb.utils.embedding_functions import SentenceTransformerEmbeddingFunction

from src.config import DATABASE_FOLDER, COLLECTION_NAME, EMBEDDING_MODEL
from src.field_enums import ColumnTypes, ReservedFields, FieldTypes
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

    @staticmethod
    def cast_value(key: str, val, target_type: FieldTypes) -> dict:  # pylint: disable=too-many-return-statements
        """Cast a value to the target type and return as dict entries for a row"""
        if val is None:
            return {key: None}
        if target_type == FieldTypes.JSON:
            return {key: json.dumps(val, default=str)}
        if target_type == FieldTypes.LIST:
            parsed_list = val if isinstance(val, list) else [val]
            if not parsed_list:
                return {f"{key}\t": False}
            return {f"{key}\t{item}": True for item in parsed_list}
        if target_type == FieldTypes.DATE:
            return {
                key: val.isoformat() if isinstance(val, (datetime, date)) else str(val)
            }
        if target_type == FieldTypes.STRING:
            return {key: str(val)}
        if target_type == FieldTypes.FLOAT:
            return {key: float(val)}
        if target_type == FieldTypes.INT:
            return {key: int(val)}
        if target_type == FieldTypes.BOOL:
            return {key: bool(val)}
        return {key: val}

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
            fields=self._decode_frontmatter(metadata, column_types),
            filename=str(metadata.get(ReservedFields.FILENAME, "")),
        )

    @staticmethod
    def _decode_frontmatter(
        metadata: Dict[str, Any], column_types: ColumnTypes
    ) -> Dict[str, Any]:
        """Turn stored Chroma metadata keys back into a front matter dict."""
        fields: Dict[str, Any] = {}
        list_items: Dict[str, list[str]] = {}

        for key, val in metadata.items():
            if key.startswith("\n") or ReservedFields.contains(key):
                continue
            if "\t" in key:
                field, item = key.split("\t", 1)
                if val is True:
                    list_items.setdefault(field, []).append(item)
                elif val is False:
                    list_items.setdefault(field, [])
                continue

            field_type = column_types.get(key)
            if field_type in (FieldTypes.JSON, FieldTypes.JSON.value):
                fields[key] = json.loads(val) if isinstance(val, str) else val
            else:
                fields[key] = val

        for field, items in list_items.items():
            fields[field] = sorted(items)

        return fields

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
