use super::super::path::path_to_markdown_filename;
use super::{download_text_file, log_export_error, run_export};
use crate::info_messages::export_failed_toast;
use crate::models::block::EditorEntry;
use leptos::prelude::*;

/// Prompt the browser to save each editor entry as a markdown file.
pub fn export_entries_as_markdown(
    entries: Vec<EditorEntry>,
    progress: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
) {
    run_export(
        entries,
        progress,
        error_toast,
        path_to_markdown_filename,
        export_one,
    );
}

/// Convert one editor entry to a markdown download. Return toast text on failure.
fn export_one(entry: EditorEntry, filename: &str) -> Option<String> {
    let content = entry.to_markdown();
    download_text_file(filename, &content, "text/markdown;charset=utf-8")
        .err()
        .map(|err| log_export_error(export_failed_toast(filename, &format!("{err:?}"))))
}
