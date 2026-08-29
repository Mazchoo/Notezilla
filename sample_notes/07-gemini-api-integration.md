---
phase: 2
title: Gemini API Integration
tags: [llm, gemini]
status: todo
---

# 07 - Gemini API Integration

**Phase 2: LLM Integration (Gemini)**

Build the logic to send system prompts and context to the Gemini API. The user specifies an API key and a model name.

## Steps

1. Implement an HTTP client to communicate with the Gemini REST API (`generateContent`).
2. Add configuration for Gemini API key, model name, and generation parameters (temperature, max tokens).
3. Reuse the existing prompt assembly: user prompt, all open files as context, and a single template markdown as output format.
4. Chat interface sidebar can send the assembled prompt to Gemini as well as Ollama.
5. Implement streaming response handling to save response to markdown.
6. Add a health-check that pings `GET https://generativelanguage.googleapis.com/v1beta/models/{model}?key={api_key}` and treats a successful response as the key and model being valid.
7. Write tests using mocked Gemini responses for the client, health-check, and error paths.

## Acceptance Criteria

- [ ] The connector sends well-formed requests to Gemini and receives generated text.
- [ ] The Gemini API key, model, and generation parameters are configurable.
- [ ] Setting a configuration to an invalid value will reject it and return an error toast.
- [ ] All open markdown files in the editor as well as open templates are used to create a prompt.
- [ ] Response is saved as markdown file with provided name.
- [ ] A health-check of `GET https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash?key=YOUR_API_KEY` returns okay when the model is valid.
- [ ] Graceful error handling when the API key is rejected or the model is not found.
