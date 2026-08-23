//! Flattens Mermaid labels onto explicit alphabetic baselines.
//!
//! Ironpress ignores `dominant-baseline` and `<tspan>` positioning, so every
//! label line becomes its own `<text>` element with an absolute baseline.

use crate::constants::{BASELINE_ASCENT_RATIO, DEFAULT_FONT_SIZE, PDF_FONT_FAMILY, TEXT};
use crate::rendering::svg_attr::{svg_attr, svg_attr_f64};

/// One laid-out line of a Mermaid label.
struct LabelLine {
    x: f64,
    y: f64,
    text: String,
    bold: bool,
    italic: bool,
}

/// Flatten one `<text>` element into one `<text>` per label line.
///
/// Returns an empty string for a label with no text content, and the element
/// unchanged when it has no `x`/`y` position to lay lines out from.
pub fn flatten_label(element: &str) -> String {
    let Some(gt) = element.find('>') else {
        return element.to_string();
    };
    let open = &element[..gt];
    let (Some(x), Some(y)) = (svg_attr_f64(open, "x"), svg_attr_f64(open, "y")) else {
        return element.to_string();
    };
    let font_size = svg_attr(open, "font-size")
        .and_then(|value| parse_font_px(&value))
        .unwrap_or(DEFAULT_FONT_SIZE as f64);
    let anchor = svg_attr(open, "text-anchor").unwrap_or_else(|| "middle".to_string());
    let fill = svg_attr(open, "fill").unwrap_or_else(|| TEXT.to_string());
    let parent_bold = svg_attr(open, "font-weight").as_deref() == Some("bold");
    let parent_italic = svg_attr(open, "font-style").as_deref() == Some("italic");

    let mut out = String::new();
    for line in label_lines(&element[gt + 1..], x, y) {
        out.push_str(&label_text_element(
            &line,
            font_size,
            &anchor,
            &fill,
            parent_bold,
            parent_italic,
        ));
    }
    out
}

/// Build one `<text>` element for a label line on its alphabetic baseline.
fn label_text_element(
    line: &LabelLine,
    font_size: f64,
    anchor: &str,
    fill: &str,
    parent_bold: bool,
    parent_italic: bool,
) -> String {
    let baseline = line.y + font_size * BASELINE_ASCENT_RATIO;
    let mut out = format!(
        "<text text-anchor=\"{anchor}\" x=\"{}\" y=\"{baseline}\" font-size=\"{font_size}px\" \
         font-family=\"{PDF_FONT_FAMILY}\" fill=\"{fill}\"",
        line.x
    );
    if parent_bold || line.bold {
        out.push_str(" font-weight=\"bold\"");
    }
    if parent_italic || line.italic {
        out.push_str(" font-style=\"italic\"");
    }
    out.push_str(&format!(">{}</text>", line.text));
    out
}

/// Collect label lines from `<tspan>` children, else from plain text content.
///
/// Tspans that carry a non-zero `dy` are separate lines stacked from the
/// parent's `y`; tspans without one are runs of a single line and are joined.
fn label_lines(inner: &str, default_x: f64, parent_y: f64) -> Vec<LabelLine> {
    let mut lines = Vec::new();
    let mut rest = inner;
    let mut y_cursor = parent_y;
    let mut saw_dy = false;
    while let Some(start) = rest.find("<tspan") {
        let Some(open_end) = rest[start..].find('>') else {
            break;
        };
        let open = &rest[start..start + open_end];
        let (content, next) = if open.ends_with('/') {
            ("", &rest[start + open_end + 1..])
        } else {
            let content_at = start + open_end + 1;
            let Some(close_rel) = rest[content_at..].find("</tspan>") else {
                break;
            };
            (
                rest[content_at..content_at + close_rel].trim(),
                &rest[content_at + close_rel + "</tspan>".len()..],
            )
        };
        if let Some(dy) = svg_attr_f64(open, "dy") {
            if dy != 0.0 {
                saw_dy = true;
            }
            y_cursor += dy;
        }
        let text = strip_tags(content);
        if !text.is_empty() {
            lines.push(LabelLine {
                x: svg_attr_f64(open, "x").unwrap_or(default_x),
                y: y_cursor,
                text,
                bold: svg_attr(open, "font-weight").as_deref() == Some("bold"),
                italic: svg_attr(open, "font-style").as_deref() == Some("italic"),
            });
        }
        rest = next;
    }
    if lines.is_empty() {
        return plain_text_line(inner, default_x, parent_y);
    }
    if saw_dy {
        return lines;
    }
    vec![join_runs(lines, parent_y)]
}

/// Return the label's plain text content as a single line, if it has any.
fn plain_text_line(inner: &str, x: f64, y: f64) -> Vec<LabelLine> {
    let text = strip_tags(inner.trim().trim_end_matches("</text>").trim());
    if text.is_empty() {
        return Vec::new();
    }
    vec![LabelLine {
        x,
        y,
        text,
        bold: false,
        italic: false,
    }]
}

