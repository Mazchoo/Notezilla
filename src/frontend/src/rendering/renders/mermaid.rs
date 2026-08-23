//! Renders a Mermaid diagram block to inline SVG.

use super::mermaid_arrow::expand_path_markers;
use super::mermaid_label::flatten_label;
use super::render_error::render_error_html;
use super::{Render, RenderPdf};
use crate::constants::{MERMAID_ERROR_CLASS, MERMAID_STROKE_SLOP};
use crate::rendering::svg_text_elements::rewrite_text_elements;
use rusty_mermaid::svg::SvgRenderer;
use rusty_mermaid::{render, Color, Primitive, Scene, Theme};

/// Mermaid diagram render.
pub struct MermaidRender;

impl MermaidRender {
    /// Render Mermaid source to an inline SVG string.
    ///
    /// Returns the rusty-mermaid parse error when `source` is not a diagram.
    pub fn render_svg(&self, source: &str) -> Result<String, String> {
        let theme = diagram_theme();
        let scene = render(source, &theme).map_err(|e| e.to_string())?;
        let svg_theme = padded_theme(theme, &scene);
        let svg = SvgRenderer::with_theme(&svg_theme).render_themed(&scene, &svg_theme);
        Ok(prepare_svg(&svg))
    }
}

impl Render for MermaidRender {
    /// Return inline SVG for `source`, or a fallback block on parse failure.
    fn render(&self, source: &str) -> String {
        self.render_svg(source)
            .unwrap_or_else(|error| render_error_html(MERMAID_ERROR_CLASS, source, &error))
    }
}

impl RenderPdf for MermaidRender {}

/// Return the diagram theme: the dark palette with the background suppressed.
///
/// The dark theme's text matches the editor text color, so labels read
/// correctly both on shape fills and on the diagram background. A white
/// `background` suppresses the background `<rect>` (the renderer skips it for
/// white), letting the editor surface show through.
fn diagram_theme() -> Theme {
    Theme {
        background: Color::WHITE,
        ..Theme::dark()
    }
}

/// Grow theme padding so content left of or above the origin stays in view.
///
/// rusty-mermaid can place subgraphs at negative coordinates while still
/// reporting the scene origin at (0, 0); without the extra padding the SVG
/// translate clips the left and top strokes of subgraph-only diagrams.
fn padded_theme(theme: Theme, scene: &Scene) -> Theme {
    let (min_x, min_y) = content_mins(scene);
    let overhang = (-min_x).max(-min_y);
    if overhang <= 0.0 {
        return theme;
    }
    Theme {
        padding: theme.padding.max(overhang + MERMAID_STROKE_SLOP),
        ..theme
    }
}

/// Return the lowest x/y among scene primitives that overhang the origin.
fn content_mins(scene: &Scene) -> (f64, f64) {
    let mut min_x: f64 = 0.0;
    let mut min_y: f64 = 0.0;

    for element in scene.elements() {
        match &element.primitive {
            Primitive::Rect { bbox, style, .. } => {
                let half_stroke = style.stroke_width.unwrap_or(0.0) / 2.0;
                min_x = min_x.min(bbox.left() - half_stroke);
                min_y = min_y.min(bbox.top() - half_stroke);
            }
            Primitive::Circle {
                center,
                radius,
                style,
            } => {
                let reach = radius + style.stroke_width.unwrap_or(0.0) / 2.0;
                min_x = min_x.min(center.x - reach);
                min_y = min_y.min(center.y - reach);
            }
            Primitive::Ellipse {
                center,
                rx,
                ry,
                style,
            } => {
                let half_stroke = style.stroke_width.unwrap_or(0.0) / 2.0;
                min_x = min_x.min(center.x - rx - half_stroke);
                min_y = min_y.min(center.y - ry - half_stroke);
            }
            Primitive::Text { position, .. } => {
                min_x = min_x.min(position.x);
                min_y = min_y.min(position.y);
            }
            Primitive::Polygon { points, style } => {
                let half_stroke = style.stroke_width.unwrap_or(0.0) / 2.0;
                for point in points {
                    min_x = min_x.min(point.x - half_stroke);
                    min_y = min_y.min(point.y - half_stroke);
                }
            }
            // Paths, groups, and arcs: subgraph clipping is driven by Rect
            // bounds, and edge paths sit inside those boxes.
            _ => {}
        }
    }

    (min_x, min_y)
}

