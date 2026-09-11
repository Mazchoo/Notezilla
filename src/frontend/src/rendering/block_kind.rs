//! Classifies a fenced markdown code block for special rendering.

/// Kind of fenced block intercepted during markdown parsing.
pub(super) enum BlockKind {
    Graphviz,
    Mermaid,
    /// Code in the fence's language token, which may be empty.
    Code(String),
}

impl BlockKind {
    /// Classify a fence by the first token of its info string, ignoring case.
    pub(super) fn from_fence_language(language: &str) -> Self {
        let token = language.split_ascii_whitespace().next().unwrap_or("");
        match token.to_ascii_lowercase().as_str() {
            "graphviz" => BlockKind::Graphviz,
            "mermaid" => BlockKind::Mermaid,
            _ => BlockKind::Code(token.to_string()),
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
    /// Assert the fence language is the first info-string token.
    fn diagram_fence_uses_the_first_info_token() {
        assert!(matches!(
            BlockKind::from_fence_language("graphviz title"),
            BlockKind::Graphviz
        ));
        assert!(matches!(
            BlockKind::from_fence_language("mermaid extra_info"),
            BlockKind::Mermaid
        ));
    }

    #[test]
    /// Assert diagram fence tokens are matched without case.
    fn diagram_fence_tokens_are_case_insensitive() {
        assert!(matches!(
            BlockKind::from_fence_language("Graphviz"),
            BlockKind::Graphviz
        ));
        assert!(matches!(
            BlockKind::from_fence_language("MERMAID"),
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