/// Join styled runs of one line into a single line at the parent's `y`.
fn join_runs(lines: Vec<LabelLine>, parent_y: f64) -> LabelLine {
    let bold = lines.iter().all(|line| line.bold);
    let italic = lines.iter().all(|line| line.italic);
    let x = lines.first().map_or(0.0, |line| line.x);
    let text = lines.iter().map(|line| line.text.as_str()).collect();
    LabelLine {
        x,
        y: parent_y,
        text,
        bold,
        italic,
    }
}

/// Strip HTML/SVG tags from `s`, keeping the text content.
fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Parse a CSS `font-size` value in pixels, rejecting non-positive sizes.
fn parse_font_px(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_end_matches("px")
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|size| *size > 0.0)
}

#[cfg(test)]
mod tests {
    use super::{flatten_label, label_lines, parse_font_px, strip_tags};
    use crate::constants::{BASELINE_ASCENT_RATIO, PDF_FONT_FAMILY, TEXT};

    #[test]
    /// Assert a pixel font size is parsed with or without the unit.
    fn parses_font_size_in_pixels() {
        assert_eq!(parse_font_px("14px"), Some(14.0));
        assert_eq!(parse_font_px(" 12 "), Some(12.0));
    }

    #[test]
    /// Assert a non-positive or unparseable font size is rejected.
    fn rejects_unusable_font_size() {
        assert_eq!(parse_font_px("0px"), None);
        assert_eq!(parse_font_px("-4px"), None);
        assert_eq!(parse_font_px("inherit"), None);
    }

    #[test]
    /// Assert nested markup is reduced to its text content.
    fn strips_nested_markup() {
        assert_eq!(strip_tags("<b>bold</b> text"), "bold text");
    }

    #[test]
    /// Assert tspan runs without `dy` join into one line at the parent baseline.
    fn runs_without_dy_join_into_one_line() {
        let lines = label_lines(r#"<tspan x="5">ab</tspan><tspan>cd</tspan>"#, 1.0, 20.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "abcd");
        assert_eq!(lines[0].x, 5.0);
        assert_eq!(lines[0].y, 20.0);
    }

    #[test]
    /// Assert tspans with `dy` stack downwards from the parent baseline.
    fn tspans_with_dy_stack_into_lines() {
        let lines = label_lines(
            r#"<tspan dy="0">one</tspan><tspan dy="14">two</tspan>"#,
            1.0,
            20.0,
        );
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].y, 20.0);
        assert_eq!(lines[1].y, 34.0);
    }

    #[test]
    /// Assert a label without tspans yields its plain text at the parent position.
    fn plain_text_becomes_one_line() {
        let lines = label_lines("A</text>", 3.0, 20.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "A");
        assert_eq!(lines[0].x, 3.0);
    }

    #[test]
    /// Assert an empty label yields no lines.
    fn empty_label_has_no_lines() {
        assert!(label_lines("</text>", 0.0, 0.0).is_empty());
    }

    #[test]
    /// Assert the flattened label drops the baseline below the reported `y`.
    fn flattened_label_uses_alphabetic_baseline() {
        let html = flatten_label(r#"<text x="10" y="20" font-size="14px">A</text>"#);
        let expected = 20.0 + 14.0 * BASELINE_ASCENT_RATIO;
        assert!(html.contains(&format!("y=\"{expected}\"")), "{html}");
        assert!(!html.contains("dominant-baseline"), "{html}");
    }

    #[test]
    /// Assert the flattened label inlines the PDF font and the parent fill.
    fn flattened_label_inlines_font_and_fill() {
        let html = flatten_label(r##"<text x="1" y="2" fill="#ff0000">A</text>"##);
        assert!(
            html.contains(&format!("font-family=\"{PDF_FONT_FAMILY}\"")),
            "{html}"
        );
        assert!(html.contains("fill=\"#ff0000\""), "{html}");
    }

    #[test]
    /// Assert a label without a fill falls back to the page text color.
    fn label_without_fill_uses_text_color() {
        let html = flatten_label(r#"<text x="1" y="2">A</text>"#);
        assert!(html.contains(&format!("fill=\"{TEXT}\"")), "{html}");
    }

    #[test]
    /// Assert parent and tspan emphasis both reach the flattened line.
    fn keeps_bold_and_italic_emphasis() {
        let parent = flatten_label(r#"<text x="1" y="2" font-weight="bold">A</text>"#);
        assert!(parent.contains("font-weight=\"bold\""), "{parent}");
        let child = flatten_label(r#"<text x="1" y="2"><tspan font-style="italic">A</tspan></text>"#);
        assert!(child.contains("font-style=\"italic\""), "{child}");
    }

    #[test]
    /// Assert a label with no position is left untouched.
    fn label_without_position_is_unchanged() {
        let element = "<text>A</text>";
        assert_eq!(flatten_label(element), element);
    }

    #[test]
    /// Assert an empty label is dropped rather than emitted as an empty element.
    fn empty_label_is_dropped() {
        assert_eq!(flatten_label(r#"<text x="1" y="2"></text>"#), "");
    }
}
