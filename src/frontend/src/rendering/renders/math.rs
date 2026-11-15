//! Renders one LaTeX expression to MathML or to ironpress `data-math` HTML.

use super::{Render, RenderPdf};
use crate::constants::TEXT;
use crate::info_messages::MATH_CONVERTER_PANIC;
use crate::rendering::{escape_html, escape_html_comment};
use math_core::{LatexToMathML, MathCoreConfig, MathDisplay};
use std::sync::LazyLock;

/// Shared LaTeX to MathML converter; it holds no per-call state.
///
/// `allow_unreliable_rendering` enables `\widetilde`, `\widecheck` and
/// `\utilde`, which math-core leaves off because browsers stretch them unevenly.
static CONVERTER: LazyLock<LatexToMathML> = LazyLock::new(|| {
    let config = MathCoreConfig {
        allow_unreliable_rendering: true,
        ..MathCoreConfig::default()
    };
    LatexToMathML::new(config, Vec::new()).expect(MATH_CONVERTER_PANIC)
});

/// LaTeX render for one display style.
pub struct MathRender {
    display: MathDisplay,
}

impl MathRender {
    /// Build a math render for `display`.
    pub fn new(display: MathDisplay) -> Self {
        Self { display }
    }

    /// Return the fallback HTML for a LaTeX conversion error.
    ///
    /// Inline math cannot use the shared `<pre>` fallback: it must stay inside
    /// the surrounding paragraph, so both styles use `<code>` instead.
    fn error_html(&self, latex: &str, error: &str) -> String {
        let escaped = escape_html(latex);
        let class = match self.display {
            MathDisplay::Inline => "math-error math-error-inline",
            MathDisplay::Block => "math-error math-error-block",
        };
        let error = escape_html_comment(error);
        format!("<code class=\"{class}\">{escaped}</code><!-- math error: {error} -->")
    }
}

