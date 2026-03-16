---
phase: 2
title: Summarization Logic
tags: [llm, summarization, python]
status: todo
---

# 07 - Summarization Logic

**Phase 2: Local LLM Integration (Ollama)**

Implement the "structured summary" principle to ensure LLM outputs are saved directly into the Notezilla file tree.

## Steps

1. Define a structured output format for LLM summaries that includes a title, tags, and sectioned Markdown body.
2. Build a post-processing pipeline that takes raw LLM output and formats it into a valid Notezilla note (with correct frontmatter).
3. Implement an auto-save function that writes the processed summary to the appropriate location in the file tree based on the template's target path.
4. Add a review step where the generated note can be previewed and edited before final save (exposed via API for the GUI).
5. Implement re-summarization: given an existing note, re-run it through the LLM with updated context to produce a refreshed version.
6. Write tests for output parsing, formatting, and file-save workflows.

## Acceptance Criteria

- [ ] LLM output is automatically formatted into a valid Notezilla Markdown note with frontmatter.
- [ ] Generated notes are saved to the correct file tree location and indexed in the vector database.
- [ ] A preview/review API allows the user to inspect and edit generated content before saving.
- [ ] Re-summarization updates an existing note in place without creating duplicates.
- [ ] Malformed LLM output is handled gracefully with a fallback format.
- [ ] The structured summary includes extracted tags based on content analysis.
