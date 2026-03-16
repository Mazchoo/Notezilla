---
phase: 1
title: Note Schema Design
tags: [core, schema, design, python]
status: todo
---

# 04 - Note Schema Design

**Phase 1: Core Python Library & FastMCP API**

Make a population job that will populate the database with all defined yaml fields and populates the database.

## Steps (ToDo - needs reworking)

1. Design a canonical JSON schema for a note including fields: `id`, `title`, `path`, `tags`, `content`, `created_at`, `updated_at`.
2. Define the Markdown frontmatter format that maps bidirectionally to the JSON schema.
3. Implement serialization: JSON to Markdown file (with frontmatter + body).
4. Implement deserialization: Markdown file (frontmatter + body) to JSON.
5. Add validation logic that enforces required fields and type constraints.
6. Document the schema with examples of valid notes in both formats.

## Acceptance Criteria

- [ ] A documented JSON schema defines all note fields with types and required/optional status.
- [ ] A corresponding Markdown frontmatter format is defined and documented.
- [ ] Round-trip conversion (Markdown -> JSON -> Markdown) preserves all data without loss.
- [ ] Validation rejects notes missing required fields (`path`, `content`) with clear error messages.
- [ ] The `tags` field accepts a list of strings; the `path` field represents an ordered file tree location.
- [ ] At least 3 example notes demonstrate the schema in both JSON and Markdown formats.
