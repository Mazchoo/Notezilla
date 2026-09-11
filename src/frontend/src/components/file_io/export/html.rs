use super::super::path::{html_page_title, path_to_html_filename};
use super::{
    build_html_document, download_text_file, entry_body_html, log_export_error, run_export,
};
use crate::constants::EXPORT_TEMPLATE;
use crate::info_messages::export_failed_toast;
use crate::models::block::EditorEntry;
use crate::rendering::render_markdown;
use leptos::prelude::*;

/// Prompt the browser to save each editor entry as a standalone HTML file.
pub fn export_entries_as_html(
    entries: Vec<EditorEntry>,
    progress: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
) {
    run_export(
        entries,
        progress,
        error_toast,
        path_to_html_filename,
        export_one,
    );
}

/// Convert one editor entry to an HTML download. Return toast text on failure.
fn export_one(entry: EditorEntry, filename: &str) -> Option<String> {
    let path = entry.title.path.get_untracked();
    let page_title = html_page_title(&path);
    let body_html = entry_body_html(entry, render_markdown);
    let document = build_html_document(EXPORT_TEMPLATE, &page_title, &body_html);
    download_text_file(filename, &document, "text/html;charset=utf-8")
        .err()
        .map(|err| log_export_error(export_failed_toast(filename, &format!("{err:?}"))))
}
