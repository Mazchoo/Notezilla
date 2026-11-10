//! Renders a Graphviz DOT block to inline SVG.

use super::graphviz_label::{
    flatten_edge_label, flatten_node_label, parse_font_classes, parse_path_geometry,
};
use super::marker_arrow::expand_path_markers;
use super::render_error::render_error_html;
use super::{Render, RenderPdf};
use crate::constants::{GRAPHVIZ_ERROR_CLASS, SVG_XML_DECLARATION};
use crate::rendering::svg_text_elements::rewrite_text_elements;
use crate::theme::current_theme;
use layout::backends::svg::SVGWriter;
use layout::gv::{DotParser, GraphBuilder};

/// Graphviz DOT diagram render.
pub struct GraphvizRender;

impl GraphvizRender {
    /// Render DOT source to an inline SVG string.
    ///
    /// Returns the layout-rs parse error when `dot` is not valid DOT source.
    pub fn render_svg(&self, dot: &str) -> Result<String, String> {
        let mut parser = DotParser::new(dot);
        let tree = parser.process().map_err(|e| format!("{e:?}"))?;
        let mut builder = GraphBuilder::new();
        builder.visit_graph(&tree);
        let mut graph = builder.get();
        let mut writer = SVGWriter::new();
        graph.do_it(false, false, false, &mut writer);
        Ok(prepare_svg(&writer.finalize()))
    }
}

impl Render for GraphvizRender {
    /// Return inline SVG for `source`, or a fallback block on parse failure.
    fn render(&self, source: &str) -> String {
        self.render_svg(source)
            .unwrap_or_else(|error| render_error_html(GRAPHVIZ_ERROR_CLASS, source, &error))
    }
}

impl RenderPdf for GraphvizRender {}

/// Flatten graphviz labels, map layout-rs default paints onto the palette, and
/// paint arrowheads as polygons.
///
/// Ironpress ignores `marker-start` / `marker-end` and `<textPath>`, so PDF
/// edges would otherwise be bare strokes without labels. Arrows are expanded
/// after recoloring so they take the palette stroke.
fn prepare_svg(raw: &str) -> String {
    let body = strip_marker_defs(strip_xml_declaration(raw));
    let fonts = parse_font_classes(&body);
    let paths = parse_path_geometry(&body);
    let labeled = rewrite_text_elements(&body, |element| {
        if element.contains("<textPath") {
            flatten_edge_label(element, &paths, &fonts)
        } else {
            flatten_node_label(element, &fonts)
        }
    });
    expand_path_markers(&recolor_default_paints(&labeled))
}

/// Return `svg` without the layout-rs `<defs>` block of arrow markers.
///
/// Every marker reference is replaced by a polygon, so the definitions are dead.
fn strip_marker_defs(svg: &str) -> String {
    let Some(start) = svg.find("<defs>") else {
        return svg.to_string();
    };
    let Some(len) = svg[start..].find("</defs>") else {
        return svg.to_string();
    };
    let end = start + len + "</defs>".len();
    format!("{}{}", &svg[..start], &svg[end..])
}

/// Return `raw` without the layout-rs XML declaration.
fn strip_xml_declaration(raw: &str) -> &str {
    raw.trim_start()
        .strip_prefix(SVG_XML_DECLARATION)
        .unwrap_or(raw)
        .trim_start()
}

/// Replace black and white `fill` / `stroke` paints with the editor palette.
///
/// Matching is by colour value, not by attribute, so `fillcolor=black` and
/// `color=white` follow the theme like the layout-rs defaults do. Black (and
/// `context-stroke`) maps to `--text`; white maps to `--bg-3`.
fn recolor_default_paints(svg: &str) -> String {
    let palette = current_theme().palette();
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some((value_start, value_end)) = next_paint_value(rest) {
        out.push_str(&rest[..value_start]);
        let value = &rest[value_start..value_end];
        out.push_str(palette_paint(value, palette.text, palette.bg_3).unwrap_or(value));
        rest = &rest[value_end..];
    }
    out.push_str(rest);
    out
}

