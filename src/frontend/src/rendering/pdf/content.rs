//! Recolors the paint operators of a PDF content stream.
//!
//! Ironpress ignores the HTML text color in two places: math rules, radicals
//! and large delimiters are hardcoded `0 0 0 rg` / `0 0 0 RG`, and math glyphs
//! and embedded-font body text are emitted with no fill, so both come out
//! black. This pass paints them with the current theme's text color.
//!
//! Inline SVG (mermaid, graphviz) already carries the colors it was rendered
//! with, including intentional black, so its black is left unchanged.

use crate::constants::{PDF_BLACK_FILL, PDF_BLACK_STROKE, PDF_SVG_PLACEMENT_PREFIX};
use crate::theme::current_theme;

/// Recolor black outside inline SVG and fill text objects that set no fill.
pub fn recolor_content(content: &[u8]) -> Vec<u8> {
    let palette = current_theme().palette();
    let mut out = Vec::with_capacity(content.len() + 64);
    let mut depth = 0usize;
    let mut svg_depth = None;
    let mut rest = content;
    while !rest.is_empty() {
        let (line, next) = split_line(rest);
        rest = next;
        let op = line.trim_ascii();
        if op == b"q" || op.starts_with(b"q ") {
            depth += 1;
            if svg_depth.is_none() && split_line(rest).0.starts_with(PDF_SVG_PLACEMENT_PREFIX) {
                svg_depth = Some(depth);
            }
        } else if op == b"Q" {
            if svg_depth == Some(depth) {
                svg_depth = None;
            }
            depth = depth.saturating_sub(1);
        }
        let repaint = match op {
            _ if svg_depth.is_some() => None,
            PDF_BLACK_FILL => Some(palette.text_fill),
            PDF_BLACK_STROKE => Some(palette.text_stroke),
            _ => None,
        };
        match repaint {
            Some(operator) => {
                out.extend_from_slice(operator.trim_ascii_end());
                out.extend_from_slice(&line[op.len()..]);
            }
            None => out.extend_from_slice(line),
        }
        // The fill has to sit inside the text object: a fill set before `BT`
        // does not apply to the glyphs ironpress emits after it.
        if op == b"BT" && !sets_fill(rest) {
            out.extend_from_slice(palette.text_fill);
        }
    }
    out
}

/// Split `content` after its first newline, keeping the newline on the line.
fn split_line(content: &[u8]) -> (&[u8], &[u8]) {
    match content.iter().position(|b| *b == b'\n') {
        Some(end) => content.split_at(end + 1),
        None => (content, &[]),
    }
}

/// Return whether a text object body sets its own fill before its `ET`.
fn sets_fill(body: &[u8]) -> bool {
    body.split(|b| *b == b'\n')
        .take_while(|line| line.trim_ascii() != b"ET")
        .any(|line| line.trim_ascii_end().ends_with(b" rg"))
}

#[cfg(test)]
mod tests {
    use super::recolor_content;
    use crate::constants::{TEXT_FILL, TEXT_STROKE};

    /// Return `content` recolored, as a string.
    fn recolor(content: &str) -> String {
        String::from_utf8_lossy(&recolor_content(content.as_bytes())).to_string()
    }

    /// Return `bytes` as a string without its trailing newline.
    fn op(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).trim_end().to_string()
    }

    #[test]
    /// Assert math black fill and stroke become the theme text color.
    fn math_black_becomes_text_color() {
        assert_eq!(
            recolor("0 0 0 rg\n1 2 3 4 re\nf\n0.4 w\n0 0 0 RG\n"),
            format!(
                "{}\n1 2 3 4 re\nf\n0.4 w\n{}\n",
                op(TEXT_FILL),
                op(TEXT_STROKE)
            )
        );
    }

    #[test]
    /// Assert black inside an inline SVG group is kept, and recolored after it closes.
    fn svg_black_is_kept() {
        let svg = "q\n1 0 0 -1 72 700 cm\nq\n0 0 0 rg\nQ\nBT\n0 0 0 rg\n(A) Tj\nET\nQ\n";
        assert_eq!(recolor(svg), svg);
        assert_eq!(
            recolor(&format!("{svg}0 0 0 rg\n")),
            format!("{svg}{}\n", op(TEXT_FILL))
        );
    }

    #[test]
    /// Assert a text object without a fill gains the theme text fill.
    fn unpainted_text_gains_text_fill() {
        assert_eq!(
            recolor("BT\n/liberationsans 12 Tf\n<0001> Tj\nET"),
            format!(
                "BT\n{}\n/liberationsans 12 Tf\n<0001> Tj\nET",
                op(TEXT_FILL)
            )
        );
    }

    #[test]
    /// Assert other colors and text objects with their own fill are left alone.
    fn own_colors_are_left_alone() {
        for content in [
            "0.5 0.5 0.5 rg\n",
            "10 0 0 rg\n",
            "BT\n/Courier 12 Tf\n0 Tr\n0.5 0.5 1 rg\n(x) Tj\nET\n",
        ] {
            assert_eq!(recolor(content), content);
        }
    }
}
