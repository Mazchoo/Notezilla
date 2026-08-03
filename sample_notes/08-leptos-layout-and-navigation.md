---
phase: 3
title: Leptos Layout & Navigation
tags: [gui, leptos, bulma, navigation]
status: todo
---

# 08 - Leptos Layout & Navigation

**Phase 3: Leptos GUI Development**

Create the main application layout in a VS Code-style shell (activity bar, sidebar, editor) using Leptos and Bulma CSS, with a sidebar file tree based on the ordered file tree path.

## Steps

1. Set up a Leptos (WASM) frontend and include Bulma CSS for base styling, with dark theme overrides for the VS Code-style chrome.
2. Implement `AppShell` with a top bar, activity bar, collapsible/resizable sidebar, and content/editor area.
3. Build a file tree component that fetches the note list from the FastMCP API and renders it as a hierarchical tree using the `path` field (Bulma `menu` / `menu-list`).
4. Add click handling on tree nodes to open a note in the content area.
5. Implement tree node icons that distinguish folders from files and show note status.
6. Add toolbar/activity-bar actions: create new note, refresh tree, collapse/expand all, and switch sidebar panels as needed.
7. Ensure the layout is responsive and works at various window sizes.

## Acceptance Criteria

- [x] The app launches with a Leptos + Bulma VS Code-style layout: activity bar, sidebar, and content area.
- [x] The sidebar displays a hierarchical file tree built from note paths returned by the API.
- [x] Clicking a note in the tree opens it in the content area.
- [x] The sidebar is collapsible (and preferably resizable) to maximize the content area.
- [x] A "new note" action opens a blank editor with a path selector.
- [x] The tree refreshes automatically or via a refresh button when notes change.
