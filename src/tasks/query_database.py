"""Query notes from the database by metadata fields"""

from typing import List
from time import perf_counter

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_db_column_types
from src.backend.note import NoteData


LIST_FIELD = "tags"
LIST_VALUE = "paragraph"
SEARCH_FIELD = "filename"
SEARCH_VALUE = "p0.md"
SEARCH_TEXT = "I like crepes"


def print_query_results(notes: List[NoteData]):
    """Show the query results"""
    for note in notes:
        print(f"--- {note.filename or 'unknown'} ---")
        print(f"  metadata: {note.fields}")
        print(f"  text: {note.text[:200]}")
        print()


if __name__ == "__main__":
    db = NoteDatabase()
    column_types = get_db_column_types()

    start = perf_counter()
    result = db.query_field_contains(LIST_FIELD, LIST_VALUE, column_types, 5)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Search by using 'in list' query method for tags")
    print_query_results(result)
    print(f"Time taken: {time_taken_ms:.1f}ms")
    print("------")
    print()

    start = perf_counter()
    result = db.query_by_field(SEARCH_FIELD, SEARCH_VALUE, column_types, 5)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Search exact match for specific field")
    print_query_results(result)
    print(f"Time taken: {time_taken_ms:.1f}ms")
    print("------")
    print()

    start = perf_counter()
    result = db.query_by_text(SEARCH_TEXT, column_types, 5)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Semantic search by text")
    print_query_results(result)
    print(f"Time taken: {time_taken_ms:.1f}ms")
    print("------")
