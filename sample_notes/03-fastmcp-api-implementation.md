---
phase: 1
title: FastMCP API Implementation
tags: [core, fastmcp, api, python]
status: todo
---

# 03 - FastMCP API Implementation

**Phase 1: Core Python Library & FastMCP API**

Build the Python API using FastMCP to expose CRUD operations and search capabilities.

## Steps (ToDo add capability to add new yaml field to database)

1. Set up a FastMCP project with the appropriate server configuration.
2. Implement the `notes` tool to list and retrieve notes with optional filtering by path or tags.
3. Implement the `update_note` tool to create or update a markdown file and trigger re-indexing.
4. Implement the `delete_note` tool to remove a note file and its vector database entry.
5. Implement a semantic search resource endpoint that accepts a query and returns matching notes.
6. Implement a metadata-based filtering resource endpoint that allows searching by tags and paths.
7. Add input validation and error responses for all endpoints.
8. Write integration tests covering all tools and resources.

## Acceptance Criteria

- [x] `notes` tool returns a list of notes with metadata, supporting optional tag and path filters.
- [x] `update_note` tool creates a new `.md` file or updates an existing one with correct frontmatter.
- [x] `delete_note` tool removes the file from disk and from the vector database.
- [x] Semantic search resource returns ranked results for a given query string.
- [x] Metadata filtering resource supports filtering by tags (any/all match) and path prefix.
- [ ] All endpoints return meaningful error messages for invalid inputs.
- [ ] The API is usable by MCP-compatible clients (e.g. Claude Desktop).
- [ ] Add tests that mock the file directory events
