---
phase: 3
title: Search Interface
tags: [gui, blazor, search, vector]
status: todo
---

# 10 - Search Interface

**Phase 3: Blazor GUI Development**

Build the UI for both keyword and semantic (vector) search queries.

## Steps

1. Add a search bar component to the top of the application layout.
2. Implement a keyword search mode that filters notes by title, tags, or content substring via the API.
3. Implement a semantic search mode that sends the query to the vector search resource and displays ranked results.
4. Build a search results panel showing matched notes with title, path, relevance score, and a content snippet.
5. Add a toggle or tab to switch between keyword and semantic search modes.
6. Implement click-to-open on search results to navigate to the note in the editor.
7. Add search history or recent searches for quick re-access.

## Acceptance Criteria

- [ ] A search bar is accessible from all views in the application.
- [ ] Keyword search returns notes matching by title, tag, or content.
- [ ] Semantic search returns notes ranked by vector similarity with visible relevance scores.
- [ ] Results display note title, path, a content preview snippet, and matching tags.
- [ ] Clicking a result opens the note in the editor.
- [ ] The user can switch between keyword and semantic search modes.
- [ ] Empty or no-match states display a helpful message.
