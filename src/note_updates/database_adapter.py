"""Handles all database interactions for note storage and retrieval"""

from typing import List

import chromadb

from src.config import DATABASE_FOLDER, COLLECTION_NAME
from src.reserved_fields import ReservedFields


class NoteDatabase:
    """Manages a ChromaDB collection for markdown notes"""

    def __init__(self, path: str = DATABASE_FOLDER):
        self._client = chromadb.PersistentClient(path=path)
        self._next_id = 0
        self._collection = self._client.get_or_create_collection(
            name=COLLECTION_NAME,
            metadata={"hnsw:space": "cosine"},
        )

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
            metadata = {k: v for k, v in row.items() if v is not None}

            ids.append(str(self._next_id))
            self._next_id += 1
            documents.append(text)
            metadatas.append(metadata)

        self._collection.upsert(ids=ids, documents=documents, metadatas=metadatas)

    def __len__(self) -> int:
        return self._collection.count()

    def reset_collection(self):
        """Drop and recreate the collection"""
        self._client.delete_collection(COLLECTION_NAME)
        self._collection = self._client.get_or_create_collection(
            name=COLLECTION_NAME,
            metadata={"hnsw:space": "cosine"},
        )
