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
4. Implement streaming response handling so the GUI can display generation progress.
5. Add a health-check function to verify the Ollama instance is running and the requested model is available.
6. Write tests using mocked Ollama responses for the client, prompt assembly, and error paths.

## Acceptance Criteria

- [ ] The connector sends well-formed requests to a local Ollama instance and receives generated text.
- [ ] The Ollama host, model, and generation parameters are configurable.
- [ ] Context from the vector database is correctly included in the prompt sent to Ollama.
- [ ] Streaming responses are supported so partial output can be displayed progressively.
- [ ] A health-check endpoint returns the connection status and available models.
- [ ] Graceful error handling when Ollama is offline or the model is not found.