impl Render for MathRender {
    /// Return MathML for `source`, wrapping block math in `div.math-block`.
    fn render(&self, source: &str) -> String {
        match CONVERTER.convert_with_local_state(source, self.display) {
            Ok(result) => match self.display {
                MathDisplay::Inline => result.mathml,
                MathDisplay::Block => format!(r#"<div class="math-block">{}</div>"#, result.mathml),
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
            MathDisplay::Inline => format!(
                r#"<span class="math-inline" style="color:{TEXT}" data-math="{escaped}">{escaped}</span>"#
            ),
            MathDisplay::Block => format!(
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
    use math_core::MathDisplay;

    #[test]
    /// Assert inline math renders bare inline MathML.
    fn inline_renders_inline_mathml() {
        let html = MathRender::new(MathDisplay::Inline).render("x_i");
        assert!(html.starts_with("<math>"), "{html}");
        assert!(!html.contains(r#"display="block""#), "{html}");
        assert!(!html.contains("math-block"), "{html}");
    }

    #[test]
    /// Assert block math renders block MathML inside `div.math-block`.
    fn block_renders_block_mathml_in_wrapper() {
        let html = MathRender::new(MathDisplay::Block).render("E = mc^2");
        assert!(html.contains(r#"display="block""#), "{html}");
        assert!(html.starts_with(r#"<div class="math-block">"#), "{html}");
    }

    #[test]
    /// Assert the conversion-error fallback is inline `<code>` naming the style.
    ///
    /// The fallback must stay inline for both styles because inline math sits
    /// inside a paragraph, where a `<pre>` block is not allowed.
    fn error_fallback_is_inline_code_naming_the_style() {
        let inline = MathRender::new(MathDisplay::Inline).error_html("x_", "unexpected EOF");
        assert!(inline.starts_with("<code class=\"math-error"), "{inline}");
        assert!(inline.contains("math-error-inline"), "{inline}");
        assert!(
            inline.contains("<!-- math error: unexpected EOF -->"),
            "{inline}"
        );
        let block = MathRender::new(MathDisplay::Block).error_html("x_", "unexpected EOF");
        assert!(block.contains("math-error-block"), "{block}");
    }

    #[test]
    /// Assert the error fallback escapes markup in the LaTeX source.
    fn error_fallback_escapes_source_markup() {
        let html = MathRender::new(MathDisplay::Inline).error_html("a < b", "bad");
        assert!(html.contains("a &lt; b"), "{html}");
    }

    #[test]
    /// Assert PDF inline math keeps the LaTeX in `data-math` and drops MathML.
    fn pdf_inline_uses_data_math() {
        let html = MathRender::new(MathDisplay::Inline).render_pdf(r"x = \pi");
        assert!(html.contains(r#"class="math-inline""#), "{html}");
        assert!(html.contains(r#"data-math="x = \pi""#), "{html}");
        assert!(html.contains(&format!(r#"style="color:{TEXT}""#)), "{html}");
        assert!(!html.contains("<math"), "{html}");
    }

    #[test]
    /// Assert PDF block math uses a `math-display` block.
    fn pdf_block_uses_math_display_block() {
        let html = MathRender::new(MathDisplay::Block).render_pdf(r"\frac{a}{b}");
        assert!(html.starts_with(r#"<div class="math-display""#), "{html}");
        assert!(html.contains(r#"data-math="\frac{a}{b}""#), "{html}");
    }

    #[test]
    /// Assert PDF math escapes markup in the LaTeX source.
    fn pdf_escapes_source_markup() {
        let html = MathRender::new(MathDisplay::Inline).render_pdf(r#"a < b" "#);
        assert!(html.contains("&lt;"), "{html}");
        assert!(html.contains("&quot;"), "{html}");
    }

    #[test]
    /// Assert PDF math markup differs from the editor's MathML.
    fn pdf_html_differs_from_editor_html() {
        let render = MathRender::new(MathDisplay::Inline);
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
    /// Assert `\mathcal` and `\mathbf` render as script and bold letters.
    fn mathcal_renders_script_mathml() {
        let html = MathRender::new(MathDisplay::Inline).render(r"\mathbf{q} \in \mathcal{V}_T");
        assert!(!html.contains("math-error"), "{html}");
        assert!(html.contains('𝒱'), "{html}");
        assert!(html.contains("<mi>𝐪</mi>"), "{html}");
    }

    #[test]
    /// Assert a style survives a following subscript or superscript.
    fn style_survives_subscript_and_superscript() {
        let html = MathRender::new(MathDisplay::Inline).render(r"\mathbf{x}_i + \mathbb{R}^n");
        assert!(html.contains("<msub><mi>𝐱</mi>"), "{html}");
        assert!(html.contains("<msup><mi>ℝ</mi>"), "{html}");
    }

    #[test]
    /// Assert PDF math keeps `\mathcal`, which ironpress parses itself.
    fn pdf_keeps_mathcal() {
        let html = MathRender::new(MathDisplay::Inline).render_pdf(r"\mathcal{V}");
        assert!(html.contains(r#"data-math="\mathcal{V}""#), "{html}");
    }

    #[test]
    /// Assert `\pmatrix` is not rewritten as a `\pm` command.
    fn pdf_does_not_space_pmatrix() {
        assert_eq!(super::space_ironpress_binops(r"\pmatrix"), r"\pmatrix");
    }

    #[test]
    /// Assert PDF `data-math` carries the spaced `\pm`.
    fn pdf_data_math_contains_spaced_pm() {
        let html =
            MathRender::new(MathDisplay::Block).render_pdf(r"\frac{-b \pm \sqrt{b^2-4ac}}{2a}");
        assert!(html.contains(r"\;\pm\;"), "{html}");
        assert!(!html.contains(r" \pm \"), "{html}");
    }

    /// Render `latex` as block math and assert it converted.
    fn block(latex: &str) -> String {
        let html = MathRender::new(MathDisplay::Block).render(latex);
        assert!(!html.contains("math-error"), "{html}");
        html
    }

    #[test]
    /// Assert `alignedat` aligns column pairs with no space between the pairs.
    fn alignedat_renders_tight_column_pairs() {
        let html = block("\\begin{alignedat}{2}\na &= b, &\\quad c &= d \\\\\ne &= f, &\\quad g &= h\n\\end{alignedat}");
        assert_eq!(html.matches("<mtr>").count(), 2, "{html}");
        assert_eq!(html.matches("<mtd").count(), 8, "{html}");
        assert_eq!(
            html.matches("padding-left: 0;padding-right: 0;").count(),
            8,
            "{html}"
        );
    }

    #[test]
    /// Assert `alignedat` rejects a missing or non-numeric pair count.
    fn alignedat_requires_a_pair_count() {
        let html = MathRender::new(MathDisplay::Block)
            .render("\\begin{alignedat}{x}\na &= b\n\\end{alignedat}");
        assert!(html.contains("math-error"), "{html}");
    }

    #[test]
    /// Assert `\widetilde` renders a stretchy accent the style sheet can widen.
    fn widetilde_renders_classed_stretchy_accent() {
        let html = block(r"\widetilde{X_1 + X_2}");
        assert!(
            html.contains(r#"<mover accent="true" class="math-core-wide-tilde">"#),
            "{html}"
        );
        assert!(html.contains(r#"<mo stretchy="true">~</mo>"#), "{html}");
    }

    #[test]
    /// Assert `\widehat`, `\overline` and `\underline` render stretchy accents.
    fn wide_accents_and_lines_render() {
        let html = block(r"\widehat{A + B} \overline{z} \underline{u + v}");
        assert_eq!(
            html.matches(r#"<mover accent="true">"#).count(),
            2,
            "{html}"
        );
        assert!(html.contains(r#"<munder accentunder="true">"#), "{html}");
    }

    #[test]
    /// Assert `\pmod`, `\pod` and `\bmod` all render.
    fn modular_arithmetic_commands_render() {
        let html = block(r"a \equiv b \pmod{m} \qquad a \equiv b \pod{m} \qquad r = a \bmod m");
        assert_eq!(html.matches("<mi>mod</mi>").count(), 2, "{html}");
        assert_eq!(
            html.matches(r#"<mo stretchy="false">(</mo>"#).count(),
            2,
            "{html}"
        );
    }

    #[test]
    /// Assert `\boxed` frames its content.
    fn boxed_renders_box_enclosure() {
        let html = block(r"\boxed{e^{i\pi} + 1 = 0}");
        assert!(html.contains(r#"<menclose notation="box">"#), "{html}");
    }

    #[test]
    /// Assert `\cancelto` strikes the expression with an arrow carrying the value.
    fn cancelto_renders_arrow_with_value() {
        let html = block(r"\cancelto{0}{5x^3}");
        assert!(
            html.starts_with(
                r#"<div class="math-block"><math display="block"><msup><menclose notation="updiagonalarrow">"#
            ),
            "{html}"
        );
        assert!(
            html.contains(
                r#"<mrow class="menclose-updiagonalarrow"></mrow></menclose><mn>0</mn></msup>"#
            ),
            "{html}"
        );
    }

    #[test]
    /// Assert `\cancel` keeps the strike element the style sheet draws.
    fn cancel_renders_strike() {
        let html = block(r"\cancel{x - 1}");
        assert!(
            html.contains(r#"<menclose notation="updiagonalstrike">"#),
            "{html}"
        );
        assert!(
            html.contains(r#"class="menclose-updiagonalstrike""#),
            "{html}"
        );
    }

    #[test]
    /// Assert `\color` accepts a `#rrggbb` literal and scopes it to the group.
    fn color_accepts_hex_literal() {
        let html = block(r"{\color{#2563eb} F} = \color{#dc2626}{m}");
        assert!(
            html.contains(r#"<mrow style="color:#2563EB;"><mi>F</mi></mrow><mo>=</mo>"#),
            "{html}"
        );
        assert!(html.contains(r#"<mrow style="color:#DC2626;">"#), "{html}");
    }

    #[test]
    /// Assert a `#` that is not a color literal is still an error.
    fn stray_hash_is_still_an_error() {
        for latex in [r"x #2563eb", r"\color{#25} x", r"\color{#2563ebf} x"] {
            let html = MathRender::new(MathDisplay::Block).render(latex);
            assert!(html.contains("math-error"), "{latex}: {html}");
        }
    }

    #[test]
    /// Assert math accents inside `\text` render as accented letters.
    fn text_mode_accepts_math_accents() {
        let html = block(r"\text{(Amp\grave{e}re-Maxwell)}");
        assert!(
            html.contains("<mtext>(Ampe\u{300}re-Maxwell)</mtext>"),
            "{html}"
        );
    }

    #[test]
    /// Assert `aligned` takes an empty column before a trailing `\text` note.
    fn aligned_takes_trailing_text_column() {
        let html = block("\\begin{aligned}\na &= b && \\text{(Gauss's)} \\\\\nc &= d && \\text{(x)}\n\\end{aligned}");
        assert_eq!(html.matches("<mtd").count(), 8, "{html}");
        assert!(html.contains("<mtext>(Gauss’s)</mtext>"), "{html}");
    }
}
