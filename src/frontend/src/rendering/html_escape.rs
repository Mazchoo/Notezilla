//! Escapes text for insertion into HTML.

/// Escape `&`, `<`, `>`, `"`, and `'` for safe insertion into HTML text or attributes.
pub(crate) fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Escape markup characters in already-escaped SVG text or attribute values.
///
/// Like [`escape_html`], but an `&` that already starts a character reference
/// (`&lt;`, `&#60;`, `&#x3c;`) is kept, so upstream-escaped text is not
/// double-escaped.
pub(crate) fn escape_svg_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (at, c) in s.char_indices() {
        match c {
            '&' if starts_char_reference(&s[at + 1..]) => out.push('&'),
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Return whether `rest` (the text after an `&`) completes a character reference.
fn starts_char_reference(rest: &str) -> bool {
    let Some(end) = rest.find(';') else {
        return false;
    };
    let body = &rest[..end];
    if let Some(hex) = body.strip_prefix("#x").or_else(|| body.strip_prefix("#X")) {
        return !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit());
    }
    if let Some(dec) = body.strip_prefix('#') {
        return !dec.is_empty() && dec.chars().all(|c| c.is_ascii_digit());
    }
    body.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && body.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Escape `s` so it can sit inside `<!-- … -->` without closing the comment.
pub(crate) fn escape_html_comment(s: &str) -> String {
    escape_html(s).replace("--", "- -")
}

#[cfg(test)]
mod tests {
    use super::{escape_html, escape_svg_text};

    #[test]
    /// Assert existing character references survive SVG text escaping.
    fn svg_text_keeps_character_references() {
        assert_eq!(escape_svg_text("a&lt;b&#60;&#x3C;&amp;"), "a&lt;b&#60;&#x3C;&amp;");
    }

    #[test]
    /// Assert raw markup and bare `&` are escaped in SVG text.
    fn svg_text_escapes_raw_markup() {
        assert_eq!(
            escape_svg_text(r#"<b>"x" & y; &#;"#),
            "&lt;b&gt;&quot;x&quot; &amp; y; &amp;#;"
        );
    }

    #[test]
    /// Assert markup characters are replaced by entities.
    fn escapes_markup_characters() {
        assert_eq!(
            escape_html(r#"<a href="x">&</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
        );
    }

    #[test]
    /// Assert `&` is escaped before the entities it introduces.
    fn escapes_ampersand_only_once() {
        assert_eq!(escape_html("<"), "&lt;");
        assert_eq!(escape_html("&lt;"), "&amp;lt;");
    }

    #[test]
    /// Assert text without markup characters is unchanged.
    fn plain_text_is_unchanged() {
        assert_eq!(escape_html("x = 1"), "x = 1");
    }

    #[test]
    /// Assert `'` is encoded for attribute use.
    fn escapes_apostrophe() {
        assert_eq!(escape_html("it's"), "it&#39;s");
    }
}
