---
phase: 3
title: Blazor Layout & Navigation
tags: [gui, blazor, mudblazor, navigation]
status: todo
---

# 08 - Blazor Layout & Navigation

**Phase 3: Blazor GUI Development**

Create the main application layout with a sidebar file tree based on the ordered file tree path.

## Steps

1. Set up a new Blazor project with MudBlazor as the component/styling library.
2. Implement the main layout with a collapsible sidebar and a content area.
3. Build a file tree component that fetches the note list from the FastMCP API and renders it as a hierarchical tree using the `path` field.
4. Add click handling on tree nodes to open a note in the content area.
5. Implement tree node icons that distinguish folders from files and show note status.
6. Add a toolbar with actions: create new note, refresh tree, collapse/expand all.
7. Ensure the layout is responsive and works at various window sizes.

## Acceptance Criteria

- [ ] The app launches with a MudBlazor-styled layout containing a sidebar and content area.
- [ ] The sidebar displays a hierarchical file tree built from note paths returned by the API.
- [ ] Clicking a note in the tree opens it in the content area.
- [ ] The sidebar is collapsible to maximize the content area.
- [ ] A "new note" action opens a blank editor with a path selector.
- [ ] The tree refreshes automatically or via a refresh button when notes change.
