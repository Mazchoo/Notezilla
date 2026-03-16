---
phase: 3
title: Markdown Editor
tags: [gui, blazor, editor, markdown]
status: todo
---

# 09 - Markdown Editor

**Phase 3: Blazor GUI Development**

Integrate a side-by-side Markdown editor with live preview and Mermaid diagram support.

## Steps

1. Integrate or build a text editor component with Markdown syntax highlighting.
2. Implement a split-pane layout with the editor on the left and rendered preview on the right.
3. Wire live preview updates so that typing in the editor immediately re-renders the preview.
4. Add a frontmatter editor panel (above or inline) for editing tags and path metadata.
5. Implement save functionality that calls the `update_note` API and provides visual feedback.
6. Add keyboard shortcuts for common actions: save (Ctrl+S), bold, italic, heading, code block.
7. Support drag-and-drop or paste for inserting images (stored alongside the note).

## Acceptance Criteria

- [ ] The editor displays Markdown source with syntax highlighting.
- [ ] A side-by-side preview renders the Markdown in real time as the user types.
- [ ] Mermaid diagram blocks render as diagrams in the preview (via the WASM renderer from Phase 4 or a fallback).
- [ ] Code blocks display with syntax highlighting in the preview.
- [ ] Frontmatter fields (tags, path) are editable via a structured form or inline YAML.
- [ ] Saving a note calls the API and shows a success/error notification.
- [ ] Standard keyboard shortcuts work for formatting and saving.
