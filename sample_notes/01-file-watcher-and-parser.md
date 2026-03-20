---
phase: 1
title: File Watcher & Parser
tags: [core, filesystem, parser, python]
status: todo
---

# 01 - File Watcher & Parser

**Phase 1: Core Python Library & FastMCP API**

Implement a service to monitor the "Filesystem Notes" folder and parse Markdown frontmatter for tags and paths.

## Steps

1. Choose and integrate a filesystem watching library (e.g. `watchdog`) to monitor a configurable notes directory for file create, update, and delete events.
2. Implement a Markdown frontmatter parser that extracts YAML metadata (tags, path, title) from `.md` files.
3. Build an event dispatcher that emits structured change events (created, modified, deleted) with the parsed note metadata and content.
4. Handle edge cases: rapid successive edits (debouncing), temporary files, non-markdown files in the directory.
5. Add configuration for the root notes directory path via environment variable or config file.
6. Write unit tests for the parser and integration tests for the watcher.

## Acceptance Criteria

- [x] The watcher detects new, modified, and deleted `.md` files within the watched directory and subdirectories.
- [ ] Frontmatter with `tags` and `path` fields is correctly parsed from each note.
- [ ] Malformed or missing frontmatter is handled gracefully with a warning, not a crash.
- [x] Rapid file changes are debounced so downstream consumers receive a single consolidated event.
- [ ] Non-markdown files are ignored.
- [ ] The watched directory path is configurable.
