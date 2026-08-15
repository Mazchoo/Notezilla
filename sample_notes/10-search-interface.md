---
phase: 3
title: Search Interface
tags: [gui, leptos, search, vector]
status: todo
---

# 10 - Search Interface

**Phase 3: Leptos GUI Development**

Build the UI for both keyword and semantic (vector) search queries. Searching is done with single text query and can be augmented with filters.

## Steps

1. Add a search bar component to the top of the application layout.
2. Implement a keyword search mode that filters notes by title, tags, or content substring via the API.
3. Implement a semantic search mode that sends the query to the vector search resource and displays ranked results.
4. Build a search results panel showing matched notes with title, path, relevance score, and a content snippet.
5. Implement click-to-open on search results to navigate to the note in the editor.

## Acceptance Criteria

- [x] A search bar is accessible from all views in the application.
- [x] Keyword search returns notes matching by title, tag, or content.
- [x] Semantic search returns notes ranked by vector similarity with visible relevance scores.
- [x] Results can searched by frontmatter
- [x] Results can be filtered by comma separated path start
- [x] Clicking a result opens the note in the editor.
- [x] Empty or no-match states display a helpful message for requesting missing field.
- [ ] Searching without text (purely on tags or path) works
- [ ] Totally empty query will return everything paginated
- [ ] (Extra) at an abstract interface for the database adatper to BE
