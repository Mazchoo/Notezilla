//! Flattens layout-rs node and edge labels onto explicit alphabetic baselines.
//!
//! Ironpress ignores `dominant-baseline`, `<tspan>` positioning, and
//! `<textPath>`, so every label line becomes its own `<text>` element with an
//! absolute baseline.

use crate::constants::{
    BASELINE_FROM_CENTER, DEFAULT_FONT_SIZE, GRAPHVIZ_EDGE_LABEL_GAP, GRAPHVIZ_LABEL_FONT_FAMILY,
};
use crate::rendering::escape_svg_text;
use crate::rendering::svg_attr::{svg_attr, svg_attr_f64};
use crate::rendering::svg_path_ends::path_midpoint;
use crate::theme::current_theme;
use std::collections::HashMap;

/// Collect the `d` geometry of every `<path>` that has an `id`, keyed by id.
pub fn parse_path_geometry(svg: &str) -> HashMap<String, String> {
    let mut paths = HashMap::new();
    let mut rest = svg;
    while let Some(start) = rest.find("<path") {
        rest = &rest[start..];
        let end = rest.find('>').unwrap_or(rest.len());
        let open = &rest[..end];
        if let (Some(id), Some(d)) = (svg_attr(open, "id"), svg_attr(open, "d")) {
            paths.insert(id, d);
        }
        rest = &rest[end..];
    }
    paths
}

/// Flatten a `<text><textPath>` edge label into an upright positioned `<text>`.
///
/// layout-rs centres the label at 50% of the referenced edge path. The label
/// sits just above a mostly horizontal edge and just right of a mostly
/// vertical one. Returns an empty string for an edge without a label, and the
/// element unchanged when the referenced path cannot be resolved.
pub fn flatten_edge_label(
    element: &str,
    paths: &HashMap<String, String>,
    fonts: &HashMap<String, usize>,
) -> String {
    let Some(path_at) = element.find("<textPath") else {
        return element.to_string();
    };
    let inner = &element[path_at..];
    let gt = inner.find('>').unwrap_or(inner.len());
    let open = &inner[..gt];
    let text = inner[gt..]
        .trim_start_matches('>')
        .split("</textPath>")
        .next()
        .unwrap_or_default()
        .trim();
    if text.is_empty() {
        return String::new();
    }
    let Some(mid) = svg_attr(open, "href")
        .and_then(|href| paths.get(href.trim_start_matches('#')))
        .and_then(|d| path_midpoint(d))
    else {
        return element.to_string();
    };
    let font_size = label_font_size(open, fonts);
    let gap = font_size * GRAPHVIZ_EDGE_LABEL_GAP;
    let (x, y) = mid.point;
    if mid.dir.1.abs() > mid.dir.0.abs() {
        let baseline = y + font_size * BASELINE_FROM_CENTER;
        label_text_element("start", x + gap, baseline, font_size, text)
    } else {
        label_text_element("middle", x, y - gap, font_size, text)
    }
}

/// Parse the `.aN` CSS font-size classes layout-rs emits, keyed by class name.
pub fn parse_font_classes(svg: &str) -> HashMap<String, usize> {
    let mut fonts = HashMap::new();
    let mut rest = svg;
    while let Some(at) = rest.find(".a") {
        rest = &rest[at + 2..];
        let size: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if size.is_empty() {
            continue;
        }
        if let Ok(px) = size.parse::<usize>() {
            fonts.insert(format!("a{size}"), px);
        }
    }
    fonts
}

/// Flatten one `<text>` element into one `<text>` per label line.
///
/// Returns an empty string for a label with no text content, and the element
/// unchanged when it has no `x`/`y` position to lay lines out from.
pub fn flatten_node_label(element: &str, fonts: &HashMap<String, usize>) -> String {
    let gt = element.find('>').unwrap_or(element.len());
    let open = &element[..gt];
    let (Some(x), Some(y)) = (svg_attr_f64(open, "x"), svg_attr_f64(open, "y")) else {
        return element.to_string();
    };
    let font_size = label_font_size(open, fonts);

    let lines = tspan_lines(&element[gt + 1..]);
    if lines.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for (index, (line_x, text)) in lines.iter().enumerate() {
        let baseline = line_baseline(y, font_size, index, lines.len());
        out.push_str(&label_text_element(
            "middle",
            line_x.unwrap_or(x),
            baseline,
            font_size,
            text,
        ));
    }
    out
}

/// Return the font size of a label from its CSS class, else the default size.
fn label_font_size(open_tag: &str, fonts: &HashMap<String, usize>) -> f64 {
    svg_attr(open_tag, "class")
        .and_then(|class| fonts.get(&class).copied())
        .unwrap_or(DEFAULT_FONT_SIZE) as f64
}

