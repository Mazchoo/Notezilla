---
phase: 3
title: Generation Hub
tags: [gui, blazor, llm, generation]
status: todo
---

# 11 - Generation Hub

**Phase 3: Blazor GUI Development**

A dedicated view to select a template, trigger Ollama, and review the generated Markdown before saving.

## Steps

1. Create a new page/view accessible from the main navigation for note generation.
2. Build a template selector that lists available templates with their names and descriptions from the template engine API.
3. Add input fields for any user-provided context or placeholder values required by the selected template.
4. Implement a "Generate" button that sends the request to the Ollama connector via the API.
5. Display the streaming LLM output in a live preview pane as it is generated.
6. Add an inline editor for the generated output so the user can make edits before saving.
7. Implement a "Save as Note" action that writes the final content to the file tree via the API.
8. Show the target file path and allow the user to adjust it before saving.

## Acceptance Criteria

- [ ] A dedicated generation view is accessible from the sidebar or top navigation.
- [ ] Available templates are listed with name and description; selecting one shows its required inputs.
- [ ] The user can provide context or placeholder values before generating.
- [ ] Clicking "Generate" triggers LLM generation and displays streaming output in real time.
- [ ] The generated Markdown can be edited inline before saving.
- [ ] "Save as Note" writes the note to the chosen path and it appears in the file tree.
- [ ] Generation errors (Ollama offline, timeout) display a clear error message with retry option.
