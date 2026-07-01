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
fn normalize_markdown_body(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Path, markdown body, and front-matter fields for `upsert_note`.
pub fn entry_save_params(entry: EditorEntry) -> (String, String, Value) {
    let path = normalize_note_path(&entry.title.path.get_untracked());
    let contents = normalize_markdown_body(&entry.content.text.get_untracked());
    let fields = match entry.front_matter.get_untracked() {
        Some(fm) => yaml_to_fields(&fm.raw.get_untracked()),
        None => json!({}),
    };
    (path, contents, fields)
}
