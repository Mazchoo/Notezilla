//! Renders a Mermaid diagram block to inline SVG.

use super::mermaid_arrow::expand_path_markers;
use super::mermaid_label::flatten_label;
use super::render_error::render_error_html;
use super::{Render, RenderPdf};
use crate::constants::{MERMAID_ERROR_CLASS, MERMAID_STROKE_SLOP};
use crate::rendering::svg_text_elements::rewrite_text_elements;
use crate::theme::{current_theme, ColorTheme};
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
        Ok(prepare_svg(&svg, source))
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

/// Return the diagram theme for the current night or day palette.
///
/// Night is `Theme::dark()` with a white `background` so the renderer omits
/// the background `<rect>` and the editor surface shows through. Day uses
/// black strokes and labels on `Theme::light()`. Night ink is otherwise
/// unchanged from the original dark palette.
fn diagram_theme() -> Theme {
    match current_theme() {
        ColorTheme::Night => Theme {
            background: Color::WHITE,
            ..Theme::dark()
        },
        ColorTheme::Day => {
            let mut theme = Theme::light();
            theme.background = Color::WHITE;
            theme.node_stroke = Color::BLACK;
            theme.subgraph_stroke = Color::BLACK;
            theme.composite_stroke = Color::BLACK;
            theme.region_stroke = Color::BLACK;
            theme.node_text = Color::BLACK;
            theme.edge_stroke = Color::BLACK;
            theme.edge_label_text = Color::BLACK;
            theme.muted_text = Color::BLACK;
            theme.subgraph_label = Color::BLACK;
            theme.composite_label = Color::BLACK;
            theme.note_text = Color::BLACK;
            theme.divider_stroke = Color::BLACK;
            theme.grid_stroke = Color::BLACK;
            theme.detail_stroke = Color::BLACK;
            theme.activation_stroke = Color::BLACK;
            theme.lifeline_stroke = Color::BLACK;
            theme
        }
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
fn prepare_svg(svg: &str, source: &str) -> String {
    let labeled = rewrite_text_elements(svg, flatten_label);
    let expanded = expand_path_markers(&labeled);
    recolor_day_pie_separators(source, &expanded)
}

/// Return whether `source` is a mermaid pie diagram.
fn is_pie_diagram(source: &str) -> bool {
    source
        .trim_start()
        .lines()
        .next()
        .is_some_and(|line| line.trim_start().to_ascii_lowercase().starts_with("pie"))
}

/// Recolor pie slice separators from white (theme background) to black in day mode.
///
/// rusty-mermaid strokes pie slices with `theme.background`. That field is
/// forced to white so the SVG background rect is omitted, which would leave
/// white separators on a light page.
fn recolor_day_pie_separators(source: &str, svg: &str) -> String {
    if current_theme() != ColorTheme::Day || !is_pie_diagram(source) {
        return svg.to_string();
    }
    svg.replace("stroke=\"#ffffff\"", "stroke=\"#000000\"")
}

#[cfg(test)]
mod tests {
    use super::{
        diagram_theme, padded_theme, MermaidRender, Render, RenderPdf, MERMAID_ERROR_CLASS,
    };
    use crate::constants::PDF_FONT_FAMILY;
    use crate::theme::{ColorTheme, ThemeGuard};
    use rusty_mermaid::{render, Color, Theme};

    #[test]
    /// Assert the diagram theme suppresses the background rect.
    fn diagram_theme_suppresses_background() {
        // The SVG renderer omits the background rect when it is white.
        let theme = diagram_theme();
        assert_eq!(theme.background, Color::WHITE);
        assert_eq!(theme.node_stroke, Theme::dark().node_stroke);
        assert_eq!(theme.node_text, Theme::dark().node_text);
        let svg = MermaidRender
            .render_svg("graph LR\n    A[Square Rect] --> B((Circle))\n")
            .unwrap();
        assert!(svg.contains("<svg"), "{svg}");
        assert!(
            svg.to_ascii_lowercase().contains("#7c6fbd"),
            "night node borders must keep the original dark stroke: {svg}"
        );
    }

    #[test]
    /// Assert day diagrams use black strokes and labels on the light palette.
    fn diagram_theme_follows_color_theme() {
        let night_stroke = diagram_theme().node_stroke;
        assert_eq!(night_stroke, Theme::dark().node_stroke);
        let _guard = ThemeGuard::set(ColorTheme::Day);
        let day = diagram_theme();
        assert_eq!(day.background, Color::WHITE);
        assert_eq!(day.node_text, Color::BLACK);
        assert_eq!(day.edge_stroke, Color::BLACK);
        assert_eq!(day.node_stroke, Color::BLACK);
        assert_eq!(day.subgraph_stroke, Color::BLACK);
        assert_eq!(day.composite_stroke, Color::BLACK);
        assert_eq!(day.region_stroke, Color::BLACK);
        let svg = MermaidRender
            .render_svg("graph LR\n    A[Square Rect] --> B((Circle))\n")
            .unwrap();
        let lower = svg.to_ascii_lowercase();
        assert!(
            !lower.contains("stroke=\"#9370db\"") && !lower.contains("stroke=\"#7c6fbd\""),
            "day node borders must not be purple: {svg}"
        );
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
    /// Assert day pie charts use black labels and black slice separators.
    fn day_pie_uses_black_text_and_slice_strokes() {
        let _guard = ThemeGuard::set(ColorTheme::Day);
        let svg = MermaidRender.render(
            "pie title What Voldemort doesn't have?\n\"FRIENDS\" : 2\n\"FAMILY\" : 3\n\"NOSE\" : 4\n",
        );
        assert!(svg.contains("FRIENDS"), "{svg}");
        assert!(svg.contains("fill=\"#000000\""), "{svg}");
        assert!(svg.contains("stroke=\"#000000\""), "{svg}");
        assert!(
            !svg.to_ascii_lowercase().contains("stroke=\"#ffffff\""),
            "pie separators must not stay white: {svg}"
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
    /// Assert a parse-error snippet cannot close the fallback HTML comment.
    fn parse_error_snippet_cannot_break_out_of_html_comment() {
        let source = "--><b>x</b><!--";
        assert!(MermaidRender.render_svg(source).is_err());
        let html = MermaidRender.render(source);
        assert!(
            html.contains(&format!("class=\"{MERMAID_ERROR_CLASS}\"")),
            "{html}"
        );
        assert!(!html.contains("<b>x</b>"), "{html}");
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
