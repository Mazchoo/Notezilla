---
phase: 2
title: Template Engine
tags: [llm, templates, python]
status: todo
---

# TODO - Research what are good templates for reports, diagrams ect

# 05 - Template Engine

**Phase 2: Local LLM Integration (Ollama)**

Create a system to store Markdown templates with placeholders for LLM injection.

## Steps (ToDo - expect markdown templates to be more flexible than this)

1. Design a template format using Markdown with placeholder syntax (e.g. `{{context}}`, `{{summary}}`) for LLM-generated content.
2. Implement a template loader that reads `.md` template files from a configurable templates directory.
3. Build a template renderer that substitutes placeholders with provided values (context from vector search, user input, etc.).
4. Create a set of built-in starter templates: topic research with diagrams, animal facts, code demonstration and mathematical proof.
5. Add template metadata (name, description, required placeholders) via frontmatter so the GUI can display available templates.
6. Write tests for template loading, validation, and rendering.

## Acceptance Criteria

- [ ] Ensure that all rendered notes are collapsible
- [ ] Templates are stored as `.md` files in a dedicated templates directory.
- [ ] At least 4 built-in starter templates are provided.
- [ ] Backend api call is available to get, delete and update templates
- [ ] Only a single template can be open in the editor at any given time
- [ ] Template must be visually distinctive from other files
- [ ] Tests exist on frontend for utilities and file creation and ensure they can be debugged locally
