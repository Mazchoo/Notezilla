//! Replaces `$…$` and `$$…$$` math delimiters before markdown parsing.
//!
//! Math is substituted first so that `_` and `*` inside an equation are not
//! parsed as emphasis, and so display math becomes a block-level element.

use super::renders::{MathRender, RenderPdf, RenderTarget};
use latex2mathml::DisplayStyle;

/// Render one delimited LaTeX expression for `target`.
fn render_math(latex: &str, display: DisplayStyle, target: RenderTarget) -> String {
    MathRender::new(display).render_for(target, latex.trim())
}

/// Replace `$…$` and `$$…$$` outside code spans with math rendered for `target`.
pub fn substitute_math(src: &str, target: RenderTarget) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i..].starts_with(b"```") {
            let end = skip_fenced_code(src, i);
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }

        if bytes[i] == b'`' {
            let end = skip_inline_code(src, i);
            out.push_str(&src[i..end]);
            i = end;
            continue;
        }

        if bytes[i..].starts_with(b"$$") && !is_escaped(bytes, i) {
            if let Some(close) = find_closing_display_math(bytes, i) {
                let latex = &src[i + 2..close];
                if !latex.trim().is_empty() {
                    out.push_str(&render_math(latex, DisplayStyle::Block, target));
                    i = close + 2;
                    continue;
                }
            }
            out.push_str("$$");
            i += 2;
            continue;
        }

        if bytes[i] == b'$' && !is_escaped(bytes, i) {
            let next = bytes.get(i + 1).copied();
            let opens = next.is_some_and(|b| !b.is_ascii_whitespace() && b != b'$');
            let after_dollar = i > 0 && bytes[i - 1] == b'$';
            if opens && !after_dollar {
                if let Some(close) = find_closing_inline_dollar(bytes, i + 1) {
                    out.push_str(&render_math(
                        &src[i + 1..close],
                        DisplayStyle::Inline,
                        target,
                    ));
                    i = close + 1;
                    continue;
                }
            }
        }

        let ch = src[i..].chars().next().expect("byte index is a boundary");
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

/// Return whether the byte at `i` is escaped by an odd run of preceding backslashes.
fn is_escaped(bytes: &[u8], i: usize) -> bool {
    let mut n = 0;
    while i > n && bytes[i - 1 - n] == b'\\' {
        n += 1;
    }
    return n % 2 == 1;
}

/// Find the closing `$$` for display math starting at `open_at`.
///
/// Same-line `$$…$$` always closes. A `$$` that is not at the start of a line
/// does not span a newline, so a trailing `$$` and the `$$` inside `$a$$b$`
/// stay literal.
fn find_closing_display_math(bytes: &[u8], open_at: usize) -> Option<usize> {
    let from = open_at + 2;
    if let Some(close) = find_dollars_before_newline(bytes, from) {
        return Some(close);
    }
    if at_line_start(bytes, open_at) {
        return find_line_start_dollars(bytes, from);
    }
    None
}

