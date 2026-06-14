use crate::models::block::{split_front_matter, EditorEntry, FrontMatterBlock};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{Event, FileReader, HtmlInputElement};

/// Handles a file-input `change` event: reads the selected file as UTF-8 text
/// and replaces the provided `entries` signal with a single [`EditorEntry`]
/// (title block + markdown content block) containing the entire file content.
pub fn load_markdown_file(ev: Event, entries: RwSignal<Vec<EditorEntry>>) {
    let input = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

    let Some(input) = input else { return };
    let Some(file_list) = input.files() else {
        return;
    };
    let Some(file) = file_list.get(0) else { return };

    // Capture the file name to use as the title path.
    let file_name = file.name();
    let file_path = format!("./{}", file_name);

    let reader = FileReader::new().expect("FileReader not available");
    let reader_clone = reader.clone();

    let onload = Closure::once(move || {
        let result = reader_clone
            .result()
            .expect("FileReader result unavailable");
        let text = result
            .as_string()
            .expect("FileReader result is not a string");

        // Parse front matter (if any), then build the entry.
        // EditorEntry::new always starts with a blank "tags: []" front matter;
        // if the file has its own front matter, overwrite the signal with it.
        let entry = if text.is_empty() {
            EditorEntry::empty(file_path)
        } else {
            let (fm_raw, content) = split_front_matter(&text);
            let entry = EditorEntry::new(file_path, content);
            if let Some(raw) = fm_raw {
                // Override the default blank front matter with the file's own.
                entry.front_matter.set(Some(FrontMatterBlock::new(raw)));
            }
            entry
        };
        entries.set(vec![entry]);

        // Reset so the same file can be re-imported if needed.
        input.set_value("");
    });

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    reader
        .read_as_text(&file)
        .expect("Failed to start reading file");
}
