use super::save::display_note_path;
use crate::components::sidebar::file_tree_backend::FileTreeBackend;
use crate::components::toast::show_toast;
use crate::info_messages::FILE_READ_ERROR_TOAST;
use crate::models::block::{split_front_matter, EditorEntry, FrontMatterBlock};
use leptos::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, FileReader, HtmlInputElement};

/// Log a file-read failure, show a toast, and reset the file input.
fn report_file_read_error(detail: &str, toast: RwSignal<Option<String>>, input: &HtmlInputElement) {
    web_sys::console::error_1(&detail.into());
    show_toast(toast, FILE_READ_ERROR_TOAST);
    input.set_value("");
}

/// Build an [`EditorEntry`] from a note path, markdown body, and optional front matter YAML.
pub fn entry_from_content(
    path: impl Into<String>,
    body: &str,
    front_matter_raw: Option<String>,
) -> EditorEntry {
    if body.is_empty() && front_matter_raw.is_none() {
        return EditorEntry::empty(path);
    }

    let entry = EditorEntry::new(path, body);
    if let Some(raw) = front_matter_raw.filter(|raw| !raw.is_empty()) {
        entry.front_matter.set(Some(FrontMatterBlock::new(raw)));
    }
    entry
}

/// Build an [`EditorEntry`] from a full markdown file (body may include YAML front matter).
pub fn entry_from_markdown(path: impl Into<String>, text: &str) -> EditorEntry {
    if text.is_empty() {
        return EditorEntry::empty(path);
    }
    let (fm_raw, content) = split_front_matter(text);
    entry_from_content(path, &content, fm_raw)
}

/// Build an [`EditorEntry`] from a backend `get_note` document and metadata.
pub fn entry_from_note(
    path: impl Into<String>,
    body: &str,
    metadata: &HashMap<String, Value>,
) -> EditorEntry {
    entry_from_content(path, body, front_matter_from_metadata(metadata))
}

/// Replace any open note with the same full path, then insert the new one before templates.
pub fn open_note_in_editor(entries: RwSignal<Vec<EditorEntry>>, entry: EditorEntry) {
    entries.update(|list| FileTreeBackend::Notes.open_in_editor(list, entry));
}

/// Return the relative path for an imported file.
fn relative_path_from_file(file: &web_sys::File) -> String {
    js_sys::Reflect::get(file as &JsValue, &JsValue::from_str("webkitRelativePath"))
        .ok()
        .and_then(|v| v.as_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| file.name())
}

/// Read the selected markdown file and open it in the editor.
pub fn load_markdown_file(
    ev: Event,
    entries: RwSignal<Vec<EditorEntry>>,
    toast: RwSignal<Option<String>>,
) {
    let input = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

    let Some(input) = input else { return };
    let Some(file_list) = input.files() else {
        return;
    };
    let Some(file) = file_list.get(0) else { return };

    let file_path = display_note_path(&relative_path_from_file(&file));

    let Some(reader) = FileReader::new().ok() else {
        report_file_read_error("FileReader not available", toast, &input);
        return;
    };
    let reader_clone = reader.clone();
    let input_onload = input.clone();
    let input_onerror = input.clone();

    let onload = Closure::once(move || {
        let result = match reader_clone.result() {
            Ok(value) => value,
            Err(_) => {
                report_file_read_error("FileReader result unavailable", toast, &input_onload);
                return;
            }
        };
        let Some(text) = result.as_string() else {
            report_file_read_error("FileReader result is not a string", toast, &input_onload);
            return;
        };

        let entry = entry_from_markdown(file_path, &text);
        open_note_in_editor(entries, entry);

        input_onload.set_value("");
    });

    let onerror = Closure::once(move || {
        report_file_read_error("Failed to read file", toast, &input_onerror);
    });

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    if reader.read_as_text(&file).is_err() {
        report_file_read_error("Failed to start reading file", toast, &input);
    }
}

/// Build YAML front matter from a backend metadata map.
fn front_matter_from_metadata(meta: &HashMap<String, Value>) -> Option<String> {
    let mut scalar_fields: Vec<(String, Value)> = Vec::new();
    let mut list_items: HashMap<String, Vec<String>> = HashMap::new();

    for (key, val) in meta {
        if is_internal_metadata_key(key) {
            continue;
        }
        if let Some((field, item)) = key.split_once('\t') {
            if val.as_bool() == Some(true) {
                list_items
                    .entry(field.to_string())
                    .or_default()
                    .push(item.to_string());
            }
            continue;
        }
        scalar_fields.push((key.clone(), val.clone()));
    }

    if scalar_fields.is_empty() && list_items.is_empty() {
        return None;
    }

    scalar_fields.sort_by(|a, b| a.0.cmp(&b.0));

    let mut lines = Vec::new();
    for (key, val) in scalar_fields {
        lines.push(format_front_matter_line(&key, &val));
    }

    let mut list_fields: Vec<_> = list_items.into_iter().collect();
    list_fields.sort_by(|a, b| a.0.cmp(&b.0));
    for (field, mut items) in list_fields {
        items.sort();
        lines.push(format!("{field}: [{}]", items.join(", ")));
    }

    Some(lines.join("\n"))
}

