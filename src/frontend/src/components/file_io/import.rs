use super::save::{display_note_path, normalize_note_path};
use crate::components::toast::show_toast;
use crate::models::block::{split_front_matter, EditorEntry, FrontMatterBlock};
use leptos::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, FileReader, HtmlInputElement};

const FILE_READ_ERROR_TOAST: &str = "File cannot be read";

fn report_file_read_error(
    detail: &str,
    toast: RwSignal<Option<String>>,
    input: &HtmlInputElement,
) {
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
///
/// The backend returns body text with front matter already stripped and metadata
/// in a separate map — do not re-parse `---` delimiters from the body.
pub fn entry_from_note(
    path: impl Into<String>,
    body: &str,
    metadata: &HashMap<String, Value>,
) -> EditorEntry {
    entry_from_content(path, body, front_matter_from_metadata(metadata))
}

/// Replace any open entry with the same full path, then append the new one.
pub fn open_note_in_editor(entries: RwSignal<Vec<EditorEntry>>, entry: EditorEntry) {
    let path = normalize_note_path(&entry.title.path.get_untracked());
    entries.update(|list| {
        list.retain(|e| normalize_note_path(&e.title.path.get_untracked()) != path);
        list.push(entry);
    });
}

/// Relative path for an imported file (directory structure when available).
fn relative_path_from_file(file: &web_sys::File) -> String {
    js_sys::Reflect::get(file as &JsValue, &JsValue::from_str("webkitRelativePath"))
        .ok()
        .and_then(|v| v.as_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| file.name())
}

/// Handles a file-input `change` event: reads the selected file as UTF-8 text
/// and opens it in the editor, keyed by its full relative path.
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
                report_file_read_error(
                    "FileReader result unavailable",
                    toast,
                    &input_onload,
                );
                return;
            }
        };
        let Some(text) = result.as_string() else {
            report_file_read_error(
                "FileReader result is not a string",
                toast,
                &input_onload,
            );
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

fn is_internal_metadata_key(key: &str) -> bool {
    key == "filename" || key == "text" || key.starts_with('\n')
}

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

fn format_yaml_scalar(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