/// Return the byte range of the next `fill="…"` or `stroke="…"` value in `svg`.
fn next_paint_value(svg: &str) -> Option<(usize, usize)> {
    let mut offset = 0;
    while let Some(eq) = svg[offset..].find("=\"").map(|i| offset + i) {
        let name_start = svg[..eq]
            .rfind(|c: char| c.is_ascii_whitespace())
            .map_or(0, |i| i + 1);
        let value_start = eq + 2;
        let value_end = value_start + svg[value_start..].find('"')?;
        if matches!(&svg[name_start..eq], "fill" | "stroke") {
            return Some((value_start, value_end));
        }
        offset = value_end + 1;
    }
    None
}

/// Return the palette token for a black or white SVG paint value.
fn palette_paint(value: &str, text: &'static str, bg: &'static str) -> Option<&'static str> {
    let value = value.trim();
    let is = |names: &[&str]| names.iter().any(|n| value.eq_ignore_ascii_case(n));
    if is(&["black", "#000", "#000000", "#000000ff", "context-stroke"]) {
        Some(text)
    } else if is(&["white", "#fff", "#ffffff", "#ffffffff"]) {
        Some(bg)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        recolor_default_paints, strip_marker_defs, strip_xml_declaration, GraphvizRender, Render,
        RenderPdf, GRAPHVIZ_ERROR_CLASS, SVG_XML_DECLARATION,
    };
    use crate::constants::{BASELINE_FROM_CENTER, BG_3 as NODE_FILL, TEXT as STROKE};

    #[test]
    /// Assert the XML declaration is removed so the SVG can be inlined.
    fn strips_xml_declaration() {
        let raw = format!("{SVG_XML_DECLARATION}\n<svg></svg>");
        assert_eq!(strip_xml_declaration(&raw), "<svg></svg>");
    }

    #[test]
    /// Assert SVG without an XML declaration is returned unchanged.
    fn keeps_svg_without_declaration() {
        assert_eq!(strip_xml_declaration("<svg></svg>"), "<svg></svg>");
    }

    #[test]
    /// Assert layout-rs default paints map onto the editor palette.
    fn recolors_default_paints() {
        let out = recolor_default_paints(
            r##"<ellipse fill="#ffffffff" stroke="#000000ff"/><polygon fill="context-stroke"/>"##,
        );
        assert!(out.contains(&format!("fill=\"{NODE_FILL}\"")), "{out}");
        assert!(out.contains(&format!("stroke=\"{STROKE}\"")), "{out}");
        assert!(!out.contains("context-stroke"), "{out}");
    }

    #[test]
    /// Assert black and white map by value on either paint attribute.
    fn recolors_black_and_white_on_any_paint_attribute() {
        let out = recolor_default_paints(
            r##"<rect fill="#000000ff" stroke="#ffffffff"/><path stroke="BLACK" fill="#fff"/>"##,
        );
        assert_eq!(
            out,
            format!(
                r#"<rect fill="{STROKE}" stroke="{NODE_FILL}"/><path stroke="{STROKE}" fill="{NODE_FILL}"/>"#
            )
        );
    }

    #[test]
    /// Assert other colours and non-paint attributes are left untouched.
    fn keeps_other_colours_and_attributes() {
        let input = r##"<rect fill="#ff0000ff" stroke-width="1" data-fill="#000" class="white"/>"##;
        assert_eq!(recolor_default_paints(input), input);
    }

    #[test]
    /// Assert an explicit black node fill and white edge follow the palette.
    fn explicit_fill_and_edge_colours_use_palette() {
        let svg = GraphvizRender
            .render(r#"digraph { A [style=filled fillcolor=black]; A -> B [color=white] }"#)
            .to_ascii_lowercase();
        assert!(!svg.contains("#000000"), "{svg}");
        assert!(!svg.contains("#ffffff"), "{svg}");
    }

    #[test]
    /// Assert an explicit black stroke is remapped to the page text colour.
    fn explicit_black_stroke_uses_page_text_colour() {
        let svg = GraphvizRender.render(r#"digraph { A -> B [color=black] }"#);
        assert!(
            !svg.to_ascii_lowercase().contains("#000000"),
            "black stroke must use the palette token: {svg}"
        );
    }

    #[test]
    /// Assert a rendered digraph inlines label paint, font size, and baseline.
    fn digraph_inlines_label_paint_and_baseline() {
        let svg = GraphvizRender.render_svg("digraph { A -> B }").unwrap();
        assert!(!svg.contains("<?xml"), "{svg}");
        assert!(
            !svg.contains("dominant-baseline") && !svg.contains("<tspan"),
            "labels must not rely on tspan or dominant-baseline: {svg}"
        );
        assert!(svg.contains(&format!("fill=\"{STROKE}\"")), "{svg}");
        assert!(svg.contains("font-size=\"14px\""), "{svg}");
        assert!(svg.contains(&format!("fill=\"{NODE_FILL}\"")), "{svg}");

        let cy: f64 = svg
            .split("cy=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
            .expect("ellipse cy");
        let text_y: f64 = svg
            .split("<text ")
            .nth(1)
            .and_then(|s| s.split("y=\"").nth(1))
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
            .expect("text y");
        let expected = cy + 14.0 * BASELINE_FROM_CENTER;
        assert!(
            (text_y - expected).abs() < 0.6,
            "label baseline {text_y} should sit near ellipse cy {cy} (expected {expected}): {svg}"
        );
    }

    #[test]
    /// Assert graphviz arrows are painted polygons, not marker references.
    fn arrows_are_polygons_not_marker_references() {
        let svg = GraphvizRender.render("digraph { A -> B }");
        assert!(
            svg.contains("<polygon"),
            "arrowheads must be polygons: {svg}"
        );
        assert!(svg.contains(&format!("fill=\"{STROKE}\"")), "{svg}");
        assert!(!svg.contains("marker-end"), "{svg}");
        assert!(!svg.contains("marker-start"), "{svg}");
        assert!(!svg.contains("<marker"), "{svg}");
        assert!(!svg.contains("<defs>"), "{svg}");
    }

    #[test]
    /// Assert an edge label is a positioned `<text>`, not a `<textPath>`.
    fn edge_labels_are_positioned_text() {
        let svg = GraphvizRender.render(r#"digraph { A -> B [label="hello"] }"#);
        assert!(svg.contains(">hello</text>"), "{svg}");
        assert!(!svg.contains("<textPath"), "{svg}");
    }

    #[test]
    /// Assert edges without a label leave no empty text elements behind.
    fn unlabelled_edges_emit_no_text() {
        let svg = GraphvizRender.render("digraph { A -> B }");
        assert_eq!(svg.matches("<text ").count(), 2, "{svg}");
        assert!(!svg.contains("<text>"), "{svg}");
    }

    #[test]
    /// Assert the marker `<defs>` block is removed and surrounding markup kept.
    fn strips_marker_defs() {
        let out = strip_marker_defs("<svg><defs><marker/></defs><g/></svg>");
        assert_eq!(out, "<svg><g/></svg>");
    }

    #[test]
    /// Assert invalid DOT source renders a fallback block instead of failing.
    fn invalid_dot_renders_fallback_block() {
        let source = "digraph { A ->";
        assert!(GraphvizRender.render_svg(source).is_err());
        let html = GraphvizRender.render(source);
        assert!(
            html.contains(&format!("class=\"{GRAPHVIZ_ERROR_CLASS}\"")),
            "{html}"
        );
        assert!(html.contains("digraph { A -&gt;"), "{html}");
    }

    #[test]
    /// Assert PDF export reuses the editor SVG.
    fn pdf_html_matches_editor_html() {
        let source = "digraph { A -> B }";
        assert_eq!(
            GraphvizRender.render_pdf(source),
            GraphvizRender.render(source)
        );
    }
}
