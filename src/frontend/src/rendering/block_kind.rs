//! Classifies a fenced markdown code block for special rendering.

/// Kind of fenced block intercepted during markdown parsing.
pub(super) enum BlockKind {
    Graphviz,
    Mermaid,
    /// Code in the fence's language token, which may be empty.
    Code(String),
}

impl BlockKind {
    /// Classify a fence by its language token.
    pub(super) fn from_fence_language(language: &str) -> Self {
        match language {
            "graphviz" => BlockKind::Graphviz,
            "mermaid" => BlockKind::Mermaid,
            other => BlockKind::Code(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BlockKind;

    #[test]
    /// Assert the diagram fences are classified by their language token.
    fn classifies_diagram_fences() {
        assert!(matches!(
            BlockKind::from_fence_language("graphviz"),
            BlockKind::Graphviz
        ));
        assert!(matches!(
            BlockKind::from_fence_language("mermaid"),
            BlockKind::Mermaid
        ));
    }

    #[test]
    /// Assert any other token is code, keeping the token as the language.
    fn classifies_other_fences_as_code() {
        assert!(
            matches!(BlockKind::from_fence_language("rust"), BlockKind::Code(lang) if lang == "rust")
        );
        assert!(
            matches!(BlockKind::from_fence_language(""), BlockKind::Code(lang) if lang.is_empty())
        );
    }
}
