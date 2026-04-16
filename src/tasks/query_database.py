"""Query notes from the database by metadata fields"""

from typing import List
from time import perf_counter

from src.note_updates.database_adapter import NoteDatabase


LIST_FIELD = "tags"
LIST_VALUE = "paragraph"
SEARCH_FIELD = "filename"
SEARCH_VALUE = "p0.md"
SEARCH_PATH = ["2018", "01", "14"]
SEARCH_TEXT = "I like crepes"


def print_query_results(documents: List, metadatas: List):
    """Show the query results"""
    for doc, meta in zip(documents, metadatas):
        print(f"--- {meta.get('filename', 'unknown')} ---")
        print(f"  metadata: {meta}")
        print(f"  text: {doc[:200]}")
        print()


if __name__ == "__main__":
    db = NoteDatabase()

    start = perf_counter()
    docs, metas = db.query_field_contains(LIST_FIELD, LIST_VALUE, 5)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Search by using 'in list' query method for tags")
    print_query_results(docs, metas)
    print(f"Time taken: {time_taken_ms:.3}ms")
    print("------")
    print()

    start = perf_counter()
    docs, metas = db.query_by_field(SEARCH_FIELD, SEARCH_VALUE, 5)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Search exact match for specific field")
    print_query_results(docs, metas)
    print(f"Time taken: {time_taken_ms:.3}ms")
    print("------")
    print()

    start = perf_counter()
    docs, metas = db.query_by_path(SEARCH_PATH, 5)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Search by file path")
    print_query_results(docs, metas)
    print(f"Time taken: {time_taken_ms:.3}ms")
    print("------")
    print()

    start = perf_counter()
    docs, metas, _ = db.query_by_text(SEARCH_TEXT, 5)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Semantic search by text")
    print_query_results(docs, metas)
    print(f"Time taken: {time_taken_ms:.3}ms")
    print("------")
