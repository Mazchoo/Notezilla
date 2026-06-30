"""Shared helpers for backend MCP tool tests."""

from src.backend.database_adapter import QueryResult


def _make_query_result(docs=None, metas=None) -> QueryResult:
    return QueryResult(
        documents=docs if docs is not None else ["doc1"],
        metadatas=metas if metas is not None else [{"filename": "note.md"}],
    )
