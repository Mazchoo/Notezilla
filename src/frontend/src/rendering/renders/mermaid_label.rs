//! Flattens Mermaid labels onto explicit alphabetic baselines.
//!
//! Ironpress ignores `dominant-baseline` and `<tspan>` positioning, so every
//! label line becomes its own `<text>` element with an absolute baseline.

use crate::constants::{
    BASELINE_ASCENT_RATIO, DEFAULT_FONT_SIZE, MONOSPACE_ADVANCE_RATIO, PDF_FONT_FAMILY, TEXT,
};
use crate::rendering::svg_attr::{svg_attr, svg_attr_f64};

/// One laid-out line of a Mermaid label.
struct LabelLine {
    x: f64,
    y: f64,
    runs: Vec<LabelRun>,
}

/// A span of a label line sharing one emphasis.
struct LabelRun {
    text: String,
    bold: bool,
    italic: bool,
}

/// Text attributes shared by every element of one flattened label.
struct LabelStyle<'a> {
    font_size: f64,
    anchor: &'a str,
    fill: &'a str,
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
    let style = LabelStyle {
        font_size,
        anchor: &anchor,
        fill: &fill,
        bold: svg_attr(open, "font-weight").as_deref() == Some("bold"),
        italic: svg_attr(open, "font-style").as_deref() == Some("italic"),
    };

    let mut out = String::new();
    for line in label_lines(&element[gt + 1..], x, y) {
        push_line_elements(&mut out, &line, &style);
    }
    out
}

/// Append the `<text>` elements of one label line on its alphabetic baseline.
///
/// A single-style line is one element at the line's anchor. A line mixing
/// emphasis becomes one start-anchored element per run, offset by the
/// monospace advance of the runs before it.
fn push_line_elements(out: &mut String, line: &LabelLine, style: &LabelStyle) {
    let baseline = line.y + style.font_size * BASELINE_ASCENT_RATIO;
    if let [run] = line.runs.as_slice() {
        push_text_element(out, style.anchor, line.x, baseline, run, style);
        return;
    }
    let advance = style.font_size * MONOSPACE_ADVANCE_RATIO;
    let total: usize = line.runs.iter().map(|run| glyph_count(&run.text)).sum();
    let width = total as f64 * advance;
    let mut x = match style.anchor {
        "end" => line.x - width,
        "start" => line.x,
        _ => line.x - width / 2.0,
    };
    for run in &line.runs {
        push_text_element(out, "start", x, baseline, run, style);
        x += glyph_count(&run.text) as f64 * advance;
    }
}

/// Append one `<text>` element holding `run` at (`x`, `baseline`).
fn push_text_element(
    out: &mut String,
    anchor: &str,
    x: f64,
    baseline: f64,
    run: &LabelRun,
    style: &LabelStyle,
) {
    out.push_str(&format!(
        "<text text-anchor=\"{anchor}\" x=\"{x}\" y=\"{baseline}\" font-size=\"{}px\" \
         font-family=\"{PDF_FONT_FAMILY}\" fill=\"{}\"",
        style.font_size, style.fill
    ));
    if style.bold || run.bold {
        out.push_str(" font-weight=\"bold\"");
    }
    if style.italic || run.italic {
        out.push_str(" font-style=\"italic\"");
    }
    out.push('>');
    out.push_str(&run.text);
    out.push_str("</text>");
}

/// Count rendered glyphs in escaped text, treating `&…;` entities as one.
fn glyph_count(text: &str) -> usize {
    let mut count = 0;
    let mut in_entity = false;
    for c in text.chars() {
        match c {
            '&' if !in_entity => {
                in_entity = true;
                count += 1;
            }
            ';' if in_entity => in_entity = false,
            _ if !in_entity => count += 1,
            _ => {}
        }
    }
    count
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
                runs: vec![LabelRun {
                    text,
                    bold: svg_attr(open, "font-weight").as_deref() == Some("bold"),
                    italic: svg_attr(open, "font-style").as_deref() == Some("italic"),
                }],
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
        runs: vec![LabelRun {
            text,
            bold: false,
            italic: false,
        }],
    }]
}

/// Join styled runs of one line into a single line at the parent's `y`.
///
/// Adjacent runs with the same emphasis merge, so a single-style line keeps
/// one run and mixed emphasis keeps one run per style change.
fn join_runs(lines: Vec<LabelLine>, parent_y: f64) -> LabelLine {
    let x = lines.first().map_or(0.0, |line| line.x);
    let mut runs: Vec<LabelRun> = Vec::new();
    for run in lines.into_iter().flat_map(|line| line.runs) {
        match runs.last_mut() {
            Some(last) if last.bold == run.bold && last.italic == run.italic => {
                last.text.push_str(&run.text);
            }
            _ => runs.push(run),
        }
    }
    LabelLine {
        x,
        y: parent_y,
        runs,
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
    use super::{flatten_label, glyph_count, label_lines, parse_font_px, strip_tags};
    use crate::constants::{BASELINE_ASCENT_RATIO, MONOSPACE_ADVANCE_RATIO, PDF_FONT_FAMILY, TEXT};

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
        assert_eq!(lines[0].runs.len(), 1);
        assert_eq!(lines[0].runs[0].text, "abcd");
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
        assert_eq!(lines[0].runs[0].text, "A");
        assert_eq!(lines[0].x, 3.0);
    }

    #[test]
    /// Assert a bold run among plain runs stays bold.
    fn mixed_emphasis_is_not_dropped() {
        let html = flatten_label(
            r#"<text x="1" y="2"><tspan>A</tspan><tspan font-weight="bold">B</tspan></text>"#,
        );
        assert!(html.contains("font-weight=\"bold\""), "{html}");
        assert!(html.contains(">B</text>"), "{html}");
        assert!(!html.contains(">AB<"), "{html}");
    }

    #[test]
    /// Assert mixed-emphasis runs are laid out left to right around the anchor.
    fn mixed_emphasis_runs_are_offset_by_monospace_advance() {
        let html = flatten_label(
            r#"<text x="100" y="2" font-size="10px"><tspan>ab</tspan><tspan font-style="italic">c</tspan><tspan>d</tspan></text>"#,
        );
        let advance = 10.0 * MONOSPACE_ADVANCE_RATIO;
        let left = 100.0 - 4.0 * advance / 2.0;
        assert_eq!(html.matches("<text ").count(), 3, "{html}");
        assert!(!html.contains("text-anchor=\"middle\""), "{html}");
        assert!(html.contains(&format!("x=\"{left}\"")), "{html}");
        assert!(
            html.contains(&format!("x=\"{}\"", left + 2.0 * advance)),
            "{html}"
        );
        assert!(
            html.contains(&format!("x=\"{}\"", left + 3.0 * advance)),
            "{html}"
        );
        assert_eq!(html.matches("font-style=\"italic\"").count(), 1, "{html}");
    }

    #[test]
    /// Assert an escaped entity counts as a single glyph.
    fn entity_counts_as_one_glyph() {
        assert_eq!(glyph_count("a&amp;b"), 3);
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
        let child =
            flatten_label(r#"<text x="1" y="2"><tspan font-style="italic">A</tspan></text>"#);
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
