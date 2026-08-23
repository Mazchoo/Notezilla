use super::path::{
    html_page_title, path_to_html_filename, path_to_markdown_filename, path_to_pdf_filename,
};
use crate::models::block::EditorEntry;
use crate::rendering::{escape_html, html_to_pdf_bytes, render_markdown, render_markdown_for_pdf};
use leptos::prelude::GetUntracked;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement};

const EXPORT_TEMPLATE: &str = include_str!("../../../templates/export.html");
const EXPORT_PDF_TEMPLATE: &str = include_str!("../../../templates/export-pdf.html");

/// Prompt the browser to save each editor entry as a standalone HTML file.
pub fn export_entries_as_html(entries: &[EditorEntry]) {
    for entry in entries {
        let path = entry.title.path.get_untracked();
        let filename = path_to_html_filename(&path);
        let page_title = html_page_title(&path);
        let body_html = entry_to_html_body(*entry);
        let document = build_html_document(EXPORT_TEMPLATE, &page_title, &body_html);

        if let Err(err) = download_text_file(&filename, &document, "text/html;charset=utf-8") {
            web_sys::console::error_1(&format!("Export failed for {filename}: {err:?}").into());
        }
    }
}

/// Prompt the browser to save each editor entry as a markdown file.
pub fn export_entries_as_markdown(entries: &[EditorEntry]) {
    for entry in entries {
        let path = entry.title.path.get_untracked();
        let filename = path_to_markdown_filename(&path);
        let content = entry_to_markdown(*entry);

        if let Err(err) = download_text_file(&filename, &content, "text/markdown;charset=utf-8") {
            web_sys::console::error_1(&format!("Export failed for {filename}: {err:?}").into());
        }
    }
}

/// Convert each editor entry to PDF and download the files.
pub fn export_entries_as_pdf(entries: &[EditorEntry]) -> Vec<String> {
    let mut errors = Vec::new();
    for entry in entries {
        let path = entry.title.path.get_untracked();
        let filename = path_to_pdf_filename(&path);
        let page_title = html_page_title(&path);
        let body_html = entry_to_pdf_body(*entry);
        let document = build_html_document(EXPORT_PDF_TEMPLATE, &page_title, &body_html);

        match html_to_pdf_bytes(&document) {
            Ok(bytes) => {
                if let Err(err) = download_bytes_file(&filename, &bytes, "application/pdf") {
                    let msg = format!("PDF export failed for {filename}: {err:?}");
                    web_sys::console::error_1(&msg.clone().into());
                    errors.push(msg);
                }
            }
            Err(e) => {
                let msg = format!("PDF conversion failed for {filename}: {e}");
                web_sys::console::error_1(&msg.clone().into());
                errors.push(msg);
            }
        }
    }
    errors
}

/// Serialize an editor entry to markdown, including front matter when present.
fn entry_to_markdown(entry: EditorEntry) -> String {
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

/// Render an editor entry's body HTML for browser export.
fn entry_to_html_body(entry: EditorEntry) -> String {
    entry_body_html(entry, render_markdown)
}

/// Render an editor entry's body HTML for PDF export.
fn entry_to_pdf_body(entry: EditorEntry) -> String {
    entry_body_html(entry, render_markdown_for_pdf)
}

/// Render front matter and markdown body with the given markdown renderer.
fn entry_body_html(entry: EditorEntry, render: fn(&str) -> String) -> String {
    let mut body = String::new();

    if let Some(fm) = entry.front_matter.get_untracked() {
        let raw = fm.raw.get_untracked();
        if !raw.is_empty() {
            body.push_str("<section class=\"frontmatter\"><h2>Metadata</h2><pre>");
            body.push_str(&escape_html(&raw));
            body.push_str("</pre></section>");
        }
    }

    body.push_str(&render(&entry.content.text.get_untracked()));
    body
}

/// Fill an export HTML template with a title and body.
fn build_html_document(template: &str, title: &str, body_html: &str) -> String {
    template
        .replace("{{TITLE}}", &escape_html(title))
        .replace("{{BODY}}", body_html)
}

/// Trigger a browser download of a text file.
fn download_text_file(filename: &str, content: &str, mime_type: &str) -> Result<(), JsValue> {
    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(content));

    let props = BlobPropertyBag::new();
    props.set_type(mime_type);

    let blob = Blob::new_with_str_sequence_and_options(&parts, &props)?;
    download_blob(filename, &blob)
}

/// Trigger a browser download of a binary file.
fn download_bytes_file(filename: &str, bytes: &[u8], mime_type: &str) -> Result<(), JsValue> {
    let uint8 = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&uint8);

    let props = BlobPropertyBag::new();
    props.set_type(mime_type);

    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &props)?;
    download_blob(filename, &blob)
}

/// Trigger a browser download from a `Blob`.
fn download_blob(filename: &str, blob: &Blob) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or(JsValue::NULL)?;
    let document = window.document().ok_or(JsValue::NULL)?;

    let url = web_sys::Url::create_object_url_with_blob(blob)?;

    let anchor = document
        .create_element("a")?
        .dyn_into::<HtmlAnchorElement>()?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    web_sys::Url::revoke_object_url(&url)?;
    Ok(())
}
