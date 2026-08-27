use crate::models::block::EditorEntry;
use leptos::prelude::GetUntracked;
use serde_json::{json, Value};

/// Strip a leading `./` and normalise slashes for the backend note path.
pub fn normalize_note_path(path: &str) -> String {
    let path = path.trim();
    let path = path
        .strip_prefix("./")
        .or_else(|| path.strip_prefix(".\\"))
        .unwrap_or(path);
    path.replace('\\', "/")
}

/// Format a relative note path for display in the editor title block.
pub fn display_note_path(relative: &str) -> String {
    let relative = relative.trim().replace('\\', "/");
    let relative = relative.strip_prefix("./").unwrap_or(relative.as_str());
    format!("./{relative}")
}

/// Parse YAML front matter into a JSON object for `upsert_note`.
fn yaml_to_fields(raw: &str) -> Value {
    if raw.trim().is_empty() {
        return json!({});
    }
    match serde_yaml::from_str::<Value>(raw) {
        Ok(v) => v,
        Err(e) => {
            web_sys::console::warn_1(
                &format!("YAML parse failed, saving without front matter fields: {e}").into(),
            );
            json!({})
        }
    }
}

// Platform independent
/// Normalize markdown line endings to `\n`.
fn normalize_markdown_body(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Return the path, markdown body, and front-matter fields for `upsert_note`.
pub fn entry_save_params(entry: EditorEntry) -> (String, String, Value) {
    let path = normalize_note_path(&entry.title.path.get_untracked());
    let contents = normalize_markdown_body(&entry.content.text.get_untracked());
    let fields = match entry.front_matter.get_untracked() {
        Some(fm) => yaml_to_fields(&fm.raw.get_untracked()),
        None => json!({}),
    };
    (path, contents, fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::block::{EditorEntry, FrontMatterBlock};
    use leptos::prelude::{Owner, Set};

    #[test]
    /// Assert a leading `./` or `.\\` is stripped and backslashes become slashes.
    fn normalize_note_path_strips_dot_slash_and_normalises() {
        assert_eq!(normalize_note_path("./notes/a.md"), "notes/a.md");
        assert_eq!(normalize_note_path(".\\notes\\a.md"), "notes/a.md");
        assert_eq!(normalize_note_path("  notes/a.md  "), "notes/a.md");
        assert_eq!(normalize_note_path("notes/a.md"), "notes/a.md");
    }

    #[test]
    /// Assert display paths use `./` and forward slashes.
    fn display_note_path_prefixes_dot_slash() {
        assert_eq!(display_note_path("notes\\a.md"), "./notes/a.md");
        assert_eq!(display_note_path("./notes/a.md"), "./notes/a.md");
        assert_eq!(display_note_path("  notes/a.md  "), "./notes/a.md");
    }

    #[test]
    /// Assert save params normalise the path and body and parse YAML fields.
    fn entry_save_params_normalises_path_body_and_fields() {
        let owner = Owner::new();
        owner.with(|| {
            let entry = EditorEntry::new("./notes\\hello.md", "a\r\nb");
            entry
                .front_matter
                .set(Some(FrontMatterBlock::new("title: Hello\ncount: 2")));
            let (path, contents, fields) = entry_save_params(entry);
            assert_eq!(path, "notes/hello.md");
            assert_eq!(contents, "a\nb");
            assert_eq!(fields, json!({"title": "Hello", "count": 2}));
        });
    }

    #[test]
    /// Assert missing or empty front matter yields an empty fields object.
    fn entry_save_params_empty_front_matter_is_empty_object() {
        let owner = Owner::new();
        owner.with(|| {
            let entry = EditorEntry::new("a.md", "body");
            let (_, _, fields) = entry_save_params(entry);
            assert_eq!(fields, json!({}));

            entry.front_matter.set(Some(FrontMatterBlock::new("   ")));
            let (_, _, empty) = entry_save_params(entry);
            assert_eq!(empty, json!({}));
        });
    }

    #[test]
    /// Assert CRLF line endings are converted to `\n`.
    fn normalize_markdown_body_converts_crlf() {
        assert_eq!(normalize_markdown_body("a\r\nb\r\n"), "a\nb\n");
        assert_eq!(normalize_markdown_body("a\nb"), "a\nb");
    }

    #[test]
    /// Assert YAML mappings become JSON objects and blank input is `{}`.
    fn yaml_to_fields_parses_mapping_or_empty() {
        assert_eq!(yaml_to_fields(""), json!({}));
        assert_eq!(yaml_to_fields("  "), json!({}));
        assert_eq!(yaml_to_fields("title: Hello\ncount: 2"), json!({"title": "Hello", "count": 2}));
    }
}
