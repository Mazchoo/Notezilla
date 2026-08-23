//! Renders one LaTeX expression to MathML or to ironpress `data-math` HTML.

use super::{Render, RenderPdf};
use crate::constants::TEXT;
use crate::rendering::escape_html;
use latex2mathml::{latex_to_mathml, DisplayStyle};

/// LaTeX render for one display style.
pub struct MathRender {
    display: DisplayStyle,
}

impl MathRender {
    /// Build a math render for `display`.
    pub fn new(display: DisplayStyle) -> Self {
        Self { display }
    }

    /// Return the fallback HTML for a LaTeX conversion error.
    ///
    /// Inline math cannot use the shared `<pre>` fallback: it must stay inside
    /// the surrounding paragraph, so both styles use `<code>` instead.
    fn error_html(&self, latex: &str, error: &str) -> String {
        let escaped = escape_html(latex);
        let class = match self.display {
            DisplayStyle::Inline => "math-error math-error-inline",
            DisplayStyle::Block => "math-error math-error-block",
        };
        format!("<code class=\"{class}\">{escaped}</code><!-- math error: {error} -->")
    }
}

impl Render for MathRender {
    /// Return MathML for `source`, wrapping block math in `div.math-block`.
    fn render(&self, source: &str) -> String {
        match latex_to_mathml(source, self.display) {
            Ok(mathml) => match self.display {
                DisplayStyle::Inline => mathml,
                DisplayStyle::Block => format!(r#"<div class="math-block">{mathml}</div>"#),
            },
            Err(e) => self.error_html(source, &e.to_string()),
        }
    }
}

impl RenderPdf for MathRender {
    /// Return `data-math` HTML so ironpress typesets the LaTeX itself.
    ///
    /// Ironpress ignores MathML, so PDF export keeps the LaTeX source in a
    /// `data-math` attribute and paints it with the page text color.
    fn render_pdf(&self, source: &str) -> String {
        let escaped = escape_html(&space_ironpress_binops(source));
        match self.display {
            DisplayStyle::Inline => format!(
                r#"<span class="math-inline" style="color:{TEXT}" data-math="{escaped}">{escaped}</span>"#
            ),
            DisplayStyle::Block => format!(
                r#"<div class="math-display" style="color:{TEXT}" data-math="{escaped}">{escaped}</div>"#
            ),
        }
    }
}

/// Insert thick spaces around `\pm` / `\mp` so ironpress treats them as binary.
///
/// Ironpress classifies `±` and `∓` as Ord, so Knuth spacing around them is 0.
/// `\;` is the thick space it does parse (5mu). Leave `\pmatrix` and similar
/// commands untouched: only a complete `\pm` / `\mp` token is rewritten.
fn space_ironpress_binops(latex: &str) -> String {
    let mut out = String::with_capacity(latex.len() + 8);
    let mut chars = latex.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if ch == '\\' {
            let rest = &latex[i + 1..];
            let cmd_len = rest
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .map(|c| c.len_utf8())
                .sum::<usize>();
            let cmd = &rest[..cmd_len];
            if cmd == "pm" || cmd == "mp" {
                out.push_str(r"\;\");
                out.push_str(cmd);
                out.push_str(r"\;");
                for _ in 0..cmd.chars().count() {
                    chars.next();
                }
                continue;
            }
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{MathRender, Render, RenderPdf};
    use crate::constants::TEXT;
    use latex2mathml::DisplayStyle;

    #[test]
    /// Assert inline math renders bare inline MathML.
    fn inline_renders_inline_mathml() {
        let html = MathRender::new(DisplayStyle::Inline).render("x_i");
        assert!(html.contains(r#"display="inline""#), "{html}");
        assert!(!html.contains("math-block"), "{html}");
    }

    #[test]
    /// Assert block math renders block MathML inside `div.math-block`.
    fn block_renders_block_mathml_in_wrapper() {
        let html = MathRender::new(DisplayStyle::Block).render("E = mc^2");
        assert!(html.contains(r#"display="block""#), "{html}");
        assert!(html.starts_with(r#"<div class="math-block">"#), "{html}");
    }

    #[test]
    /// Assert the conversion-error fallback is inline `<code>` naming the style.
    ///
    /// The fallback must stay inline for both styles because inline math sits
    /// inside a paragraph, where a `<pre>` block is not allowed.
    fn error_fallback_is_inline_code_naming_the_style() {
        let inline = MathRender::new(DisplayStyle::Inline).error_html("x_", "unexpected EOF");
        assert!(inline.starts_with("<code class=\"math-error"), "{inline}");
        assert!(inline.contains("math-error-inline"), "{inline}");
        assert!(inline.contains("<!-- math error: unexpected EOF -->"), "{inline}");
        let block = MathRender::new(DisplayStyle::Block).error_html("x_", "unexpected EOF");
        assert!(block.contains("math-error-block"), "{block}");
    }

    #[test]
    /// Assert the error fallback escapes markup in the LaTeX source.
    fn error_fallback_escapes_source_markup() {
        let html = MathRender::new(DisplayStyle::Inline).error_html("a < b", "bad");
        assert!(html.contains("a &lt; b"), "{html}");
    }

    #[test]
    /// Assert PDF inline math keeps the LaTeX in `data-math` and drops MathML.
    fn pdf_inline_uses_data_math() {
        let html = MathRender::new(DisplayStyle::Inline).render_pdf(r"x = \pi");
        assert!(html.contains(r#"class="math-inline""#), "{html}");
        assert!(html.contains(r#"data-math="x = \pi""#), "{html}");
        assert!(html.contains(&format!(r#"style="color:{TEXT}""#)), "{html}");
        assert!(!html.contains("<math"), "{html}");
    }

    #[test]
    /// Assert PDF block math uses a `math-display` block.
    fn pdf_block_uses_math_display_block() {
        let html = MathRender::new(DisplayStyle::Block).render_pdf(r"\frac{a}{b}");
        assert!(html.starts_with(r#"<div class="math-display""#), "{html}");
        assert!(html.contains(r#"data-math="\frac{a}{b}""#), "{html}");
    }

    #[test]
    /// Assert PDF math escapes markup in the LaTeX source.
    fn pdf_escapes_source_markup() {
        let html = MathRender::new(DisplayStyle::Inline).render_pdf(r#"a < b" "#);
        assert!(html.contains("&lt;"), "{html}");
        assert!(html.contains("&quot;"), "{html}");
    }

    #[test]
    /// Assert PDF math markup differs from the editor's MathML.
    fn pdf_html_differs_from_editor_html() {
        let render = MathRender::new(DisplayStyle::Inline);
        assert_ne!(render.render_pdf("x_i"), render.render("x_i"));
    }

    #[test]
    /// Assert PDF LaTeX puts thick space around `\pm` and `\mp`.
    fn pdf_spaces_pm_and_mp() {
        assert_eq!(super::space_ironpress_binops(r"a\pm b"), r"a\;\pm\; b");
        assert_eq!(super::space_ironpress_binops(r"a\mp b"), r"a\;\mp\; b");
        assert_eq!(
            super::space_ironpress_binops(r"\frac{-b \pm \sqrt{x}}{2a}"),
            r"\frac{-b \;\pm\; \sqrt{x}}{2a}"
        );
    }

    #[test]
    /// Assert `\pmatrix` is not rewritten as a `\pm` command.
    fn pdf_does_not_space_pmatrix() {
        assert_eq!(super::space_ironpress_binops(r"\pmatrix"), r"\pmatrix");
    }

    #[test]
    /// Assert PDF `data-math` carries the spaced `\pm`.
    fn pdf_data_math_contains_spaced_pm() {
        let html = MathRender::new(DisplayStyle::Block)
            .render_pdf(r"\frac{-b \pm \sqrt{b^2-4ac}}{2a}");
        assert!(html.contains(r"\;\pm\;"), "{html}");
        assert!(!html.contains(r" \pm \"), "{html}");
    }
}
