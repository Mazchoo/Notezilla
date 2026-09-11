mod export_interface;
mod html;
mod markdown;
mod pdf;
mod svg;

use crate::components::toast::show_error_toast;
use crate::info_messages::{export_progress_label, EXPORT_METADATA_HEADING};
use crate::models::block::EditorEntry;
use crate::rendering::escape_html;
use export_interface::ExportOne;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement};

pub use html::export_entries_as_html;
pub use markdown::export_entries_as_markdown;
pub use pdf::export_entries_as_pdf;
pub use svg::export_entries_as_svg_diagrams;

/// Generate and download each entry, showing a spinner and yielding so it can paint.
fn run_export(
    entries: Vec<EditorEntry>,
    progress: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
    filename_for: fn(&str) -> String,
    export_one: impl ExportOne,
) {
    if progress.get_untracked().is_some() || entries.is_empty() {
        return;
    }
    let total = entries.len();
    let first_name = filename_for(&entries[0].title.path.get_untracked());
    progress.set(Some(export_progress_label(&first_name, 0, total)));

    spawn_local(async move {
        let mut errors = Vec::new();
        for (i, entry) in entries.into_iter().enumerate() {
            let name = filename_for(&entry.title.path.get_untracked());
            if i > 0 {
                progress.set(Some(export_progress_label(&name, i, total)));
            }
            yield_for_paint().await;
            if let Some(err) = export_one(entry, &name) {
                errors.push(err);
            }
        }
        progress.set(None);
        if !errors.is_empty() {
            show_error_toast(error_toast, errors.join("\n"));
        }
    });
}

/// Yield so the overlay can paint before the next blocking WASM conversion.
async fn yield_for_paint() {
    let timeout = js_sys::Promise::new(&mut |resolve, _reject| {
        let Some(window) = web_sys::window() else {
            let _ = resolve.call0(&JsValue::UNDEFINED);
            return;
        };
        if window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
            .is_err()
        {
            let _ = resolve.call0(&JsValue::UNDEFINED);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(timeout).await;

    let frame = js_sys::Promise::new(&mut |resolve, _reject| {
        let Some(window) = web_sys::window() else {
            let _ = resolve.call0(&JsValue::UNDEFINED);
            return;
        };
        if window.request_animation_frame(&resolve).is_err() {
            let _ = resolve.call0(&JsValue::UNDEFINED);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(frame).await;
}

/// Log a download failure and return the toast text.
fn log_export_error(message: String) -> String {
    web_sys::console::error_1(&message.clone().into());
    message
}

/// Render front matter and markdown body with the given markdown renderer.
fn entry_body_html(entry: EditorEntry, render: fn(&str) -> String) -> String {
    let mut body = String::new();

    if let Some(fm) = entry.front_matter.get_untracked() {
        let raw = fm.raw.get_untracked();
        if !raw.is_empty() {
            body.push_str(&format!(
                "<section class=\"frontmatter\"><h2>{EXPORT_METADATA_HEADING}</h2><pre>"
            ));
            body.push_str(&escape_html(&raw));
            body.push_str("</pre></section>");
        }
    }

    body.push_str(&render(&entry.content.text.get_untracked()));
    body
}

/// Fill an export HTML template with a title and body.
fn build_html_document(template: &str, title: &str, body_html: &str) -> String {
    crate::theme::fill_export_template(template, &escape_html(title), body_html)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::info_messages::EXPORT_METADATA_HEADING;

    #[test]
    /// Assert template placeholders are replaced and the title is HTML-escaped.
    fn build_html_document_fills_title_and_body() {
        let html = build_html_document("<title>{{TITLE}}</title>{{BODY}}", "A & B", "<p>ok</p>");
        assert_eq!(html, "<title>A &amp; B</title><p>ok</p>");
    }

    #[test]
    /// Assert HTML export embeds the active color theme stylesheet.
    fn build_html_document_embeds_the_active_theme() {
        use crate::constants::{DAY_TEXT, EXPORT_TEMPLATE};
        use crate::theme::{ColorTheme, ThemeGuard};

        let night = build_html_document(EXPORT_TEMPLATE, "n", "<p>ok</p>");
        assert!(night.contains(r#"data-theme="dark""#), "{night}");
        assert!(night.contains("--text: #cdd6f4"), "{night}");

        let _guard = ThemeGuard::set(ColorTheme::Day);
        let day = build_html_document(EXPORT_TEMPLATE, "n", "<p>ok</p>");
        assert!(day.contains(r#"data-theme="light""#), "{day}");
        assert!(day.contains(&format!("--text: {DAY_TEXT}")), "{day}");
        assert!(day.contains("--bg-2: #eff1f5"), "{day}");
    }

    #[test]
    /// Assert body HTML prepends escaped front matter and the rendered markdown.
    fn entry_body_html_prepends_escaped_front_matter() {
        use crate::models::block::{EditorEntry, FrontMatterBlock};
        use leptos::prelude::{Owner, Set};

        let owner = Owner::new();
        owner.with(|| {
            let entry = EditorEntry::new("./a.md", "ignored");
            entry
                .front_matter
                .set(Some(FrontMatterBlock::new("title: <x>")));
            let html = entry_body_html(entry, |_| "<p>md</p>".into());
            assert!(
                html.starts_with(&format!(
                    "<section class=\"frontmatter\"><h2>{EXPORT_METADATA_HEADING}</h2><pre>"
                )),
                "{html}"
            );
            assert!(html.contains("title: &lt;x&gt;"), "{html}");
            assert!(html.ends_with("</pre></section><p>md</p>"), "{html}");

            let no_fm = EditorEntry::new("./b.md", "ignored");
            assert_eq!(entry_body_html(no_fm, |_| "<p>md</p>".into()), "<p>md</p>");
        });
    }
}