/// Return the baseline of label line `index` of `count` lines.
///
/// layout-rs places `y` above the node center by half the block height, so the
/// lines are re-centered and then dropped onto their alphabetic baselines.
fn line_baseline(y: f64, font_size: f64, index: usize, count: usize) -> f64 {
    let count = count as f64;
    let center_y = y + (count + 1.0) * font_size / 2.0;
    let offset = index as f64 - (count - 1.0) / 2.0;
    center_y + offset * font_size + font_size * BASELINE_FROM_CENTER
}

/// Build one `<text>` element for a label line, aligned by `anchor`.
///
/// `text` is re-escaped so a label that reaches here unescaped cannot close
/// the element; layout-rs character references are kept as they are.
fn label_text_element(anchor: &str, x: f64, baseline: f64, font_size: f64, text: &str) -> String {
    let fill = escape_svg_text(current_theme().palette().text);
    let text = escape_svg_text(text);
    format!(
        "<text text-anchor=\"{anchor}\" x=\"{x}\" y=\"{baseline}\" font-size=\"{font_size}px\" \
         font-family=\"{GRAPHVIZ_LABEL_FONT_FAMILY}\" fill=\"{fill}\">{text}</text>"
    )
}

/// Collect label lines from `<tspan>` children, else from plain text content.
fn tspan_lines(inner: &str) -> Vec<(Option<f64>, String)> {
    let mut lines = Vec::new();
    let mut rest = inner;
    while let Some(start) = rest.find("<tspan") {
        let Some(open_end) = rest[start..].find('>') else {
            break;
        };
        let open = &rest[start..start + open_end];
        let content_at = start + open_end + 1;
        let Some(close_rel) = rest[content_at..].find("</tspan>") else {
            break;
        };
        let content = rest[content_at..content_at + close_rel].trim();
        if !content.is_empty() {
            lines.push((svg_attr_f64(open, "x"), content.to_string()));
        }
        rest = &rest[content_at + close_rel + "</tspan>".len()..];
    }
    if lines.is_empty() {
        let text = inner.trim().trim_end_matches("</text>").trim();
        if !text.is_empty() && !text.contains('<') {
            lines.push((None, text.to_string()));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::{
        flatten_edge_label, flatten_node_label, line_baseline, parse_font_classes,
        parse_path_geometry, tspan_lines, BASELINE_FROM_CENTER, GRAPHVIZ_LABEL_FONT_FAMILY,
    };
    use crate::constants::TEXT;
    use std::collections::HashMap;

    /// One edge path keyed by id, for edge label tests.
    fn edge_paths(id: &str, d: &str) -> HashMap<String, String> {
        HashMap::from([(id.to_string(), d.to_string())])
    }

    #[test]
    /// Assert path geometry is collected by id and paths without an id are skipped.
    fn collects_path_geometry_by_id() {
        let paths =
            parse_path_geometry(r#"<path id="arrow0" d="M 0 0 L 1 0" /><path d="M 5 5" />"#);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths.get("arrow0").map(String::as_str), Some("M 0 0 L 1 0"));
    }

    #[test]
    /// Assert a horizontal edge label is centred above the edge midpoint.
    fn horizontal_edge_label_sits_above_midpoint() {
        let html = flatten_edge_label(
            r##"<text><textPath href="#arrow0" startOffset="50%" class="a14">hi</textPath></text>"##,
            &edge_paths("arrow0", "M 0 50 L 100 50"),
            &HashMap::from([("a14".to_string(), 14)]),
        );
        assert!(!html.contains("textPath"), "{html}");
        assert!(html.contains("text-anchor=\"middle\""), "{html}");
        assert!(html.contains("x=\"50\""), "{html}");
        assert!(html.contains("y=\"46.5\""), "{html}");
        assert!(html.contains(">hi</text>"), "{html}");
    }

    #[test]
    /// Assert a vertical edge label starts just right of the edge midpoint.
    fn vertical_edge_label_sits_beside_midpoint() {
        let html = flatten_edge_label(
            r##"<text><textPath href="#arrow0" class="a14">hi</textPath></text>"##,
            &edge_paths("arrow0", "M 10 0 L 10 100"),
            &HashMap::from([("a14".to_string(), 14)]),
        );
        assert!(html.contains("text-anchor=\"start\""), "{html}");
        assert!(html.contains("x=\"13.5\""), "{html}");
        assert!(
            html.contains(&format!("y=\"{}\"", 50.0 + 14.0 * BASELINE_FROM_CENTER)),
            "{html}"
        );
    }

    #[test]
    /// Assert an edge without a label is dropped.
    fn unlabelled_edge_is_dropped() {
        let html = flatten_edge_label(
            r##"<text><textPath href="#arrow0" class="a14"></textPath></text>"##,
            &edge_paths("arrow0", "M 0 0 L 1 0"),
            &HashMap::new(),
        );
        assert_eq!(html, "");
    }

    #[test]
    /// Assert a label whose path cannot be resolved is left untouched.
    fn unresolved_edge_label_is_unchanged() {
        let element = r##"<text><textPath href="#missing">hi</textPath></text>"##;
        assert_eq!(
            flatten_edge_label(element, &HashMap::new(), &HashMap::new()),
            element
        );
    }

    #[test]
    /// Assert `.aN` style rules become font sizes keyed by class name.
    fn parses_font_size_classes() {
        let fonts = parse_font_classes(".a14 { font-size: 14px } .a20 { font-size: 20px }");
        assert_eq!(fonts.get("a14"), Some(&14));
        assert_eq!(fonts.get("a20"), Some(&20));
    }

    #[test]
    /// Assert a single line sits one baseline offset below the node center.
    fn single_line_baseline_sits_below_center() {
        // layout-rs reports y one font size above the center for one line.
        let font_size = 14.0;
        let y = 100.0;
        let center = y + font_size;
        assert_eq!(
            line_baseline(y, font_size, 0, 1),
            center + font_size * BASELINE_FROM_CENTER
        );
    }

    #[test]
    /// Assert consecutive lines are one font size apart.
    fn line_baselines_are_one_font_size_apart() {
        let first = line_baseline(100.0, 14.0, 0, 2);
        let second = line_baseline(100.0, 14.0, 1, 2);
        assert!((second - first - 14.0).abs() < 1e-9, "{first} {second}");
    }

    #[test]
    /// Assert tspan children become one line each, keeping their own `x`.
    fn tspan_children_become_lines() {
        let lines = tspan_lines(r#"<tspan x="5">one</tspan><tspan x="6">two</tspan>"#);
        assert_eq!(
            lines,
            vec![
                (Some(5.0), "one".to_string()),
                (Some(6.0), "two".to_string())
            ]
        );
    }

    #[test]
    /// Assert plain text content becomes one line without an `x`.
    fn plain_text_becomes_one_line() {
        assert_eq!(tspan_lines("A</text>"), vec![(None, "A".to_string())]);
    }

    #[test]
    /// Assert an empty label yields no lines.
    fn empty_label_has_no_lines() {
        assert!(tspan_lines("</text>").is_empty());
    }

    #[test]
    /// Assert a flattened label inlines fill, font family, and font size.
    fn flattened_label_inlines_paint_and_font() {
        let mut fonts = HashMap::new();
        fonts.insert("a14".to_string(), 14);
        let html = flatten_node_label(r#"<text x="10" y="20" class="a14">A</text>"#, &fonts);
        assert!(html.contains(&format!("fill=\"{TEXT}\"")), "{html}");
        assert!(html.contains("font-size=\"14px\""), "{html}");
        assert!(
            html.contains(&format!("font-family=\"{GRAPHVIZ_LABEL_FONT_FAMILY}\"")),
            "{html}"
        );
        assert!(html.contains(">A</text>"), "{html}");
        assert!(!html.contains("class="), "{html}");
    }

    #[test]
    /// Assert an unknown class falls back to the default font size.
    fn unknown_class_uses_default_font_size() {
        let html = flatten_node_label(r#"<text x="1" y="2" class="zz">A</text>"#, &HashMap::new());
        assert!(html.contains("font-size=\"14px\""), "{html}");
    }

    #[test]
    /// Assert a label with no position is left untouched.
    fn label_without_position_is_unchanged() {
        let element = "<text>A</text>";
        assert_eq!(flatten_node_label(element, &HashMap::new()), element);
    }

    #[test]
    /// Assert an empty label is dropped rather than emitted as an empty element.
    fn empty_label_is_dropped() {
        assert_eq!(
            flatten_node_label(r#"<text x="1" y="2"></text>"#, &HashMap::new()),
            ""
        );
    }

    #[test]
    /// Assert raw markup in a tspan line cannot close the `<text>` element.
    fn node_label_markup_is_escaped() {
        let html = flatten_node_label(
            r#"<text x="1" y="2"><tspan x="1">a</text><b>&amp;</tspan></text>"#,
            &HashMap::new(),
        );
        assert!(!html.contains("<b>"), "{html}");
        assert!(html.contains(">a&lt;/text&gt;&lt;b&gt;&amp;</text>"), "{html}");
    }

    #[test]
    /// Assert raw markup in an edge label is escaped once.
    fn edge_label_markup_is_escaped() {
        let html = flatten_edge_label(
            r##"<text><textPath href="#arrow0">a<b> &lt;</textPath></text>"##,
            &edge_paths("arrow0", "M 0 0 L 100 0"),
            &HashMap::new(),
        );
        assert!(html.contains(">a&lt;b&gt; &lt;</text>"), "{html}");
    }
}
