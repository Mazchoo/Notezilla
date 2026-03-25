"""Query notes from the database by metadata fields"""

from typing import Tuple, Any, Optional
from time import perf_counter

from chromadb.api.types import Documents, Metadatas

from src.note_updates.database_adapter import NoteDatabase


LIST_FIELD = "tags"
LIST_VALUE = "core"
SEARCH_FIELD = "filename"
SEARCH_VALUE = "spam.md"
SEARCH_PATH = ["folder"]


def print_query_results(documents: Optional[Documents], metadatas: Optional[Metadatas]):
    """Show the query results"""
    for doc, meta in zip(documents or [], metadatas or []):
        print(f"--- {meta.get('filename', 'unknown')} ---")
        print(f"  metadata: {meta}")
        print(f"  text: {doc[:200]}")
        print()


def search_by_field(
    database: NoteDatabase, field: str, value: Any
) -> Tuple[Documents, Metadatas]:
    """Find notes where a metadata field equals the given value"""
    results = database.query_by_field(field, value)

    documents = results["documents"] or []
    metadatas = results["metadatas"] or []

    return documents, metadatas


def search_in_list_field(
    database: NoteDatabase, field: str, value: str
) -> Tuple[Optional[Documents], Optional[Metadatas]]:
    """Find notes where a list field contains an exact value"""
    results = database.query_field_contains(field, value)

    return results["documents"], results["metadatas"]


def search_sub_path(
    database: NoteDatabase, path_parts: list
) -> Tuple[Optional[Documents], Optional[Metadatas]]:
    """Find all notes under a given folder path"""
    results = database.query_by_path(path_parts)

    return results["documents"], results["metadatas"]


if __name__ == "__main__":
    db = NoteDatabase()

    start = perf_counter()
    docs, metas = search_in_list_field(db, LIST_FIELD, LIST_VALUE)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Search 'in'")
    print_query_results(docs, metas)
    print(f"Time taken: {time_taken_ms:.3}ms")
    print("------")
    print()

    start = perf_counter()
    docs, metas = search_by_field(db, SEARCH_FIELD, SEARCH_VALUE)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Search exact match")
    print_query_results(docs, metas)
    print(f"Time taken: {time_taken_ms:.3}ms")
    print("------")
    print()

    start = perf_counter()
    docs, metas = search_sub_path(db, SEARCH_PATH)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Search by path")
    print_query_results(docs, metas)
    print(f"Time taken: {time_taken_ms:.3}ms")
    print("------")
