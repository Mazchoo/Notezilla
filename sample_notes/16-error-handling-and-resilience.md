---
phase: 5
title: Error Handling & Resilience
tags: [error-handling, resilience, polish]
status: todo
---

# 16 - Error Handling & Resilience

**Phase 5: Sync & Polish**

Robust handling for when the local LLM is offline or the vector index needs a rebuild.

## Steps

1. Implement a service status dashboard in the GUI showing connection state for: FastMCP API, Ollama, ChromaDB.
2. Add health-check polling that periodically verifies each service is reachable and updates status indicators.
3. Implement graceful degradation: if Ollama is offline, disable generation features but keep editing and search functional.
4. Add a "Rebuild Index" action with a progress indicator for when the vector database needs a full re-index.
5. Implement user-facing error notifications (toast/snackbar) with actionable messages (e.g. "Ollama is offline. Start it with `ollama serve`").
6. Add retry logic with exponential backoff for transient API failures.
7. Log errors to a local log file for debugging.

## Acceptance Criteria

- [ ] A status dashboard shows the connection state of all backend services.
- [ ] The app remains functional for editing and browsing when Ollama is offline (generation features are disabled with a clear message).
- [ ] The app remains functional for editing when ChromaDB is unavailable (search is disabled with a clear message).
- [ ] "Rebuild Index" re-indexes all notes with a visible progress indicator.
- [ ] Transient API failures are retried automatically with backoff before showing an error.
- [ ] Error notifications include actionable guidance (not just "something went wrong").
- [ ] Errors are logged locally for troubleshooting.
