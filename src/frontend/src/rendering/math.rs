use latex2mathml::{latex_to_mathml, DisplayStyle};

/// Convert a single LaTeX math expression to MathML.
///
/// `DisplayStyle::Inline` is for `$…$` (flows with surrounding text).
/// `DisplayStyle::Block` is for `$$…$$` (standalone display equation).
pub fn render_latex(latex: &str, display: DisplayStyle) -> String {
    match latex_to_mathml(latex, display) {
        Ok(mathml) => match display {
            DisplayStyle::Inline => mathml,
            DisplayStyle::Block => format!(r#"<div class="math-block">{mathml}</div>"#),
        },
        Err(e) => {
            let escaped = escape_html(latex);
            let class = match display {
                DisplayStyle::Inline => "math-error math-error-inline",
                DisplayStyle::Block => "math-error math-error-block",
            };
            format!(
                "<code class=\"{class}\">{escaped}</code><!-- math error: {e} -->"
            )
        }
    }
}

/// Replace `$…$` / `$$…$$` outside code spans and fenced blocks with MathML HTML.
///
/// Runs before `pulldown-cmark` so characters like `_` inside math are not
/// interpreted as markdown emphasis. Raw MathML is left as HTML for the
/// CommonMark HTML passthrough.
pub fn substitute_math(src: &str) -> String {
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
                out.push_str(&render_latex(latex, DisplayStyle::Block));
                i = close + 2;
                continue;
            }
        }

        if bytes[i] == b'$' && !is_escaped(bytes, i) {
            if let Some(close) = find_closing_inline_dollar(bytes, i + 1) {
                let latex = src[i + 1..close].trim();
                out.push_str(&render_latex(latex, DisplayStyle::Inline));
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

fn is_escaped(bytes: &[u8], i: usize) -> bool {
    i > 0 && bytes[i - 1] == b'\\'
}

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

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_vs_block() {
        let out = substitute_math(r#"inline $\gets$ and block

$$
E = mc^2
$$
"#);
        assert!(out.contains(r#"display="inline""#), "{out}");
        assert!(out.contains(r#"display="block""#), "{out}");
        assert!(out.contains(r#"class="math-block""#), "{out}");
        assert!(out.contains('←') || out.contains("gets") || out.contains("<mo"), "{out}");
    }

    #[test]
    fn skips_fenced_code() {
        let src = "```\n$not_math$\n```\n";
        let out = substitute_math(src);
        assert!(out.contains("$not_math$"), "{out}");
    }

    #[test]
    fn skips_inline_code() {
        let out = substitute_math("use `$x$` in code");
        assert!(out.contains("`$x$`"), "{out}");
    }

    #[test]
    fn underscores_not_eaten_before_markdown() {
        let out = substitute_math(r#"$x_i$"#);
        assert!(out.contains("<math"), "{out}");
        assert!(!out.contains("$x_i$"), "{out}");
    }
}
