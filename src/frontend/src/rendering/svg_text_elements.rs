//! Rewrites every `<text>` element of an SVG document.

/// Return `svg` with each `<text>…</text>` element replaced by `rewrite`.
///
/// A `<text>` element without a closing tag is copied through unchanged.
pub fn rewrite_text_elements(svg: &str, mut rewrite: impl FnMut(&str) -> String) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text") {
        out.push_str(&rest[..start]);
        let Some(close_rel) = rest[start..].find("</text>") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let end = start + close_rel + "</text>".len();
        out.push_str(&rewrite(&rest[start..end]));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::rewrite_text_elements;

    #[test]
    /// Assert each text element is passed to the rewriter with its tags.
    fn passes_whole_element_to_rewriter() {
        let mut seen = Vec::new();
        rewrite_text_elements(
            r#"<g><text x="1">A</text><text x="2">B</text></g>"#,
            |element| {
                seen.push(element.to_string());
                String::new()
            },
        );
        assert_eq!(
            seen,
            vec![r#"<text x="1">A</text>"#, r#"<text x="2">B</text>"#]
        );
    }

    #[test]
    /// Assert markup outside text elements is preserved verbatim.
    fn keeps_surrounding_markup() {
        let out = rewrite_text_elements("<g><text>A</text></g>", |_| "<t/>".to_string());
        assert_eq!(out, "<g><t/></g>");
    }

    #[test]
    /// Assert an SVG without text elements is returned unchanged.
    fn svg_without_text_is_unchanged() {
        let svg = r#"<svg><rect width="1"/></svg>"#;
        assert_eq!(rewrite_text_elements(svg, |_| String::new()), svg);
    }

    #[test]
    /// Assert a `<text>` element with no closing tag is copied once, not duplicated.
    fn unterminated_element_is_copied_once() {
        let svg = "<g><text x=\"1\">A";
        assert_eq!(rewrite_text_elements(svg, |_| "X".to_string()), svg);
    }

    #[test]
    /// Assert a nested `<textPath>` does not end the enclosing text element.
    fn nested_text_path_does_not_close_element() {
        let svg = r##"<text><textPath href="#p">A</textPath></text>"##;
        let mut seen = Vec::new();
        rewrite_text_elements(svg, |element| {
            seen.push(element.to_string());
            String::new()
        });
        assert_eq!(seen, vec![svg]);
    }
}