/// Return the next unescaped `$$` on this line, before any newline.
fn find_dollars_before_newline(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i + 2 <= bytes.len() {
        if bytes[i] == b'\n' || bytes[i] == b'\r' {
            return None;
        }
        if bytes[i..].starts_with(b"$$") && !is_escaped(bytes, i) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Return the next unescaped `$$` that sits at the start of a line.
fn find_line_start_dollars(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i + 2 <= bytes.len() {
        if bytes[i..].starts_with(b"$$") && !is_escaped(bytes, i) && at_line_start(bytes, i) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Return whether `i` is at the start of a line, ignoring leading spaces and tabs.
fn at_line_start(bytes: &[u8], i: usize) -> bool {
    let mut j = i;
    while j > 0 && matches!(bytes[j - 1], b' ' | b'\t') {
        j -= 1;
    }
    j == 0 || bytes[j - 1] == b'\n' || bytes[j - 1] == b'\r'
}

/// Find the closing `$` for inline math, rejecting a display `$$` pair.
///
/// The closer must be on the same line, have a non-space before it, and must
/// not be followed by a digit. A newline or a `$$` run cancels the opener.
fn find_closing_inline_dollar(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'\n' || bytes[i] == b'\r' {
            return None;
        }
        if bytes[i] == b'$' && !is_escaped(bytes, i) {
            // `$$` is display syntax; do not split it into an inline closer
            // plus a new opener (`$a$$b$` stays literal).
            if bytes[i..].starts_with(b"$$") {
                return None;
            }
            let preceded_by_non_space = i > 0 && !bytes[i - 1].is_ascii_whitespace();
            let followed_by_digit = bytes.get(i + 1).is_some_and(|b| b.is_ascii_digit());
            if preceded_by_non_space && !followed_by_digit {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Return the index just past the fenced code block starting at `start`.
fn skip_fenced_code(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let mut i = start;
    while i < bytes.len() && bytes[i] == b'`' {
        i += 1;
    }
    let fence_len = i - start;
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    if i < bytes.len() {
        i += 1;
    }
    // The block ends at a line of at least `fence_len` backticks and nothing else.
    while i < bytes.len() {
        let line_start = i;
        while i < bytes.len() && bytes[i] != b'\n' {
            i += 1;
        }
        let line = src[line_start..i].trim_end_matches('\r');
        if line.bytes().all(|b| b == b'`') && line.len() >= fence_len {
            if i < bytes.len() {
                i += 1;
            }
            return i;
        }
        if i < bytes.len() {
            i += 1;
        }
    }
    bytes.len()
}

/// Return the index just past the inline code span starting at `start`.
fn skip_inline_code(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let mut i = start;
    while i < bytes.len() && bytes[i] == b'`' {
        i += 1;
    }
    let fence_len = i - start;
    while i + fence_len <= bytes.len() {
        if bytes[i..].starts_with(&bytes[start..start + fence_len]) {
            // A longer run of backticks does not close the span.
            let after = i + fence_len;
            if after < bytes.len() && bytes[after] == b'`' {
                i += 1;
                continue;
            }
            return after;
        }
        i += 1;
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::{skip_fenced_code, skip_inline_code, substitute_math, RenderTarget};
    use crate::constants::TEXT;

    #[test]
    /// Assert inline `$…$` and block `$$…$$` become distinct MathML displays.
    fn editor_target_renders_inline_and_block_mathml() {
        let out = substitute_math(
            "inline $\\gets$ and block\n\n$$\nE = mc^2\n$$\n",
            RenderTarget::Editor,
        );
        assert!(out.contains(r#"display="inline""#), "{out}");
        assert!(out.contains(r#"display="block""#), "{out}");
        assert!(out.contains(r#"class="math-block""#), "{out}");
    }

    #[test]
    /// Assert `_` inside math is converted before markdown emphasis runs.
    fn underscores_are_converted_before_markdown() {
        let out = substitute_math("$x_i$", RenderTarget::Editor);
        assert!(out.contains("<math"), "{out}");
        assert!(!out.contains("$x_i$"), "{out}");
    }

    #[test]
    /// Assert the PDF target emits `data-math` tags instead of MathML.
    fn pdf_target_renders_data_math() {
        let out = substitute_math(
            "inline $x_i$ and block\n\n$$\n\\frac{a}{b}\n$$\n",
            RenderTarget::Pdf,
        );
        assert!(!out.contains("<math"), "{out}");
        assert!(out.contains(r#"class="math-inline""#), "{out}");
        assert!(out.contains(r#"data-math="x_i""#), "{out}");
        assert!(out.contains(&format!(r#"style="color:{TEXT}""#)), "{out}");
        assert!(out.contains(r#"class="math-display""#), "{out}");
        assert!(out.contains(r#"\frac{a}{b}"#), "{out}");
    }

    #[test]
    /// Assert surrounding text is preserved around substituted math.
    fn keeps_surrounding_text() {
        let out = substitute_math("before $x$ after", RenderTarget::Editor);
        assert!(out.starts_with("before "), "{out}");
        assert!(out.ends_with(" after"), "{out}");
    }

    #[test]
    /// Assert a lone `$` without a closing delimiter is left as text.
    fn unclosed_dollar_is_left_as_text() {
        let out = substitute_math("costs $5 total", RenderTarget::Editor);
        assert_eq!(out, "costs $5 total");
    }

    #[test]
    /// Assert inline `$…$` does not span a newline; the opener is cancelled.
    fn inline_math_does_not_span_a_newline() {
        let out = substitute_math("$a\nb$", RenderTarget::Editor);
        assert_eq!(out, "$a\nb$");
    }

    #[test]
    /// Assert two currency amounts stay literal text.
    fn two_dollar_amounts_are_not_math() {
        let out = substitute_math("costs $5 and $10 today", RenderTarget::Editor);
        assert_eq!(out, "costs $5 and $10 today");
    }

    #[test]
    /// Assert an escaped `\$` does not open math.
    fn escaped_dollar_does_not_open_math() {
        let out = substitute_math("\\$5 and \\$6", RenderTarget::Editor);
        assert_eq!(out, "\\$5 and \\$6");
    }

    #[test]
    /// Assert `\\$` opens math: the backslash is escaped, the dollar is not.
    fn even_backslash_run_does_not_escape_dollar() {
        let out = substitute_math("\\\\$x$", RenderTarget::Editor);
        assert!(out.contains("<math"), "{out}");
        assert!(!out.contains("$x$"), "{out}");
    }

    #[test]
    /// Assert multi-byte characters around math are preserved.
    fn keeps_multibyte_characters() {
        let out = substitute_math("café $x$ ☕", RenderTarget::Editor);
        assert!(out.starts_with("café "), "{out}");
        assert!(out.ends_with(" ☕"), "{out}");
    }

    #[test]
    /// Assert both targets keep literal `$…$` inside fenced and inline code.
    fn code_spans_keep_literal_dollars() {
        for target in [RenderTarget::Editor, RenderTarget::Pdf] {
            let fenced = substitute_math("```\n$not_math$\n```\n", target);
            assert!(fenced.contains("$not_math$"), "{fenced}");
            let inline = substitute_math("use `$x$` in code", target);
            assert!(inline.contains("`$x$`"), "{inline}");
        }
    }

    #[test]
    /// Assert a list item's `$…$` stays inline for PDF export.
    fn pdf_list_item_single_dollar_stays_inline() {
        let out = substitute_math("- $x = \\pi$\n", RenderTarget::Pdf);
        assert!(out.contains(r#"<span class="math-inline""#), "{out}");
        assert!(out.contains(r#"data-math="x = \pi""#), "{out}");
        assert!(!out.contains("math-display"), "{out}");
    }

    #[test]
    /// Assert a fenced block is skipped up to and including its closing fence.
    fn skips_whole_fenced_block() {
        let src = "```rust\nlet x = 1;\n```\nafter";
        let end = skip_fenced_code(src, 0);
        assert_eq!(&src[..end], "```rust\nlet x = 1;\n```\n");
    }

    #[test]
    /// Assert an unterminated fenced block is skipped to the end of the source.
    fn skips_unterminated_fenced_block_to_end() {
        let src = "```\nno close";
        assert_eq!(skip_fenced_code(src, 0), src.len());
    }

    #[test]
    /// Assert an inline code span is skipped up to its closing backticks.
    fn skips_inline_code_span() {
        let src = "`code` after";
        let end = skip_inline_code(src, 0);
        assert_eq!(&src[..end], "`code`");
    }

    #[test]
    /// Assert a double-backtick span is skipped as a whole.
    fn skips_double_backtick_span() {
        let src = "``a ` b`` after";
        let end = skip_inline_code(src, 0);
        assert_eq!(&src[..end], "``a ` b``");
    }

    #[test]
    /// Assert `$a$$b$` and a line-ending `$$` stay literal; later `$$…$$` still converts.
    fn abutting_and_trailing_dollars_stay_literal() {
        let out = substitute_math(
            "hello $a$$b$\ntrailing $$\n\n$$\nE = mc^2\n$$\n",
            RenderTarget::Editor,
        );
        assert!(out.contains("hello $a$$b$"), "{out}");
        assert!(out.contains("trailing $$"), "{out}");
        assert!(out.contains(r#"class="math-block""#), "{out}");
        assert!(!out.contains("math-error"), "{out}");
    }
}
