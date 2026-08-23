//! Converts an export HTML document to PDF bytes.

mod color_from_hex;
mod content;
mod list_kind;
mod objects;
mod radical;

pub(crate) use color_from_hex::pdf_rgb_operator;
pub(super) use list_kind::{pdf_list_item_open, PdfListKind};

use crate::constants::PDF_HEADER;
use content::recolor_content;
use objects::{
    find_subslice, find_xref_offset, parse_catalog_id, rewrite_length, serialize_pdf, split_objects,
};
use radical::correct_radicals;

/// Convert a complete HTML document string to PDF bytes.
pub fn html_to_pdf_bytes(html: &str) -> Result<Vec<u8>, String> {
    // Uncompressed page streams so math fill/stroke operators can be rewritten.
    let pdf = ironpress::HtmlConverter::new()
        .compress(false)
        .convert(html)
        .map_err(|e| e.to_string())?;
    recolor_math_operators(&pdf)
}

/// Recolor black math fill/stroke operators to the page text color.
///
/// A document that does not start with the expected ironpress header is
/// returned unchanged rather than rejected.
fn recolor_math_operators(pdf: &[u8]) -> Result<Vec<u8>, String> {
    let Some(body) = pdf.strip_prefix(PDF_HEADER) else {
        return Ok(pdf.to_vec());
    };
    let xref_at = find_xref_offset(body)?;
    let catalog_id = parse_catalog_id(&body[xref_at..])?;
    let objects = split_objects(&body[..xref_at])?;
    let recolored: Vec<Vec<u8>> = objects.into_iter().map(recolor_object).collect();
    Ok(serialize_pdf(&recolored, catalog_id))
}

/// Rewrite the content stream of one PDF object.
///
/// Objects without a stream, and objects whose stream is compressed, are
/// returned unchanged.
fn recolor_object(object: Vec<u8>) -> Vec<u8> {
    let Some(stream_at) = find_subslice(&object, b"\nstream\n") else {
        return object;
    };
    let dict = &object[..stream_at];
    if find_subslice(dict, b"/Filter").is_some() {
        return object;
    }
    let payload_start = stream_at + b"\nstream\n".len();
    let Some(endstream_rel) = find_subslice(&object[payload_start..], b"\nendstream") else {
        return object;
    };
    let payload_end = payload_start + endstream_rel;
    let mut content = object[payload_start..payload_end].to_vec();
    if content.ends_with(b"\n") {
        content.pop();
    }
    // Drop this line with `radical.rs` once ironpress places the radical sign.
    let content = correct_radicals(&content);
    let recolored = recolor_content(&content);

    let mut dict = dict.to_vec();
    rewrite_length(&mut dict, recolored.len());
    let mut out = dict;
    out.extend_from_slice(b"\nstream\n");
    out.extend_from_slice(&recolored);
    out.extend_from_slice(b"\nendstream\nendobj\n");
    out
}

#[cfg(test)]
mod tests {
    use super::{html_to_pdf_bytes, recolor_math_operators, recolor_object};
    use crate::constants::{EXPORT_PDF_TEMPLATE, TEXT_FILL};
    use crate::rendering::{render_markdown, render_markdown_for_pdf};

    /// Assert `bytes` look like a non-trivial PDF document.
    fn assert_pdf(bytes: &[u8]) {
        assert!(
            bytes.starts_with(b"%PDF"),
            "expected PDF header, got {} bytes starting {:?}",
            bytes.len(),
            bytes.get(..8)
        );
        assert!(bytes.len() > 100, "PDF too small: {} bytes", bytes.len());
        assert!(
            bytes.windows(6).any(|w| w == b"%%EOF\n") || bytes.ends_with(b"%%EOF"),
            "PDF missing EOF marker"
        );
    }

    /// Convert an export document with `body` to PDF bytes.
    fn export_to_pdf(body: &str) -> Vec<u8> {
        let document = EXPORT_PDF_TEMPLATE
            .replace("{{TITLE}}", "t")
            .replace("{{BODY}}", body);
        html_to_pdf_bytes(&document).expect("PDF conversion")
    }

