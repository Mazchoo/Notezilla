---
phase: 5
title: Bi-directional Sync
tags: [sync, vector, gui, polish]
status: todo
---

# 13 - Bi-directional Sync

**Phase 5: Sync & Polish**

Ensure that manual edits in the Leptos GUI trigger immediate updates in the Vector DB via the FastMCP service.

## Steps

1. Implement a save hook in the Leptos editor that calls the `update_note` API on every save, triggering re-indexing.
2. Add a file watcher notification channel (e.g. WebSocket or SSE) from the FastMCP service to the GUI so external file changes are reflected live.
3. Add a database checking window that checks that notes are synced up between the database and what is on disk.

## Acceptance Criteria

- [x] Saving a note in the GUI immediately updates the vector database entry.
- [x] External file changes (outside the GUI) are detected and reflected in the GUI within a few seconds.
- [ ] Conflict between database and files can be requested and reported back as a markdown
- [ ] "Resync All" rebuilds the entire vector index from the filesystem.
- [ ] The file tree and search results update without requiring a manual page refresh.
- [ ] No data loss occurs during concurrent edits from multiple sources.
