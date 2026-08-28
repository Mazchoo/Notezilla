use crate::components::sidebar::file_tree_backend::FileTreeBackend;
use crate::constants::PROMPT_TEMPLATE;
use crate::models::block::EditorEntry;
use leptos::prelude::*;

/// Serialize an editor entry to markdown, including front matter when present.
fn entry_markdown(entry: EditorEntry) -> String {
    let body = entry.content.text.get_untracked();
    match entry.front_matter.get_untracked() {
        Some(fm) => {
            let raw = fm.raw.get_untracked();
            if raw.is_empty() {
                body
            } else {
                format!("---\n{raw}\n---\n{body}")
            }
        }
        None => body,
    }
}

/// Fill the prompt template from the user text and open editor entries.
pub fn build_prompt(user_prompt: &str, entries: &[EditorEntry]) -> String {
    let mut context_parts = Vec::new();
    let mut response_template = String::new();
    for entry in entries {
        let markdown = entry_markdown(*entry);
        match entry.backend {
            FileTreeBackend::Notes => {
                let path = entry.title.path.get_untracked();
                context_parts.push(format!("## {path}\n\n{markdown}"));
            }
            FileTreeBackend::Templates => response_template = markdown,
        }
    }
    create_prompt(user_prompt, &context_parts.join("\n\n"), &response_template)
}

/// Fill the prompt template with the user prompt, open-note context, and response template.
pub fn create_prompt(user_prompt: &str, context: &str, response_template: &str) -> String {
    PROMPT_TEMPLATE
        .replace("{{USER_PROMPT}}", user_prompt)
        .replace("{{CONTEXT}}", context)
        .replace("{{RESPONSE_TEMPLATE}}", response_template)
}

#[cfg(test)]
mod tests {
    use super::{build_prompt, create_prompt};
    use crate::components::sidebar::file_tree_backend::FileTreeBackend;
    use crate::models::block::EditorEntry;
    use leptos::prelude::Owner;

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

    #[test]
    /// Assert open notes fill context and the open template fills the response slot.
    fn build_prompt_uses_open_notes_and_template() {
        let owner = Owner::new();
        owner.with(|| {
            let note = EditorEntry::new("./a.md", "note body");
            let mut tmpl = EditorEntry::new("./t.md", "template body");
            tmpl.backend = FileTreeBackend::Templates;
            let out = build_prompt("Ask this", &[note, tmpl]);
            assert!(out.contains("Ask this"));
            assert!(out.contains("./a.md"));
            assert!(out.contains("note body"));
            assert!(out.contains("template body"));
            assert!(!out.contains("{{USER_PROMPT}}"));
            assert!(!out.contains("{{CONTEXT}}"));
            assert!(!out.contains("{{RESPONSE_TEMPLATE}}"));
        });
    }
}
