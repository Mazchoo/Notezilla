//! Renders the placeholder shown for a markdown image that cannot be served.

use super::{Render, RenderPdf};
use crate::constants::IMAGE_MISSING_SVG;
use crate::rendering::escape_html;

/// Placeholder render for a markdown image reference.
pub struct MissingImageRender;

impl Render for MissingImageRender {
    /// Return placeholder HTML labelled with the image's alt text.
    ///
    /// `source` is the alt text of the image, not an image path: no markdown
    /// image is served yet, so the reference target is never read.
    fn render(&self, source: &str) -> String {
        let svg = placeholder_svg();
        let label = escape_html(source);
        format!(r#"<span class="md-image-missing" role="img" aria-label="{label}">{svg}</span>"#)
    }
}

impl RenderPdf for MissingImageRender {}

/// Return the placeholder SVG element without its XML prologue.
fn placeholder_svg() -> &'static str {
    IMAGE_MISSING_SVG
        .find("<svg")
        .map(|at| &IMAGE_MISSING_SVG[at..])
        .unwrap_or(IMAGE_MISSING_SVG)
        .trim()
}

#[cfg(test)]
mod tests {
    use super::{placeholder_svg, MissingImageRender, Render, RenderPdf};

    #[test]
    /// Assert the placeholder SVG starts at its root element.
    fn placeholder_svg_starts_at_root_element() {
        let svg = placeholder_svg();
        assert!(svg.starts_with("<svg"), "{svg}");
        assert!(svg.ends_with("</svg>"), "{svg}");
    }

    #[test]
    /// Assert the placeholder is a labelled image span holding the SVG.
    fn renders_labelled_image_span() {
        let html = MissingImageRender.render("a diagram");
        assert!(html.contains(r#"class="md-image-missing""#), "{html}");
        assert!(html.contains(r#"role="img""#), "{html}");
        assert!(html.contains(r#"aria-label="a diagram""#), "{html}");
        assert!(html.contains("<svg"), "{html}");
    }

    #[test]
    /// Assert alt text is escaped so it cannot close the label attribute.
    fn escapes_alt_text() {
        let html = MissingImageRender.render(r#"a" onload="x"#);
        assert!(html.contains("&quot;"), "{html}");
        assert!(!html.contains(r#"onload="x""#), "{html}");
    }

    #[test]
    /// Assert PDF export reuses the editor HTML.
    fn pdf_html_matches_editor_html() {
        assert_eq!(
            MissingImageRender.render_pdf("alt"),
            MissingImageRender.render("alt")
        );
    }
}