/// Flatten diagram labels and expand marker arrows for ironpress layout.
fn prepare_svg(svg: &str) -> String {
    let labeled = rewrite_text_elements(svg, flatten_label);
    expand_path_markers(&labeled)
}

#[cfg(test)]
mod tests {
    use super::{
        diagram_theme, padded_theme, MermaidRender, Render, RenderPdf, MERMAID_ERROR_CLASS,
    };
    use crate::constants::PDF_FONT_FAMILY;
    use rusty_mermaid::{render, Color};

    #[test]
    /// Assert the diagram theme suppresses the background rect.
    fn diagram_theme_suppresses_background() {
        // The SVG renderer omits the background rect when it is white.
        assert_eq!(diagram_theme().background, Color::WHITE);
        let svg = MermaidRender.render_svg("graph LR\n    A --> B\n").unwrap();
        assert!(svg.contains("<svg"), "{svg}");
    }

    #[test]
    /// Assert a diagram inside the origin keeps the theme padding unchanged.
    fn padding_is_unchanged_without_overhang() {
        let theme = diagram_theme();
        let scene = render("graph LR\n    A --> B\n", &theme).unwrap();
        let expected = theme.padding;
        assert_eq!(padded_theme(theme, &scene).padding, expected);
    }

    #[test]
    /// Assert flowchart labels use an alphabetic baseline and the PDF font.
    fn flowchart_labels_use_baseline_and_pdf_font() {
        let svg = MermaidRender.render("graph LR\n    A[Square Rect] --> B((Circle))\n");
        assert!(
            !svg.contains("dominant-baseline"),
            "ironpress ignores dominant-baseline: {svg}"
        );
        assert!(
            svg.contains(&format!("font-family=\"{PDF_FONT_FAMILY}\"")),
            "{svg}"
        );
        assert!(
            svg.contains("<polygon"),
            "arrowheads must be polygons: {svg}"
        );
        assert!(!svg.contains("marker-end"), "{svg}");
        let y: f64 = svg
            .split("y=\"")
            .filter_map(|s| s.split('"').next()?.parse().ok())
            .find(|&n: &f64| n > 20.0)
            .expect("text y");
        // Labels sit on the alphabetic baseline, below the visual center.
        assert!(y > 20.0, "unexpected label y {y}: {svg}");
    }

    #[test]
    /// Assert a pie chart renders without `dominant-baseline`.
    fn pie_renders_without_dominant_baseline() {
        let svg = MermaidRender.render("pie title Pets\n\"Dogs\" : 2\n\"Cats\" : 3\n");
        assert!(svg.contains("<svg"), "{svg}");
        assert!(!svg.contains("dominant-baseline"), "{svg}");
        assert!(
            svg.contains(&format!("font-family=\"{PDF_FONT_FAMILY}\"")),
            "{svg}"
        );
    }

    #[test]
    /// Assert unparseable source renders a fallback block instead of failing.
    fn invalid_source_renders_fallback_block() {
        let source = "not a diagram at all";
        assert!(MermaidRender.render_svg(source).is_err());
        let html = MermaidRender.render(source);
        assert!(
            html.contains(&format!("class=\"{MERMAID_ERROR_CLASS}\"")),
            "{html}"
        );
        assert!(html.contains(source), "{html}");
    }

    #[test]
    /// Assert PDF export reuses the editor SVG.
    fn pdf_html_matches_editor_html() {
        let source = "graph LR\n    A --> B\n";
        assert_eq!(
            MermaidRender.render_pdf(source),
            MermaidRender.render(source)
        );
    }
}