/// Return whether a metadata key is an internal backend field.
fn is_internal_metadata_key(key: &str) -> bool {
    key == "filename" || key == "text" || key.starts_with('\n')
}

/// Format one YAML front-matter `key: value` line.
fn format_front_matter_line(key: &str, val: &Value) -> String {
    if let Value::String(s) = val {
        if let Ok(parsed) = serde_json::from_str::<Value>(s) {
            return format_front_matter_line(key, &parsed);
        }
        return format!("{key}: {s}");
    }
    if let Value::Array(items) = val {
        let formatted: Vec<String> = items.iter().map(format_yaml_scalar).collect();
        return format!("{key}: [{}]", formatted.join(", "));
    }
    format!("{key}: {}", format_yaml_scalar(val))
}

/// Format a JSON value as a YAML scalar.
fn format_yaml_scalar(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::{GetUntracked, Owner, RwSignal};
    use serde_json::json;

    #[test]
    /// Assert internal backend keys are skipped.
    fn is_internal_metadata_key_matches_filename_text_and_newline() {
        assert!(is_internal_metadata_key("filename"));
        assert!(is_internal_metadata_key("text"));
        assert!(is_internal_metadata_key("\ninternal"));
        assert!(!is_internal_metadata_key("title"));
    }

    #[test]
    /// Assert scalars stringify and nested JSON is left as a JSON string.
    fn format_yaml_scalar_stringifies_json_values() {
        assert_eq!(format_yaml_scalar(&json!("hello")), "hello");
        assert_eq!(format_yaml_scalar(&json!(2)), "2");
        assert_eq!(format_yaml_scalar(&json!(true)), "true");
        assert_eq!(format_yaml_scalar(&json!(null)), "");
        assert_eq!(format_yaml_scalar(&json!({"a": 1})), "{\"a\":1}");
    }

    #[test]
    /// Assert a front-matter line formats scalars, arrays, and nested JSON strings.
    fn format_front_matter_line_formats_scalars_and_arrays() {
        assert_eq!(
            format_front_matter_line("title", &json!("Hello")),
            "title: Hello"
        );
        assert_eq!(format_front_matter_line("n", &json!(3)), "n: 3");
        assert_eq!(
            format_front_matter_line("tags", &json!(["b", "a"])),
            "tags: [b, a]"
        );
        assert_eq!(
            format_front_matter_line("tags", &json!("[\"a\", \"b\"]")),
            "tags: [a, b]"
        );
    }

    #[test]
    /// Assert metadata maps skip internals, expand tab-list flags, and sort keys.
    fn front_matter_from_metadata_builds_sorted_yaml() {
        let mut meta = HashMap::new();
        meta.insert("filename".into(), json!("skip.md"));
        meta.insert("text".into(), json!("body"));
        meta.insert("title".into(), json!("Hello"));
        meta.insert("priority".into(), json!(1));
        meta.insert("tags\twork".into(), json!(true));
        meta.insert("tags\thome".into(), json!(true));
        meta.insert("tags\tskip".into(), json!(false));
        let yaml = front_matter_from_metadata(&meta).expect("yaml");
        assert_eq!(yaml, "priority: 1\ntitle: Hello\ntags: [home, work]");
        assert_eq!(front_matter_from_metadata(&HashMap::new()), None);
    }

    #[test]
    /// Assert markdown with YAML front matter becomes an entry with both parts.
    fn entry_from_markdown_splits_front_matter() {
        let owner = Owner::new();
        owner.with(|| {
            let entry = entry_from_markdown("./a.md", "---\ntitle: x\n---\nbody");
            assert_eq!(entry.title.path.get_untracked(), "./a.md");
            assert_eq!(entry.content.text.get_untracked(), "body");
            let fm = entry.front_matter.get_untracked().expect("front matter");
            assert_eq!(fm.raw.get_untracked(), "title: x");

            let empty = entry_from_markdown("./b.md", "");
            assert_eq!(empty.content.text.get_untracked(), "");
            assert!(empty.front_matter.get_untracked().is_none());
        });
    }

    #[test]
    /// Assert a note's metadata map is converted into front matter on the entry.
    fn entry_from_note_uses_metadata_as_front_matter() {
        let owner = Owner::new();
        owner.with(|| {
            let mut meta = HashMap::new();
            meta.insert("title".into(), json!("Hello"));
            let entry = entry_from_note("./a.md", "body", &meta);
            assert_eq!(entry.content.text.get_untracked(), "body");
            let fm = entry.front_matter.get_untracked().expect("front matter");
            assert_eq!(fm.raw.get_untracked(), "title: Hello");
        });
    }

    #[test]
    /// Assert opening a note replaces any existing entry with the same path.
    fn open_note_in_editor_replaces_same_path() {
        let owner = Owner::new();
        owner.with(|| {
            let entries = RwSignal::new(vec![EditorEntry::new("./a.md", "old")]);
            open_note_in_editor(entries, EditorEntry::new("./a.md", "new"));
            let list = entries.get_untracked();
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].content.text.get_untracked(), "new");

            open_note_in_editor(entries, EditorEntry::new("./b.md", "other"));
            let list = entries.get_untracked();
            assert_eq!(list.len(), 2);
        });
    }
}
