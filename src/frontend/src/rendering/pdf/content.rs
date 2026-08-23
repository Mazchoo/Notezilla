//! Recolors the paint operators of a PDF content stream.
//!
//! Ironpress paints math with hardcoded black: fraction rules emit `0 0 0 rg`,
//! and math glyph runs emit no fill at all and so inherit black. Both are
//! repainted with the page text color so equations match body text.

use crate::constants::{MATH_FONTS, PDF_BLACK_FILL, PDF_BLACK_STROKE, TEXT_FILL, TEXT_STROKE};

/// Recolor black paint operators and add a fill to math text objects.
pub fn recolor_content(content: &[u8]) -> Vec<u8> {
    let replaced = replace_black_operators(content);
    insert_fill_after_bt(&replaced)
}

/// Replace `0 0 0 rg` and `0 0 0 RG` with the page text color operators.
fn replace_black_operators(content: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(content.len());
    let mut i = 0;
    while i < content.len() {
        let is_fill = content[i..].starts_with(PDF_BLACK_FILL);
        let is_stroke = content[i..].starts_with(PDF_BLACK_STROKE);
        if is_operator_start(content, i) && (is_fill || is_stroke) {
            let replacement = if is_fill { TEXT_FILL } else { TEXT_STROKE };
            out.extend_from_slice(replacement.trim_ascii_end());
            i += PDF_BLACK_FILL.len();
            continue;
        }
        out.push(content[i]);
        i += 1;
    }
    out
}

/// Insert the page text fill immediately after each math `BT` operator.
///
/// The fill has to sit inside the text object: a fill set before `BT` does not
/// apply to the glyphs ironpress emits after it.
fn insert_fill_after_bt(content: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(content.len() + 64);
    let mut i = 0;
    while i < content.len() {
        let Some(after_bt) = text_object_body_at(content, i) else {
            out.push(content[i]);
            i += 1;
            continue;
        };
        if !starts_math_font(&content[after_bt..]) {
            out.push(content[i]);
            i += 1;
            continue;
        }
        out.extend_from_slice(b"BT\n");
        if !has_text_fill(&content[after_bt..]) {
            out.extend_from_slice(TEXT_FILL);
        }
        i = after_bt;
    }
    out
}

/// Return the index of the body of a text object opened at `i`, if any.
fn text_object_body_at(content: &[u8], i: usize) -> Option<usize> {
    if !is_operator_start(content, i) || !content[i..].starts_with(b"BT") {
        return None;
    }
    let after = i + b"BT".len();
    if after != content.len() && !is_pdf_whitespace(content[after]) {
        return None;
    }
    let body = after
        + content[after..]
            .iter()
            .take_while(|b| is_pdf_whitespace(**b))
            .count();
    Some(body)
}

/// Return whether a text object body selects one of the math fonts.
fn starts_math_font(body: &[u8]) -> bool {
    MATH_FONTS.iter().any(|font| body.starts_with(font))
}

/// Return whether a text object body already sets the page text fill.
fn has_text_fill(body: &[u8]) -> bool {
    body.starts_with(TEXT_FILL) || body.starts_with(TEXT_FILL.trim_ascii_end())
}

/// Return whether index `i` starts a PDF operator token.
fn is_operator_start(content: &[u8], i: usize) -> bool {
    i == 0 || is_pdf_whitespace(content[i - 1])
}

/// Return whether `byte` separates PDF operator tokens.
fn is_pdf_whitespace(byte: u8) -> bool {
    matches!(byte, b'\n' | b'\r' | b' ' | b'\t')
}

#[cfg(test)]
mod tests {
    use super::{
        insert_fill_after_bt, is_operator_start, recolor_content, replace_black_operators,
        text_object_body_at,
    };
    use crate::constants::{TEXT_FILL, TEXT_STROKE};

    /// Return `bytes` as a string for readable assertions.
    fn text(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).to_string()
    }

    #[test]
    /// Assert a token at the start of the stream or after whitespace is an operator.
    fn operator_starts_at_stream_start_or_after_whitespace() {
        assert!(is_operator_start(b"BT", 0));
        assert!(is_operator_start(b"q\nBT", 2));
        assert!(!is_operator_start(b"xBT", 1));
    }

    #[test]
    /// Assert black fill becomes the page text fill.
    fn black_fill_becomes_text_fill() {
        let out = replace_black_operators(b"q\n0 0 0 rg\nBT");
        let expected = format!("q\n{}\nBT", text(TEXT_FILL).trim_end());
        assert_eq!(text(&out), expected);
    }

    #[test]
    /// Assert black stroke becomes the page text stroke.
    fn black_stroke_becomes_text_stroke() {
        let out = replace_black_operators(b"0 0 0 RG\n");
        let expected = format!("{}\n", text(TEXT_STROKE).trim_end());
        assert_eq!(text(&out), expected);
    }

    #[test]
    /// Assert a black operator that is part of a longer token is left alone.
    fn operator_inside_another_token_is_left_alone() {
        let content = b"10 0 0 rg\n";
        assert_eq!(replace_black_operators(content), content.to_vec());
    }

    #[test]
    /// Assert non-black paint operators are left alone.
    fn other_colors_are_left_alone() {
        let content = b"0.5 0.5 0.5 rg\n";
        assert_eq!(replace_black_operators(content), content.to_vec());
    }

    #[test]
    /// Assert a math text object body is found after `BT` and its whitespace.
    fn finds_math_text_object_body() {
        let content = b"BT\n/Symbol 12 Tf";
        let body = text_object_body_at(content, 0).expect("body");
        assert!(content[body..].starts_with(b"/Symbol"), "{body}");
    }

    #[test]
    /// Assert `BT` inside a longer token does not open a text object.
    fn bt_inside_another_token_is_not_a_text_object() {
        assert_eq!(text_object_body_at(b"BTx\n", 0), None);
        assert_eq!(text_object_body_at(b"qBT\n", 1), None);
    }

    #[test]
    /// Assert a math text object gains the page text fill.
    fn math_text_object_gains_text_fill() {
        let out = insert_fill_after_bt(b"BT\n/Helvetica-Oblique 12 Tf\n(x) Tj\nET");
        let expected = format!(
            "BT\n{}/Helvetica-Oblique 12 Tf\n(x) Tj\nET",
            text(TEXT_FILL)
        );
        assert_eq!(text(&out), expected);
    }

    #[test]
    /// Assert a non-math text object is left alone.
    fn non_math_text_object_is_left_alone() {
        let content = b"BT\n/Times-Roman 12 Tf\n(x) Tj\nET";
        assert_eq!(insert_fill_after_bt(content), content.to_vec());
    }

    #[test]
    /// Assert a text object that already sets the fill is not given a second one.
    fn existing_fill_is_not_duplicated() {
        let mut content = b"BT\n".to_vec();
        content.extend_from_slice(TEXT_FILL);
        content.extend_from_slice(b"/Symbol 12 Tf\n(\\160) Tj\nET");
        assert_eq!(insert_fill_after_bt(&content), content);
    }

    #[test]
    /// Assert recoloring both replaces black paint and fills math text objects.
    fn recolors_paint_and_math_text_objects() {
        let out = recolor_content(b"0 0 0 rg\nBT\n/Symbol 12 Tf\n(\\160) Tj\nET");
        let out = text(&out);
        let fill = text(TEXT_FILL);
        assert!(!out.contains("0 0 0 rg"), "{out}");
        assert!(out.starts_with(fill.trim_end()), "{out}");
        assert!(out.contains(&format!("BT\n{fill}/Symbol")), "{out}");
    }
}
