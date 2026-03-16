---
phase: 2
title: Template Engine
tags: [llm, templates, python]
status: todo
---

# 05 - Template Engine

**Phase 2: Local LLM Integration (Ollama)**

Create a system to store Markdown templates with placeholders for LLM injection.

## Steps (ToDo - expect markdown templates to be more flexible than this)

1. Design a template format using Markdown with placeholder syntax (e.g. `{{context}}`, `{{summary}}`) for LLM-generated content.
2. Implement a template loader that reads `.md` template files from a configurable templates directory.
3. Build a template renderer that substitutes placeholders with provided values (context from vector search, user input, etc.).
4. Create a set of built-in starter templates: meeting notes summary, topic research, comparison table, decision log.
5. Add template metadata (name, description, required placeholders) via frontmatter so the GUI can display available templates.
6. Write tests for template loading, validation, and rendering.

## Acceptance Criteria

- [ ] Templates are stored as `.md` files in a dedicated templates directory.
- [ ] Each template has frontmatter with `name`, `description`, and `placeholders` fields.
- [ ] The renderer substitutes all placeholders and returns valid Markdown output.
- [ ] Missing required placeholders raise a clear validation error.
- [ ] At least 4 built-in starter templates are provided.
- [ ] Templates can reference context retrieved from the vector database via a `{{context}}` placeholder.
