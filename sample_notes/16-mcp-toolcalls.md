---
phase: 6
title: MCP toolcall experimentation
tags: [polish]
status: todo
---

# 16 - MCP toolcall experimentation

Ensure that the backend MCP server can also be used to build prompts and create markdown files with a limited selection of commands. The MCP server can only create new files and does not have permission to delete or move files.

- [ ] Verify cursor and claude can use tool call to look up notes
- [ ] Verify tool call works to upsert a new markdown file and instructions are adequate
- [ ] Verify tool call works to get a note with specified file path