use super::super::path::{html_page_title, path_to_pdf_filename};
use super::{
    build_html_document, download_bytes_file, entry_body_html, log_export_error, run_export,
};
use crate::constants::EXPORT_PDF_TEMPLATE;
use crate::info_messages::{pdf_conversion_failed_toast, pdf_export_failed_toast};
use crate::models::block::EditorEntry;
use crate::rendering::{html_to_pdf_bytes, render_markdown_for_pdf};
use leptos::prelude::*;

/// Convert each editor entry to PDF and download the files.
pub fn export_entries_as_pdf(
    entries: Vec<EditorEntry>,
    progress: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
) {
    run_export(
        entries,
        progress,
        error_toast,
        path_to_pdf_filename,
        export_one,
    );
}

/// Convert one editor entry to a PDF download. Return toast text on failure.
fn export_one(entry: EditorEntry, filename: &str) -> Option<String> {
    let path = entry.title.path.get_untracked();
    let page_title = html_page_title(&path);
    let body_html = entry_body_html(entry, render_markdown_for_pdf);
    let document = build_html_document(EXPORT_PDF_TEMPLATE, &page_title, &body_html);
    match html_to_pdf_bytes(&document) {
        Ok(bytes) => download_bytes_file(filename, &bytes, "application/pdf")
            .err()
            .map(|err| log_export_error(pdf_export_failed_toast(filename, &format!("{err:?}")))),
        Err(e) => {
            let msg = pdf_conversion_failed_toast(filename, e);
            web_sys::console::error_1(&msg.clone().into());
            Some(msg)
        }
    }
}