    /// Return the content of the text object that paints `glyph`.
    fn text_object_before(pdf: &str, glyph: &str) -> String {
        let at = pdf
            .find(glyph)
            .unwrap_or_else(|| panic!("missing {glyph} in PDF"));
        let bt = pdf[..at]
            .rfind("BT\n")
            .unwrap_or_else(|| panic!("no BT before {glyph}"));
        pdf[bt..at].to_string()
    }

    /// Return the page text fill operator as it appears in a content stream.
    fn text_fill_operator() -> String {
        String::from_utf8_lossy(TEXT_FILL).trim().to_string()
    }

    #[test]
    /// Assert simple HTML converts to a valid PDF.
    fn simple_html_converts_to_pdf() {
        assert_pdf(&html_to_pdf_bytes("<h1>Hello</h1><p>World</p>").unwrap());
    }

    #[test]
    /// Assert inline SVG HTML converts to a valid PDF.
    fn inline_svg_converts_to_pdf() {
        let html = r##"<h1>Diagram</h1>
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80" viewBox="0 0 120 80">
  <rect x="10" y="10" width="100" height="60" fill="#cba6f7"/>
  <text x="60" y="48" text-anchor="middle" font-size="16">SVG</text>
</svg>"##;
        assert_pdf(&html_to_pdf_bytes(html).unwrap());
    }

    #[test]
    /// Assert a document without the ironpress header is passed through.
    fn unexpected_header_is_passed_through() {
        let pdf = b"%PDF-1.7\nnot ironpress output";
        assert_eq!(recolor_math_operators(pdf), Ok(pdf.to_vec()));
    }

