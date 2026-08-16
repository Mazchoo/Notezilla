"""Query notes from the database by text"""

from typing import List
from time import perf_counter

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_db_column_types
from src.backend.note import NoteData


SEARCH_TEXT = "I like crepes"


def print_query_results(notes: List[NoteData]):
    """Show the query results"""
    for note in notes:
        print(repr(note))
        print()


if __name__ == "__main__":
    db = NoteDatabase()
    column_types = get_db_column_types()

    start = perf_counter()
    result = db.query_by_text(SEARCH_TEXT, column_types, 5)
    time_taken_ms = (perf_counter() - start) * 1000.0

    print("Semantic search by text")
    print_query_results(result)
    print(f"Time taken: {time_taken_ms:.1f}ms")
    print("------")
