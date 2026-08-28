use crate::constants::PROMPT_TEMPLATE;

/// Fill the prompt template with the user prompt, open-note context, and response template.
pub fn create_prompt(user_prompt: &str, context: &str, response_template: &str) -> String {
    PROMPT_TEMPLATE
        .replace("{{USER_PROMPT}}", user_prompt)
        .replace("{{CONTEXT}}", context)
        .replace("{{RESPONSE_TEMPLATE}}", response_template)
}

#[cfg(test)]
mod tests {
    use super::create_prompt;

    #[test]
    /// Assert the prompt template receives the user prompt, context, and response template.
    fn create_prompt_fills_user_context_and_response_placeholders() {
        let out = create_prompt("Ask this", "note body", "template body");
        assert!(out.contains("Ask this"));
        assert!(out.contains("note body"));
        assert!(out.contains("template body"));
        assert!(!out.contains("{{USER_PROMPT}}"));
        assert!(!out.contains("{{CONTEXT}}"));
        assert!(!out.contains("{{RESPONSE_TEMPLATE}}"));
    }
}
