use crate::models::block::MarkdownBlock;
use leptos::*;
use wasm_bindgen::prelude::*;
use web_sys::{Event, FileReader, HtmlInputElement};

/// Handles a file-input `change` event: reads the selected file as UTF-8 text
/// and replaces the provided `blocks` signal with one [`MarkdownBlock`] per
/// line, preserving blank lines so that no content is lost on import.
pub fn load_markdown_file(ev: Event, blocks: RwSignal<Vec<MarkdownBlock>>) {
    let input = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

    let Some(input) = input else { return };
    let Some(file_list) = input.files() else {
        return;
    };
    let Some(file) = file_list.get(0) else { return };

    let reader = FileReader::new().expect("FileReader not available");
    let reader_clone = reader.clone();

    let onload = Closure::once(move || {
        let result = reader_clone
            .result()
            .expect("FileReader result unavailable");
        let text = result
            .as_string()
            .expect("FileReader result is not a string");

        // Every line (including blank ones) becomes its own MarkdownBlock so
        // that the original structure is preserved on import.
        let new_blocks: Vec<MarkdownBlock> = text
            .lines()
            .map(MarkdownBlock::new)
            .collect();

        blocks.set(if new_blocks.is_empty() {
            vec![MarkdownBlock::empty()]
        } else {
            new_blocks
        });

        // Reset so the same file can be re-imported if needed.
        input.set_value("");
    });

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    reader
        .read_as_text(&file)
        .expect("Failed to start reading file");
}
