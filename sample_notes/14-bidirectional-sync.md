---
phase: 5
title: Bi-directional Sync
tags: [sync, vector, gui, polish]
status: todo
---

# 14 - Bi-directional Sync

**Phase 5: Sync & Polish**

Ensure that manual edits in the Blazor GUI trigger immediate updates in the Vector DB via the FastMCP service.

## Steps

1. Implement a save hook in the Blazor editor that calls the `update_note` API on every save, triggering re-indexing.
2. Add a file watcher notification channel (e.g. WebSocket or SSE) from the FastMCP service to the GUI so external file changes are reflected live.
3. Implement conflict detection: if a note is edited in both the GUI and the filesystem simultaneously, alert the user with a diff view.
4. Add a manual "Resync All" action that triggers a full re-index of the filesystem into the vector database.
5. Implement optimistic UI updates so the tree and search results reflect changes before the API confirms.
6. Write end-to-end tests covering edit-in-GUI-reflects-in-DB and edit-on-disk-reflects-in-GUI scenarios.

## Acceptance Criteria

- [ ] Saving a note in the GUI immediately updates the vector database entry.
- [ ] External file changes (outside the GUI) are detected and reflected in the GUI within a few seconds.
- [ ] Conflicting edits are detected and the user is shown both versions with an option to resolve.
- [ ] "Resync All" rebuilds the entire vector index from the filesystem.
- [ ] The file tree and search results update without requiring a manual page refresh.
- [ ] No data loss occurs during concurrent edits from multiple sources.
