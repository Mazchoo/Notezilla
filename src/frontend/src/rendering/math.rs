use super::escape_html;
use crate::constants::TEXT;
use latex2mathml::{latex_to_mathml, DisplayStyle};

/// Convert a single LaTeX math expression to MathML.
pub fn render_latex(latex: &str, display: DisplayStyle) -> String {
    match latex_to_mathml(latex, display) {
        Ok(mathml) => match display {
            DisplayStyle::Inline => mathml,
            DisplayStyle::Block => format!(r#"<div class="math-block">{mathml}</div>"#),
        },
        Err(e) => math_error_html(latex, display, &e.to_string()),
    }
}

/// Emit ironpress `data-math` HTML so PDF conversion typesets LaTeX.
fn render_latex_for_pdf(latex: &str, display: DisplayStyle) -> String {
    let escaped = escape_html(latex);
    match display {
        DisplayStyle::Inline => {
            format!(
                r#"<span class="math-inline" style="color:{TEXT}" data-math="{escaped}">{escaped}</span>"#
            )
        }
        DisplayStyle::Block => {
            format!(
                r#"<div class="math-display" style="color:{TEXT}" data-math="{escaped}">{escaped}</div>"#
            )
        }
    }
}

/// Render a fallback HTML fragment for a LaTeX conversion error.
fn math_error_html(latex: &str, display: DisplayStyle, err: &str) -> String {
    let escaped = escape_html(latex);
    let class = match display {
        DisplayStyle::Inline => "math-error math-error-inline",
        DisplayStyle::Block => "math-error math-error-block",
    };
    format!("<code class=\"{class}\">{escaped}</code><!-- math error: {err} -->")
}

/// Replace `$…$` / `$$…$$` outside code with MathML HTML.
pub fn substitute_math(src: &str) -> String {
    rewrite_math(src, render_latex)
}

/// Replace `$…$` / `$$…$$` outside code with ironpress `data-math` tags.
pub fn substitute_math_for_pdf(src: &str) -> String {
    rewrite_math(src, render_latex_for_pdf)
}

/// Rewrite math delimiters in `src` with `render`, skipping fenced and inline code.
fn rewrite_math(src: &str, render: fn(&str, DisplayStyle) -> String) -> String {
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

        if bytes[i..].starts_with(b"$$") {
            if let Some(close) = find_closing_delim(bytes, i + 2, b"$$") {
                let latex = src[i + 2..close].trim();
                out.push_str(&render(latex, DisplayStyle::Block));
                i = close + 2;
                continue;
            }
        }

        if bytes[i] == b'$' && !is_escaped(bytes, i) {
            if let Some(close) = find_closing_inline_dollar(bytes, i + 1) {
                let latex = src[i + 1..close].trim();
                out.push_str(&render(latex, DisplayStyle::Inline));
                i = close + 1;
                continue;
            }
        }

        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

/// Return whether the byte at `i` is escaped by a preceding backslash.
fn is_escaped(bytes: &[u8], i: usize) -> bool {
    i > 0 && bytes[i - 1] == b'\\'
}

/// Find the next occurrence of `delim` at or after `from`.
fn find_closing_delim(bytes: &[u8], from: usize, delim: &[u8]) -> Option<usize> {
    let mut i = from;
    while i + delim.len() <= bytes.len() {
        if bytes[i..].starts_with(delim) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Find the closing `$` for inline math, rejecting a `$$` pair.
fn find_closing_inline_dollar(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'$' && !is_escaped(bytes, i) {
            // Don't treat the start of `$$` as an inline closer.
            if bytes[i..].starts_with(b"$$") {
                return None;
            }
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Return the index just past a fenced code block starting at `start`.
fn skip_fenced_code(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    // Opening fence: one or more backticks (at least 3), then rest of line.
    let mut i = start;
    while i < bytes.len() && bytes[i] == b'`' {
        i += 1;
    }
    let fence_len = i - start;
    // Find end of opening line.
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    if i < bytes.len() {
        i += 1; // consume newline
    }
    // Closing fence: at least `fence_len` backticks on their own line.
    while i < bytes.len() {
        let line_start = i;
        while i < bytes.len() && bytes[i] != b'\n' {
            i += 1;
        }
        let line = &src[line_start..i];
        let trimmed = line.trim_end_matches(['\r']);
        if trimmed.bytes().all(|b| b == b'`') && trimmed.len() >= fence_len {
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

/// Return the index just past an inline code span starting at `start`.
fn skip_inline_code(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let mut i = start;
    while i < bytes.len() && bytes[i] == b'`' {
        i += 1;
    }
    let fence_len = i - start;
    while i + fence_len <= bytes.len() {
        if bytes[i..].starts_with(&bytes[start..start + fence_len]) {
            // Ensure we don't match a longer run of backticks.
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
    use super::*;

    #[test]
    /// Assert inline `$…$` and block `$$…$$` become distinct MathML displays.
    fn inline_vs_block() {
        let out = substitute_math(
            r#"inline $\gets$ and block

$$
E = mc^2
$$
"#,
        );
        assert!(out.contains(r#"display="inline""#), "{out}");
        assert!(out.contains(r#"display="block""#), "{out}");
        assert!(out.contains(r#"class="math-block""#), "{out}");
        assert!(
            out.contains('←') || out.contains("gets") || out.contains("<mo"),
            "{out}"
        );
    }

    #[test]
    /// Assert fenced code keeps literal `$…$` delimiters.
    fn skips_fenced_code() {
        let src = "```\n$not_math$\n```\n";
        let out = substitute_math(src);
        assert!(out.contains("$not_math$"), "{out}");
    }

    #[test]
    /// Assert inline code keeps literal `$…$` delimiters.
    fn skips_inline_code() {
        let out = substitute_math("use `$x$` in code");
        assert!(out.contains("`$x$`"), "{out}");
    }

    #[test]
    /// Assert `_` inside math is converted before markdown emphasis runs.
    fn underscores_not_eaten_before_markdown() {
        let out = substitute_math(r#"$x_i$"#);
        assert!(out.contains("<math"), "{out}");
        assert!(!out.contains("$x_i$"), "{out}");
    }

    #[test]
    /// Assert PDF math emits `data-math` tags instead of MathML.
    fn pdf_math_uses_ironpress_data_math_not_mathml() {
        let out = substitute_math_for_pdf(
            r#"inline $x_i$ and block

$$
\frac{a}{b}
$$
"#,
        );
        assert!(!out.contains("<math"), "{out}");
        assert!(
            out.contains(r#"class="math-inline""#)
                && out.contains(r#"data-math="x_i""#)
                && out.contains(&format!(r#"style="color:{TEXT}""#)),
            "{out}"
        );
        assert!(
            out.contains(r#"class="math-display""#)
                && out.contains(r#"data-math=""#)
                && out.contains(r#"\frac{a}{b}"#),
            "{out}"
        );
        assert!(!out.contains("<em>"), "{out}");
    }

    #[test]
    /// Assert PDF math substitution skips fenced and inline code.
    fn pdf_math_skips_fenced_and_inline_code() {
        assert!(substitute_math_for_pdf("```\n$not_math$\n```\n").contains("$not_math$"));
        assert!(substitute_math_for_pdf("use `$x$` in code").contains("`$x$`"));
    }

    #[test]
    /// Assert a list-item `$…$` stays inline for PDF export.
    fn pdf_list_item_single_dollar_stays_inline() {
        let out = substitute_math_for_pdf("- $x = \\pi$\n");
        assert!(
            out.contains(r#"<span class="math-inline""#) && out.contains(r#"data-math="x = \pi""#),
            "{out}"
        );
        assert!(!out.contains("math-display"), "{out}");
        assert!(!out.contains("$x = \\pi$"), "{out}");
    }
}
