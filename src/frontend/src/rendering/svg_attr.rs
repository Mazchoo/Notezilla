//! Reads and removes attributes on a single SVG element string.

use crate::constants::{PDF_CONTENT_HEIGHT_PX, PDF_CONTENT_WIDTH_PX};

/// Return the quoted value of attribute `name` in the SVG element `tag`.
pub fn svg_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let bytes = tag.as_bytes();
    let mut at = 0;
    while let Some(rel) = tag.get(at..)?.find(name) {
        let start = at + rel;
        at = start + name.len();
        if !starts_attribute_name(bytes, start) {
            continue;
        }
        if let Some((from, to)) = quoted_value_span(bytes, at) {
            return Some(&tag[from..to]);
        }
    }
    None
}

/// Return the value of attribute `name` parsed as an `f64`.
pub fn svg_attr_f64(tag: &str, name: &str) -> Option<f64> {
    svg_attr(tag, name)?.trim().parse().ok()
}

/// Return `element` with attribute `name` and its value removed.
pub fn strip_svg_attr(element: &str, name: &str) -> String {
    let Some(value) = svg_attr(element, name) else {
        return element.to_string();
    };
    element
        .replace(&format!("{name}=\"{value}\""), "")
        .replace(&format!("{name}='{value}'"), "")
}

/// Scale the root `width` / `height` of `svg` down to fit one PDF page.
///
/// Ironpress sizes an inline SVG from these attributes and ignores CSS
/// `max-width`. It then shrinks the whole document to its widest element, so
/// one oversized diagram would make every page tiny. The `viewBox` is kept,
/// so the drawing scales with the box.
pub fn fit_svg_to_pdf_page(svg: &str) -> String {
    let Some(tag_end) = svg.find('>') else {
        return svg.to_string();
    };
    let (tag, rest) = svg.split_at(tag_end);
    let (Some(width), Some(height)) = (svg_attr_f64(tag, "width"), svg_attr_f64(tag, "height"))
    else {
        return svg.to_string();
    };
    let scale = (PDF_CONTENT_WIDTH_PX / width).min(PDF_CONTENT_HEIGHT_PX / height);
    if scale >= 1.0 {
        return svg.to_string();
    }
    let (width, height) = (width * scale, height * scale);
    let tag = strip_svg_attr(&strip_svg_attr(tag, "width"), "height");
    format!("{tag} width=\"{width}\" height=\"{height}\"{rest}")
}

/// Return whether index `at` starts an attribute name rather than a name suffix.
///
/// Rejects `end` inside `marker-end`, `x` inside `rx`, and a name inside a quoted value.
fn starts_attribute_name(tag: &[u8], at: usize) -> bool {
    if at == 0 {
        return true;
    }
    let before = tag[at - 1];
    !(before.is_ascii_alphanumeric() || matches!(before, b'-' | b'_' | b':' | b'"' | b'\''))
}

/// Return the byte span of the quoted value that follows `=` at or after `at`.
fn quoted_value_span(tag: &[u8], at: usize) -> Option<(usize, usize)> {
    let mut i = at;
    while tag.get(i)?.is_ascii_whitespace() {
        i += 1;
    }
    if *tag.get(i)? != b'=' {
        return None;
    }
    i += 1;
    while tag.get(i)?.is_ascii_whitespace() {
        i += 1;
    }
    let quote = *tag.get(i)?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    i += 1;
    let from = i;
    while *tag.get(i)? != quote {
        i += 1;
    }
    Some((from, i))
}

#[cfg(test)]
mod tests {
    use super::{fit_svg_to_pdf_page, strip_svg_attr, svg_attr, svg_attr_f64};
    use crate::constants::{PDF_CONTENT_HEIGHT_PX, PDF_CONTENT_WIDTH_PX};

    #[test]
    /// Assert a wide SVG is scaled to the page width, keeping its aspect and viewBox.
    fn wide_svg_is_scaled_to_page_width() {
        let width = PDF_CONTENT_WIDTH_PX * 2.0;
        let svg = format!(
            r#"<svg width="{width}" height="100" viewBox="0 0 {width} 100"><rect width="5"/></svg>"#
        );
        assert_eq!(
            fit_svg_to_pdf_page(&svg),
            format!(
                r#"<svg   viewBox="0 0 {width} 100" width="{PDF_CONTENT_WIDTH_PX}" height="50"><rect width="5"/></svg>"#
            )
        );
    }

