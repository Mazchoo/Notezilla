//! Converts an export HTML document to PDF bytes.

mod color_from_hex;
mod content;
mod formatting;
mod list_kind;
mod objects;
mod radical;

pub(crate) use color_from_hex::pdf_rgb_operator;
pub(super) use list_kind::{pdf_list_item_open, PdfListKind};

use formatting::recolor_math_operators;

/// Convert a complete HTML document string to PDF bytes.
pub fn html_to_pdf_bytes(html: &str) -> Result<Vec<u8>, String> {
    // Uncompressed page streams so math fill/stroke operators can be rewritten.
    let pdf = ironpress::HtmlConverter::new()
        .compress(false)
        .convert(html)
        .map_err(|e| e.to_string())?;
    recolor_math_operators(&pdf)
}