    #[test]
    /// Assert a truncated ironpress document reports the missing xref table.
    fn missing_xref_is_reported() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< >>\nendobj\n";
        assert!(recolor_math_operators(pdf).is_err());
    }

    #[test]
    /// Assert an object without a stream is left unchanged.
    fn object_without_stream_is_unchanged() {
        let object = b"1 0 obj\n<< /Type /Catalog >>\nendobj\n".to_vec();
        assert_eq!(recolor_object(object.clone()), object);
    }

    #[test]
    /// Assert a compressed stream is left unchanged.
    fn compressed_stream_is_unchanged() {
        let object = b"1 0 obj\n<< /Filter /FlateDecode /Length 4 >>\nstream\n0 0 0 rg\nendstream\nendobj\n".to_vec();
        assert_eq!(recolor_object(object.clone()), object);
    }

    #[test]
    /// Assert recoloring a stream repaints it and updates its `/Length`.
    fn recolored_stream_length_is_updated() {
        let object = b"1 0 obj\n<< /Length 9 >>\nstream\n0 0 0 rg\nendstream\nendobj\n".to_vec();
        let out = String::from_utf8_lossy(&recolor_object(object)).to_string();
        let fill = text_fill_operator();
        assert!(out.contains(&fill), "{out}");
        assert!(!out.contains("0 0 0 rg"), "{out}");
        assert!(out.contains(&format!("/Length {}", fill.len())), "{out}");
    }

    #[test]
    /// Assert graphviz markdown renders to SVG and converts to PDF.
    fn markdown_graphviz_svg_converts_to_pdf() {
        let body = render_markdown("```graphviz\ndigraph { A -> B }\n```\n");
        assert!(
            body.contains("<svg"),
            "graphviz HTML should contain SVG: {body}"
        );
        assert!(
            body.contains(&format!("fill=\"{}\"", crate::constants::TEXT))
                && body.contains("font-size=\"14px\""),
            "graphviz labels must set fill and font-size as attributes: {body}"
        );
        assert!(
            !body.contains("<tspan"),
            "graphviz labels must not use tspan: {body}"
        );
        let pdf = export_to_pdf(&body);
        assert_pdf(&pdf);
        let text = String::from_utf8_lossy(&pdf);
        let object = text_object_before(&text, "(A)");
        assert!(
            object.contains(&text_fill_operator()),
            "graphviz A must paint with page text fill: {object}"
        );
    }

    #[test]
    /// Assert mermaid markdown renders to SVG and converts to PDF.
    fn markdown_mermaid_svg_converts_to_pdf() {
        let body =
            render_markdown("```mermaid\ngraph LR\n    A[Square Rect] --> B((Circle))\n```\n");
        assert!(
            body.contains("<svg"),
            "mermaid HTML should contain SVG: {body}"
        );
        assert!(
            !body.contains("dominant-baseline") && body.contains("Courier"),
            "mermaid labels must use an alphabetic baseline and Courier: {body}"
        );
        assert!(
            body.contains("<polygon"),
            "mermaid arrows must be polygons: {body}"
        );
        let pdf = export_to_pdf(&body);
        assert_pdf(&pdf);
        let text = String::from_utf8_lossy(&pdf);
        assert!(
            text.contains("Courier") || text.contains("(A)") || text.contains("Square"),
            "PDF should contain mermaid label text"
        );
    }

    #[test]
    /// Assert the PDF export stylesheet sets the editor page background.
    fn pdf_export_stylesheet_sets_page_background() {
        use crate::constants::BG_2;
        assert!(
            EXPORT_PDF_TEMPLATE.contains("background-color: var(--bg-2)"),
            "PDF export stylesheet must set the editor page background"
        );
        assert!(
            EXPORT_PDF_TEMPLATE.contains(&format!("--bg-2: {BG_2}")),
            "PDF export stylesheet must keep the editor background token"
        );
        assert_pdf(&export_to_pdf("<p>Hi</p>"));
    }

    #[test]
    /// Assert PDF math uses `data-math` and converts with page-color fill.
    fn latex_markdown_for_pdf_uses_data_math_and_converts() {
        use crate::constants::TEXT;

        let src = "inline $x_i$ and\n\n$$\nx = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}\n$$\n";
        let html = render_markdown_for_pdf(src);
        assert!(!html.contains("<math"), "{html}");
        assert!(html.contains("data-math="), "{html}");
        assert!(html.contains(&format!(r#"style="color:{TEXT}""#)), "{html}");
        assert!(
            html.contains(r#"\frac{-b \pm \sqrt{b^2-4ac}}{2a}"#),
            "{html}"
        );
        assert!(!html.contains("<em>"), "{html}");
        let pdf = export_to_pdf(&html);
        assert_pdf(&pdf);
        let text = String::from_utf8_lossy(&pdf);
        assert!(
            text.contains("(x)") && text.contains("(b)"),
            "PDF should contain math letters x and b"
        );
        assert!(
            text.contains("Helvetica-Oblique"),
            "italic math letters should use Helvetica-Oblique"
        );
        for glyph in ["(x)", "(b)"] {
            let object = text_object_before(&text, glyph);
            assert!(
                object.contains(&text_fill_operator()),
                "fill must be inside the text object before {glyph}: {object}"
            );
        }
    }

    #[test]
    /// Assert a list-item inline `$\\pi$` converts to PDF without `<li>`.
    fn list_item_inline_pi_converts_to_pdf() {
        let html = render_markdown_for_pdf("- $x = \\pi$\n");
        assert!(html.contains("pdf-li"), "{html}");
        assert!(
            !html.contains("<li>"),
            "PDF lists must not use <li> (ironpress drops inline math): {html}"
        );
        assert!(
            html.contains(r#"class="math-inline""#) && html.contains(r#"data-math="x = \pi""#),
            "{html}"
        );
        assert!(
            !html.contains("math-display"),
            "single-dollar math must stay inline: {html}"
        );
        let pdf = export_to_pdf(&html);
        assert_pdf(&pdf);
        let text = String::from_utf8_lossy(&pdf);
        assert!(
            text.contains("(x)"),
            "PDF should contain italic x from $x = \\pi$: {text}"
        );
        assert!(
            text.contains("/Symbol") && text.contains("(\\160)"),
            "PDF should contain Symbol-encoded pi from $x = \\pi$: {text}"
        );
    }
}