    #[test]
    /// Assert an SVG taller than the page is scaled to the page height.
    fn tall_svg_is_scaled_to_page_height() {
        let height = PDF_CONTENT_HEIGHT_PX * 2.0;
        let svg =
            format!(r#"<svg width="100" height="{height}" viewBox="0 0 100 {height}"></svg>"#);
        assert_eq!(
            fit_svg_to_pdf_page(&svg),
            format!(
                r#"<svg   viewBox="0 0 100 {height}" width="50" height="{PDF_CONTENT_HEIGHT_PX}"></svg>"#
            )
        );
    }

    #[test]
    /// Assert an SVG that fits the page, or has no size, is left unchanged.
    fn fitting_or_unsized_svg_is_unchanged() {
        for svg in [
            r#"<svg width="100" height="50" viewBox="0 0 100 50"></svg>"#,
            r#"<svg viewBox="0 0 9000 50"></svg>"#,
        ] {
            assert_eq!(fit_svg_to_pdf_page(svg), svg);
        }
    }

    #[test]
    /// Assert a double-quoted attribute value is read.
    fn reads_double_quoted_value() {
        assert_eq!(svg_attr(r#"<text x="10" y="20""#, "x"), Some("10"));
    }

    #[test]
    /// Assert a single-quoted attribute value is read.
    fn reads_single_quoted_value() {
        assert_eq!(svg_attr("<text x='10'", "x"), Some("10"));
    }

    #[test]
    /// Assert whitespace around `=` does not prevent a read.
    fn reads_value_with_spaces_around_equals() {
        assert_eq!(svg_attr(r#"<text x = "10""#, "x"), Some("10"));
    }

    #[test]
    /// Assert a missing attribute returns `None`.
    fn missing_attribute_is_none() {
        assert_eq!(svg_attr(r#"<text y="20""#, "x"), None);
    }

    #[test]
    /// Assert an attribute name inside another value is not returned.
    fn does_not_read_name_inside_another_value() {
        assert_eq!(svg_attr(r#"<text y="x='1'" x="2""#, "x"), Some("2"));
    }

    #[test]
    /// Assert `x` does not match the `x` inside `rx`.
    fn does_not_match_name_suffix_after_letter() {
        assert_eq!(svg_attr(r#"<ellipse rx="5" x="1""#, "x"), Some("1"));
    }

    #[test]
    /// Assert `end` does not match the `end` inside `marker-end`.
    fn does_not_match_name_suffix_after_hyphen() {
        assert_eq!(svg_attr(r#"<path marker-end="url(#a)""#, "end"), None);
    }

    #[test]
    /// Assert a hyphenated attribute name is read in full.
    fn reads_hyphenated_name() {
        assert_eq!(
            svg_attr(r#"<text font-size="14px""#, "font-size"),
            Some("14px")
        );
    }

    #[test]
    /// Assert a name matched without a following `=` is skipped for a later match.
    fn skips_name_without_value_and_keeps_scanning() {
        assert_eq!(
            svg_attr(r##"<path stroke-width="2" stroke="#fff""##, "stroke"),
            Some("#fff")
        );
    }

    #[test]
    /// Assert a numeric value is parsed as `f64`.
    fn parses_numeric_value() {
        assert_eq!(svg_attr_f64(r#"<text y="20.5""#, "y"), Some(20.5));
    }

    #[test]
    /// Assert a non-numeric value yields `None` rather than a parse panic.
    fn non_numeric_value_is_not_a_number() {
        assert_eq!(svg_attr_f64(r#"<text y="auto""#, "y"), None);
    }

    #[test]
    /// Assert an unterminated quote yields `None` rather than a panic.
    fn unterminated_quote_is_none() {
        assert_eq!(svg_attr(r#"<text x="10"#, "x"), None);
    }

    #[test]
    /// Assert stripping removes the attribute and keeps the rest of the element.
    fn strips_attribute() {
        let stripped = strip_svg_attr(r#"<path d="M0 0" marker-end="url(#a)" />"#, "marker-end");
        assert!(!stripped.contains("marker-end"), "{stripped}");
        assert!(stripped.contains(r#"d="M0 0""#), "{stripped}");
    }

    #[test]
    /// Assert stripping a missing attribute returns the element unchanged.
    fn strips_missing_attribute_unchanged() {
        let element = r#"<path d="M0 0" />"#;
        assert_eq!(strip_svg_attr(element, "marker-end"), element);
    }
}
