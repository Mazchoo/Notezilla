use crate::models::block::EditorEntry;
use crate::rendering::render_markdown;
use leptos::prelude::GetUntracked;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement};

const EXPORT_TEMPLATE: &str = include_str!("../../../templates/export.html");

/// Prompts the browser to save each editor entry as a standalone HTML file.
pub fn export_entries_as_html(entries: &[EditorEntry]) {
    for entry in entries {
        let path = entry.title.path.get_untracked();
        let filename = path_to_html_filename(&path);
        let page_title = html_page_title(&path);
        let body_html = entry_to_html_body(*entry);
        let document = build_html_document(&page_title, &body_html);

        if let Err(err) = download_text_file(&filename, &document, "text/html;charset=utf-8") {
            web_sys::console::error_1(&format!("Export failed for {filename}: {err:?}").into());
        }
    }
}

/// Prompts the browser to save each editor entry as a markdown file.
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

fn path_to_markdown_filename(path: &str) -> String {
    let name = path
        .rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path);
    if name.ends_with(".md") || name.ends_with(".markdown") {
        name.to_string()
    } else {
        format!("{name}.md")
    }
}

fn path_to_html_filename(path: &str) -> String {
    let name = path
        .rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path);
    let stem = name
        .strip_suffix(".markdown")
        .or_else(|| name.strip_suffix(".md"))
        .unwrap_or(name);
    format!("{stem}.html")
}

fn html_page_title(path: &str) -> String {
    path_to_html_filename(path)
        .strip_suffix(".html")
        .unwrap_or("export")
        .to_string()
}

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

fn entry_to_html_body(entry: EditorEntry) -> String {
    let mut body = String::new();

    if let Some(fm) = entry.front_matter.get_untracked() {
        let raw = fm.raw.get_untracked();
        if !raw.is_empty() {
            body.push_str("<section class=\"frontmatter\"><h2>Metadata</h2><pre>");
            body.push_str(&escape_html(&raw));
            body.push_str("</pre></section>");
        }
    }

    let markdown_html = render_markdown(&entry.content.text.get_untracked());
    body.push_str(&markdown_html);
    body
}

fn build_html_document(title: &str, body_html: &str) -> String {
    EXPORT_TEMPLATE
        .replace("{{TITLE}}", &escape_html(title))
        .replace("{{BODY}}", body_html)
}

fn download_text_file(filename: &str, content: &str, mime_type: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or(JsValue::NULL)?;
    let document = window.document().ok_or(JsValue::NULL)?;

    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(content));

    let props = BlobPropertyBag::new();
    props.set_type(mime_type);

    let blob = Blob::new_with_str_sequence_and_options(&parts, &props)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;

    let anchor = document
        .create_element("a")?
        .dyn_into::<HtmlAnchorElement>()?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    web_sys::Url::revoke_object_url(&url)?;
    Ok(())
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
