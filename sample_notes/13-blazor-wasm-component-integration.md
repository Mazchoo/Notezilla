---
phase: 4
title: Blazor WASM Component Integration
tags: [blazor, wasm, clipboard, integration]
status: todo
---

# 13 - Blazor WASM Component Integration

**Phase 4: Markdown Rendering**

Include the Rust WASM renderer in the Blazor app with SVG and HTML clipboard copy use cases.

## Steps

1. Add the compiled WASM module and JS glue code to the Blazor project's static assets.
2. Create a Blazor component that calls the WASM render function via JS interop and displays the resulting HTML.
3. Initialize Mermaid.js on the client side to render any Mermaid diagram blocks output by the WASM renderer.
4. Implement a "Copy as HTML" button that copies the rendered output to the clipboard as rich HTML.
5. Implement a "Copy as SVG" button for Mermaid diagrams that extracts the rendered SVG and copies it to the clipboard.
6. Style the rendered output to match the application theme (light/dark mode support).
7. Write integration tests verifying the render pipeline from Markdown input to displayed output.

## Acceptance Criteria

- [ ] The Blazor app loads the WASM module and renders Markdown via JS interop without errors.
- [ ] Mermaid diagrams render as interactive SVGs in the preview.
- [ ] "Copy as HTML" places the rendered note HTML on the clipboard.
- [ ] "Copy as SVG" places a Mermaid diagram's SVG on the clipboard.
- [ ] Rendering performance is smooth for notes up to 10,000 words.
- [ ] The rendered output respects the application's light/dark theme.
