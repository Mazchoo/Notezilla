"""RAG lookup: retrieve relevant notes from ChromaDB"""

from time import perf_counter
from dataclasses import dataclass
from typing import Optional

from src.note_updates.database_adapter import NoteDatabase


@dataclass
class RAGResult:
    """A single retrieved note with its relevance distance."""

    filename: str
    text: str
    metadata: dict
    distance: float


def rag_query(
    text: str,
    n_results: int,
    where: Optional[dict] = None,
) -> list[RAGResult]:
    """
    Retrieve the most relevant notes for a query string.

    Uses the configured sentence-transformer model to encode the query,
    then performs a cosine-similarity search against the ChromaDB collection.
    """
    db = NoteDatabase()
    results = db.query_by_text(text, n_results=n_results, where=where)
    distances = results.get("distances")

    documents = results["documents"][0] if results["documents"] else []
    metadatas = results["metadatas"][0] if results["metadatas"] else []
    distances = distances[0] if distances else []

    return [
        RAGResult(
            filename=str(meta.get("filename", "unknown")),
            text=doc,
            metadata=dict(meta),
            distance=dist,
        )
        for doc, meta, dist in zip(documents, metadatas, distances)
    ]


if __name__ == "__main__":
    SEARCH_TEXT = "Eastern tent caterpillar"
    NR_RESULTS = 5

    start = perf_counter()
    query_results = rag_query(SEARCH_TEXT, n_results=NR_RESULTS)
    time_taken = perf_counter() - start

    for result in query_results:
        date = dict(result.metadata).get("date")
        print(f"--- {result.filename} {date} (distance: {result.distance:.4f}) ---")
        print(f"  {result.text[:200]}")
        print()

    print(f"Time taken {time_taken:.1f}s")
