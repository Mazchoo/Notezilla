---
phase: 1
title: Vector Database Integration
tags: [core, chromadb, vector, python]
status: todo
---

# 02 - Vector Database Integration

**Phase 1: Core Python Library & FastMCP API**

Set up a local vector database (ChromaDB) to index the text content of notes for semantic search.

## Steps

1. Add ChromaDB as a project dependency and configure a persistent local storage path.
2. Define a ChromaDB collection schema that stores note content embeddings alongside metadata (path, tags, title).
3. Implement an indexing function that takes parsed note data and upserts it into the collection.
4. Implement a deletion function that removes entries by note identifier when a file is deleted.
5. Build a semantic search function that accepts a query string and returns ranked results with relevance scores.
6. Wire the file watcher events (from story 01) to the index/delete functions so the database stays in sync.
7. Write tests to verify indexing, searching, and deletion workflows.

## Acceptance Criteria

- [x] ChromaDB runs locally with persistent storage so data survives process restarts.
- [x] Notes are embedded and stored with their metadata (path, tags) in the collection.
- [ ] Updating a note re-indexes it without creating duplicates.
- [ ] Deleting a note removes it from the vector database.
- [x] Semantic search returns relevant notes ranked by similarity score.
- [x] The database can be fully rebuilt from the filesystem notes directory on demand.
