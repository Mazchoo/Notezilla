//! Converts an export HTML document to PDF bytes.

mod color_from_hex;
mod content;
mod formatting;
mod list_kind;
mod objects;
mod radical;

pub(crate) use color_from_hex::pdf_rgb_operator;
pub(super) use list_kind::{pdf_list_item_open, PdfListKind};

use crate::constants::PDF_EMBEDDED_FONTS;
use formatting::recolor_math_operators;
use ironpress::HtmlConverter;

/// Convert a complete HTML document string to PDF bytes.
pub fn html_to_pdf_bytes(html: &str) -> Result<Vec<u8>, String> {
    // Uncompressed page streams so math fill/stroke operators can be rewritten.
    // Embedded font files are always Flate-compressed, so the rewrite skips them.
    let converter = PDF_EMBEDDED_FONTS.iter().fold(
        HtmlConverter::new().compress(false),
        |converter, (name, ttf)| converter.add_font(name, ttf.to_vec()),
    );
    let pdf = converter.convert(html).map_err(|e| e.to_string())?;
    recolor_math_operators(&pdf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::EXPORT_PDF_TEMPLATE;

    /// Family of the regular face, which the export template sets on the body.
    const PDF_BODY_FONT_FAMILY: &str = PDF_EMBEDDED_FONTS[0].0;

    #[test]
    /// Assert regular, bold, italic and bold italic Liberation Sans are embedded.
    fn embeds_every_liberation_sans_face() {
        let html = format!(
            "<html><head><style>body {{ font-family: \"{PDF_BODY_FONT_FAMILY}\"; }}</style></head><body>\
             <p>Regular <b>Bold</b> <i>Italic</i> <b><i>BoldItalic</i></b></p>\
             </body></html>"
        );
        let pdf = html_to_pdf_bytes(&html).expect("pdf");
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/FontFile2"), "no embedded TrueType font");
        let base_fonts: Vec<&str> = text
            .split("/BaseFont /")
            .skip(1)
            .filter_map(|rest| rest.split([' ', '/', '>']).next())
            .map(|name| name.rsplit('+').next().unwrap_or(name))
            .collect();
        for face in [
            "LiberationSans",
            "LiberationSans-Bold",
            "LiberationSans-Italic",
            "LiberationSans-BoldItalic",
        ] {
            assert!(base_fonts.contains(&face), "{face} not in {base_fonts:?}");
        }
    }

    #[test]
    /// Assert the PDF export template sets the embedded body font family.
    fn template_uses_embedded_body_font() {
        assert!(
            EXPORT_PDF_TEMPLATE.contains(&format!("font-family: \"{PDF_BODY_FONT_FAMILY}\"")),
            "{EXPORT_PDF_TEMPLATE}"
        );
    }
}
