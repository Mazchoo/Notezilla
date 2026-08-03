---
phase: 4
title: Rust WASM Markdown Renderer
tags: [rust, wasm, markdown, rendering]
status: todo
---

# 12 - Rust WASM Markdown Renderer

**Phase 4: Markdown Rendering**

Build a Rust WebAssembly (wasm-bindgen) component that accepts byte strings as input and renders Markdown with Mermaid diagrams and code syntax highlighting.

## Steps

1. Set up a Rust library project with `wasm-bindgen` and `wasm-pack` for building WebAssembly modules.
2. Choose and integrate a Rust Markdown parsing library (e.g. `pulldown-cmark` or `comrak`) for Markdown-to-HTML conversion.
3. Add code syntax highlighting support using a Rust library (e.g. `syntect`) that produces highlighted HTML spans.
4. Implement Mermaid diagram detection: identify fenced code blocks with the `mermaid` language tag and pass them through for client-side rendering.
5. Implement the public WASM API: a function that accepts byte strings (`&[u8]`), converts to UTF-8, and returns rendered HTML as a string.
6. Optimize the WASM binary size (strip debug info, use `wee_alloc`, enable LTO).
7. Write Rust unit tests for Markdown rendering, syntax highlighting, and Mermaid block detection.
8. Build and publish the `.wasm` and JS glue files via `wasm-pack`.

## Acceptance Criteria

- [x] The WASM module exposes a function that accepts byte string input and returns rendered HTML.
- [x] Standard Markdown elements (headings, lists, tables, links, images, blockquotes) render correctly.
- [x] Fenced code blocks render with syntax highlighting for common languages (Python, JS, Rust, C#, etc.).
- [x] Mermaid code blocks are identified and wrapped in a way that enables client-side Mermaid.js rendering.
- [ ] Task exists to perform wasm-opt
- [x] Input encoding errors (invalid UTF-8) are handled gracefully.
