//! Reads and removes attributes on a single SVG element string.

/// Return the quoted value of attribute `name` in the SVG element `tag`.
pub fn svg_attr(tag: &str, name: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut at = 0;
    while let Some(rel) = tag.get(at..)?.find(name) {
        let start = at + rel;
        at = start + name.len();
        if !starts_attribute_name(bytes, start) {
            continue;
        }
        if let Some((from, to)) = quoted_value_span(bytes, at) {
            return Some(tag[from..to].to_string());
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

/// Return whether index `at` starts an attribute name rather than a name suffix.
///
/// Rejects `end` inside `marker-end` and `x` inside `rx`.
fn starts_attribute_name(tag: &[u8], at: usize) -> bool {
    if at == 0 {
        return true;
    }
    let before = tag[at - 1];
    !(before.is_ascii_alphanumeric() || matches!(before, b'-' | b'_' | b':'))
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
    use super::{strip_svg_attr, svg_attr, svg_attr_f64};

    #[test]
    /// Assert a double-quoted attribute value is read.
    fn reads_double_quoted_value() {
        assert_eq!(
            svg_attr(r#"<text x="10" y="20""#, "x"),
            Some("10".to_string())
        );
    }

    #[test]
    /// Assert a single-quoted attribute value is read.
    fn reads_single_quoted_value() {
        assert_eq!(svg_attr("<text x='10'", "x"), Some("10".to_string()));
    }

    #[test]
    /// Assert whitespace around `=` does not prevent a read.
    fn reads_value_with_spaces_around_equals() {
        assert_eq!(svg_attr(r#"<text x = "10""#, "x"), Some("10".to_string()));
    }

    #[test]
    /// Assert a missing attribute returns `None`.
    fn missing_attribute_is_none() {
        assert_eq!(svg_attr(r#"<text y="20""#, "x"), None);
    }

    #[test]
    /// Assert `x` does not match the `x` inside `rx`.
    fn does_not_match_name_suffix_after_letter() {
        assert_eq!(
            svg_attr(r#"<ellipse rx="5" x="1""#, "x"),
            Some("1".to_string())
        );
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
            Some("14px".to_string())
        );
    }

    #[test]
    /// Assert a name matched without a following `=` is skipped for a later match.
    fn skips_name_without_value_and_keeps_scanning() {
        assert_eq!(
            svg_attr(r##"<path stroke-width="2" stroke="#fff""##, "stroke"),
            Some("#fff".to_string())
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
