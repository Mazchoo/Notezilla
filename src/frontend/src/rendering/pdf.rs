/// Convert a complete HTML document string to PDF bytes.
///
/// Uses ironpress's layout engine so the same path runs natively and in WASM.
/// Callers must pass a self-contained document (inline CSS, inline SVG); remote
/// stylesheets and images are not fetched.
pub fn html_to_pdf_bytes(html: &str) -> Result<Vec<u8>, String> {
    ironpress::html_to_pdf(html).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::html_to_pdf_bytes;
    use crate::rendering::render_markdown;

    fn assert_pdf(bytes: &[u8]) {
        assert!(
            bytes.starts_with(b"%PDF"),
            "expected PDF header, got {} bytes starting {:?}",
            bytes.len(),
            bytes.get(..8)
        );
        assert!(bytes.len() > 100, "PDF too small: {} bytes", bytes.len());
    }

    #[test]
    fn simple_html_converts_to_pdf() {
        let pdf = html_to_pdf_bytes("<h1>Hello</h1><p>World</p>").unwrap();
        assert_pdf(&pdf);
    }

    #[test]
    fn inline_svg_converts_to_pdf() {
        let html = r##"<h1>Diagram</h1>
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80" viewBox="0 0 120 80">
  <rect x="10" y="10" width="100" height="60" fill="#cba6f7"/>
  <text x="60" y="48" text-anchor="middle" font-size="16">SVG</text>
</svg>"##;
        let pdf = html_to_pdf_bytes(html).unwrap();
        assert_pdf(&pdf);
    }

    #[test]
    fn markdown_graphviz_svg_converts_to_pdf() {
        let body = render_markdown("```graphviz\ndigraph { A -> B }\n```\n");
        assert!(
            body.contains("<svg"),
            "graphviz HTML should contain SVG: {body}"
        );
        let pdf = html_to_pdf_bytes(&format!("<h1>Graph</h1>{body}")).unwrap();
        assert_pdf(&pdf);
    }

    #[test]
    fn markdown_mermaid_svg_converts_to_pdf() {
        let body = render_markdown("```mermaid\ngraph LR\n    A --> B\n```\n");
        assert!(
            body.contains("<svg"),
            "mermaid HTML should contain SVG: {body}"
        );
        let pdf = html_to_pdf_bytes(&format!("<h1>Flow</h1>{body}")).unwrap();
        assert_pdf(&pdf);
    }
}
