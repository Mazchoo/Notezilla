//! Renders a Graphviz DOT block to inline SVG.

use super::graphviz_label::{flatten_node_label, parse_font_classes};
use super::render_error::render_error_html;
use super::{Render, RenderPdf};
use crate::constants::{
    BG_3 as NODE_FILL, GRAPHVIZ_ERROR_CLASS, SVG_XML_DECLARATION, TEXT as STROKE,
};
use crate::rendering::svg_text_elements::rewrite_text_elements;
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

/// Flatten graphviz labels and map layout-rs default paints onto the palette.
fn prepare_svg(raw: &str) -> String {
    let body = strip_xml_declaration(raw);
    let fonts = parse_font_classes(body);
    let labeled = rewrite_text_elements(body, |element| {
        // Edge labels riding a `<textPath>` keep their own positioning.
        if element.contains("<textPath") {
            element.to_string()
        } else {
            flatten_node_label(element, &fonts)
        }
    });
    recolor_default_paints(&labeled)
}

/// Return `raw` without the layout-rs XML declaration.
fn strip_xml_declaration(raw: &str) -> &str {
    raw.trim_start()
        .strip_prefix(SVG_XML_DECLARATION)
        .unwrap_or(raw)
        .trim_start()
}

/// Replace layout-rs default paints with the editor palette.
fn recolor_default_paints(svg: &str) -> String {
    svg.replace("fill=\"#ffffffff\"", &format!("fill=\"{NODE_FILL}\""))
        .replace("stroke=\"#000000ff\"", &format!("stroke=\"{STROKE}\""))
        .replace("fill=\"context-stroke\"", &format!("fill=\"{STROKE}\""))
}

#[cfg(test)]
mod tests {
    use super::{
        recolor_default_paints, strip_xml_declaration, GraphvizRender, Render, RenderPdf,
        GRAPHVIZ_ERROR_CLASS, NODE_FILL, STROKE, SVG_XML_DECLARATION,
    };
    use crate::constants::BASELINE_FROM_CENTER;

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
