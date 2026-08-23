//! Classifies a fenced markdown code block for special rendering.

/// Kind of fenced block intercepted during markdown parsing.
pub(super) enum BlockKind {
    Graphviz,
    Mermaid,
    Code(String), // language token
}
