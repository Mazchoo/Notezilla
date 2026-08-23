//! Escapes text for insertion into HTML.

/// Escape `&`, `<`, `>`, and `"` for safe insertion into HTML text or attributes.
pub(crate) fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::escape_html;

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
}
