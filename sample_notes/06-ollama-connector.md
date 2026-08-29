---
phase: 2
title: Ollama Connector
tags: [llm, ollama, python]
status: todo
---

# 06 - Ollama Connector

**Phase 2: Local LLM Integration (Ollama)**

Build the logic to send system prompts and context (retrieved from Phase 1) to a local Ollama instance.

## Steps

1. Implement an HTTP client to communicate with the Ollama REST API (`/api/generate` or `/api/chat`).
2. Add configuration for Ollama host URL, model name, and generation parameters (temperature, max tokens).
3. Build a prompt assembly function that combines a system prompt, retrieved context from the vector database, and the rendered template into a single request.
4. Chat interface sidebar is available, it uses a prompt and uses all open files. The prompt is then followed by open files as context. The prompt ends with a single template markdown given as output format.
5. Implement streaming response handling to save response to markdown.
6. Add a health-check function to verify the Ollama instance is running and the requested model is available.
7. Write tests using mocked Ollama responses for the client, prompt assembly, and error paths.

## Acceptance Criteria

- [x] Prompt to LLM with context and response template can be copied to clipboard with top bar button
- [x] The connector sends well-formed requests to a local Ollama instance and receives generated text.
- [x] Option is avaible to simply copy the formulated prompt for use elsewhere
- [ ] The Ollama host, model, and generation parameters are configurable.
- [x] All open markdown files in the editor as well as open templates are used to create a prompt
- [x] Response is saved as markdown file with provided name
- [x] A health-check endpoint returns the connection status and available models.
- [x] Graceful error handling when Ollama is offline or the model is not found.

ToDo - check why refining_photogrametry_scenes.md suddenly indents a formula
