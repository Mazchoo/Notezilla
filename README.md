# Notezilla

## Summary

Python library that can store and track markdown files for LLM usage. The notes are stored in folders and can be tagged with metadata which let the text contents naturally map to a database.

## Core Principles

This project was generated from the following observations
* Extending LLM memory is also about extending the user memory. Diagrams and interactive content within markdown are a realtively easy way to make content more memorable.
* Markdown is becoming an acceptable readable format that both LLM's and people can use to break down larger tasks or topics.
* Summarising LLM results in a structured way is better than just a long list of conversations, organising conversations in folders and adding search capability is a common addition.

## Notezilla GUI

A Leptos GUI:
* A way to use search queries to find notes
* Generate new notes from a local LLM (Olama API)
* Edit skills (re-usable prompts) that can generate markdown according to a template
* Side by side rendering and editing of markdown

## Database API

A python FastMCP service that provides CRUD access to a vector database where details of notes are folder.
Each note has:
* path: ordered file tree path
* tags: unorded list of string names

## Component Diagram

```
+-----------------------+
|       Blazor GUI      |      +---------------------------------+  
|                       |      | Rust WASM markdown render       |
| Markdown Editor       |----->| Mermaid and code highlighting   |
| Search UI             |      +---------------------------------+
| LLM Generation API    |
+----------+------------+
           |              \
           |               \
           v                v

+--------------------+   +-------------------+
|   Notezilla API    |   |   Ollama API      |
|   (FastMCP)        |   |                   |
|                    |   | text generation   |
| CRUD interaction   |   | from templates    |
| vector search      |   | and system prompt |
| metadata queries   |   +-------------------+
+----------+---------+
           |          \
           |           \
           v            v
+--------------------+  +--------------------+
| Vector Database    |  | Filesystem Notes   |
| (semantic index)   |  | Markdown Files     |
+--------------------+  +--------------------+
```

## RooCode MCP Settings

Easiest way to make this available for RooCode:

`python -m src.backend.main`

After the server is running add a configuration for the MCP server.

```
{
  "mcpServers": {
    "notezilla": {
      "type": "streamable-http",
      "url": "http://localhost:8000/mcp",
      "disabled": false
    }
  }
}
```
