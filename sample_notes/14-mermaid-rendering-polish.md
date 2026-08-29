---
phase: 5
title: Mermaid Rendering Polish
tags: [mermaid, rendering, gui, polish]
status: todo
---

# 15 - Mermaid Rendering Polish

**Phase 5: Sync & Polish**

Ensure the GUI correctly renders Mermaid diagrams to satisfy the "extending user memory" principle.

## Steps

1. Audit all Mermaid diagram types (flowchart, sequence, class, state, ER, Gantt, pie, etc.) and verify they render correctly.
2. Fix any rendering issues with specific diagram types or edge cases (large diagrams, special characters).
3. Add zoom and pan controls for large diagrams so they are fully explorable.
4. Implement diagram export: allow users to download a rendered diagram as SVG or PNG.
5. Add a Mermaid syntax reference or cheat sheet accessible from the editor toolbar.
6. Ensure diagrams re-render correctly on theme change (light/dark mode).

## Acceptance Criteria

- [ ] All major Mermaid diagram types render correctly in the preview pane.
- [ ] Large diagrams are scrollable or zoomable, not clipped or overflowing.
- [ ] Diagrams can be exported as SVG or PNG files via a download action.
- [ ] Theme changes (light/dark) are reflected in diagram colors without a page reload.
- [ ] Special characters in diagram labels do not break rendering.
- [ ] A Mermaid syntax reference is accessible from the editor for quick lookup.
