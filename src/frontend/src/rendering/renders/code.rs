//! Renders a fenced code block with syntect syntax highlighting.

use super::{Render, RenderPdf};
use crate::constants::CODE_THEME;
use crate::rendering::escape_html;
use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Fenced code block render for one fence language token.
pub struct CodeRender {
    language: String,
}

impl CodeRender {
    /// Build a code render for the fence's language token.
    ///
    /// An empty token, or one syntect does not know, renders as plain text.
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
        }
    }

    /// Return the syntect syntax for the language token, else plain text.
    fn syntax(&self) -> &'static SyntaxReference {
        if self.language.is_empty() {
            return SYNTAX_SET.find_syntax_plain_text();
        }
        SYNTAX_SET
            .find_syntax_by_token(&self.language)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
    }
}

impl Render for CodeRender {
    /// Return a `<pre class="code-block">` block with per-token span colors.
    fn render(&self, source: &str) -> String {
        let mut highlighter = HighlightLines::new(self.syntax(), theme());
        let mut html = String::from("<pre class=\"code-block\"><code>");
        for line in LinesWithEndings::from(source) {
            html.push_str(&highlight_line(&mut highlighter, line));
        }
        html.push_str("</code></pre>");
        html
    }
}

impl RenderPdf for CodeRender {}

/// Return the syntax highlighting theme used for every code block.
fn theme() -> &'static Theme {
    &THEME_SET.themes[CODE_THEME]
}

/// Return highlighted HTML for one line, escaping it if highlighting fails.
fn highlight_line(highlighter: &mut HighlightLines<'_>, line: &str) -> String {
    let Ok(ranges) = highlighter.highlight_line(line, &SYNTAX_SET) else {
        return escape_html(line);
    };
    styled_line_to_highlighted_html(&ranges, IncludeBackground::No)
        .unwrap_or_else(|_| escape_html(line))
}

#[cfg(test)]
mod tests {
    use super::{CodeRender, Render, RenderPdf};

    #[test]
    /// Assert a known language token resolves to that syntax.
    fn known_language_resolves_to_its_syntax() {
        assert_eq!(CodeRender::new("rust").syntax().name, "Rust");
    }

    #[test]
    /// Assert an empty language token resolves to plain text.
    fn empty_language_resolves_to_plain_text() {
        assert_eq!(CodeRender::new("").syntax().name, "Plain Text");
    }

    #[test]
    /// Assert an unknown language token resolves to plain text.
    fn unknown_language_resolves_to_plain_text() {
        assert_eq!(CodeRender::new("notalanguage").syntax().name, "Plain Text");
    }

    #[test]
    /// Assert the block is wrapped in `pre.code-block > code`.
    fn wraps_source_in_code_block() {
        let html = CodeRender::new("rust").render("fn main() {}\n");
        assert!(
            html.starts_with("<pre class=\"code-block\"><code>"),
            "{html}"
        );
        assert!(html.ends_with("</code></pre>"), "{html}");
    }

    #[test]
    /// Assert highlighting colors tokens with inline styles.
    fn highlights_tokens_with_inline_styles() {
        let html = CodeRender::new("rust").render("fn main() {}\n");
        assert!(html.contains("<span style=\"color:"), "{html}");
    }

    #[test]
    /// Assert source markup is escaped so it cannot inject elements.
    fn escapes_source_markup() {
        let html = CodeRender::new("").render("<script>alert(1)</script>\n");
        assert!(html.contains("&lt;script&gt;"), "{html}");
        assert!(!html.contains("<script>"), "{html}");
    }

    #[test]
    /// Assert every source line appears in the output.
    fn keeps_every_line() {
        let html = CodeRender::new("").render("one\ntwo\nthree\n");
        for line in ["one", "two", "three"] {
            assert!(html.contains(line), "missing {line}: {html}");
        }
    }

    #[test]
    /// Assert PDF export reuses the editor HTML.
    fn pdf_html_matches_editor_html() {
        let render = CodeRender::new("rust");
        assert_eq!(
            render.render_pdf("fn main() {}\n"),
            render.render("fn main() {}\n")
        );
    }
}
