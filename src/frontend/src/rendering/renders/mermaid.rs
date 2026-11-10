//! Renders a Mermaid diagram block to inline SVG.

use super::marker_arrow::expand_path_markers;
use super::mermaid_label::flatten_label;
use super::render_error::render_error_html;
use super::{Render, RenderPdf};
use crate::constants::{MERMAID_ERROR_CLASS, MERMAID_STROKE_SLOP};
use crate::rendering::svg_text_elements::rewrite_text_elements;
use crate::theme::{current_theme, ColorTheme};
use rusty_mermaid::svg::SvgRenderer;
use rusty_mermaid::{render, Color, Point, Primitive, Scene, Style, TextAnchor, Theme};
use rusty_mermaid_core::{PathSegment, SimpleTextMeasure, TextMeasure, Transform};

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
/// rusty-mermaid can place subgraphs, edges, and end-anchored labels at
/// negative coordinates while still reporting the scene origin at (0, 0);
/// without the extra padding the SVG translate clips that ink.
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

/// Return the lowest x/y among scene primitives, clamped to at most zero.
fn content_mins(scene: &Scene) -> (f64, f64) {
    let mut bounds = Bounds::EMPTY;
    for element in scene.elements() {
        include_primitive(&mut bounds, &element.primitive);
    }
    // An empty scene keeps infinite mins, which clamp to the origin here.
    (bounds.min_x.min(0.0), bounds.min_y.min(0.0))
}

/// Axis-aligned bounds accumulated over scene primitives.
struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Bounds {
    /// Bounds that contain nothing; the first `include` sets every edge.
    const EMPTY: Self = Self {
        min_x: f64::INFINITY,
        min_y: f64::INFINITY,
        max_x: f64::NEG_INFINITY,
        max_y: f64::NEG_INFINITY,
    };

    /// Grow to cover `reach_x` / `reach_y` either side of `point`.
    fn include(&mut self, point: Point, reach_x: f64, reach_y: f64) {
        self.min_x = self.min_x.min(point.x - reach_x);
        self.min_y = self.min_y.min(point.y - reach_y);
        self.max_x = self.max_x.max(point.x + reach_x);
        self.max_y = self.max_y.max(point.y + reach_y);
    }

    /// Return whether nothing has been included yet.
    fn is_empty(&self) -> bool {
        self.min_x > self.max_x
    }

    /// Return the four corners of the box.
    fn corners(&self) -> [Point; 4] {
        [
            Point::new(self.min_x, self.min_y),
            Point::new(self.max_x, self.min_y),
            Point::new(self.min_x, self.max_y),
            Point::new(self.max_x, self.max_y),
        ]
    }
}

/// Return half the stroke width, the ink a stroke paints outside its outline.
fn half_stroke(style: &Style) -> f64 {
    style.stroke_width.unwrap_or(0.0) / 2.0
}

/// Grow `bounds` to cover the ink `primitive` paints.
fn include_primitive(bounds: &mut Bounds, primitive: &Primitive) {
    match primitive {
        Primitive::Rect { bbox, style, .. } => {
            let half = half_stroke(style);
            bounds.include(Point::new(bbox.left(), bbox.top()), half, half);
            bounds.include(Point::new(bbox.right(), bbox.bottom()), half, half);
        }
        Primitive::Circle {
            center,
            radius,
            style,
        } => {
            let reach = radius + half_stroke(style);
            bounds.include(*center, reach, reach);
        }
        Primitive::Ellipse {
            center,
            rx,
            ry,
            style,
        } => {
            let half = half_stroke(style);
            bounds.include(*center, rx + half, ry + half);
        }
        Primitive::Polygon { points, style } => {
            let half = half_stroke(style);
            for point in points {
                bounds.include(*point, half, half);
            }
        }
        Primitive::Path {
            segments, style, ..
        } => include_path(bounds, segments, half_stroke(style)),
        Primitive::Text {
            position,
            content,
            anchor,
            style,
        } => {
            // Same measurer the layout uses to size the boxes around labels.
            let size = SimpleTextMeasure::default().measure(content, style);
            let half_width = size.width / 2.0;
            let center_x = match anchor {
                TextAnchor::Start => position.x + half_width,
                TextAnchor::Middle => position.x,
                TextAnchor::End => position.x - half_width,
            };
            bounds.include(
                Point::new(center_x, position.y),
                half_width,
                size.height / 2.0,
            );
        }
        Primitive::Arc {
            center,
            outer_r,
            style,
            ..
        } => {
            // The full outer circle bounds every sector of it.
            let reach = outer_r + half_stroke(style);
            bounds.include(*center, reach, reach);
        }
        Primitive::Group {
            transform,
            children,
        } => {
            let mut inner = Bounds::EMPTY;
            for child in children {
                include_primitive(&mut inner, child);
            }
            if inner.is_empty() {
                return;
            }
            // An affine image of a box is bounded by its transformed corners.
            for corner in inner.corners() {
                bounds.include(apply_transform(transform, corner), 0.0, 0.0);
            }
        }
    }
}

/// Grow `bounds` to cover a path's points, curve control points, and arcs.
///
/// Bézier curves stay inside the hull of their control points, so including
/// the control points is a safe, slightly loose bound.
fn include_path(bounds: &mut Bounds, segments: &[PathSegment], half: f64) {
    let mut current = Point::new(0.0, 0.0);
    let mut subpath_start = current;
    for segment in segments {
        match *segment {
            PathSegment::MoveTo(to) => {
                bounds.include(to, half, half);
                subpath_start = to;
                current = to;
            }
            PathSegment::LineTo(to) => {
                bounds.include(to, half, half);
                current = to;
            }
            PathSegment::CubicTo { cp1, cp2, to } => {
                bounds.include(cp1, half, half);
                bounds.include(cp2, half, half);
                bounds.include(to, half, half);
                current = to;
            }
            PathSegment::QuadTo { cp, to } => {
                bounds.include(cp, half, half);
                bounds.include(to, half, half);
                current = to;
            }
            PathSegment::ArcTo {
                rx,
                ry,
                rotation,
                large_arc,
                sweep,
                to,
            } => {
                if let Some((center, half_w, half_h)) =
                    arc_ellipse_extent(current, to, rx, ry, rotation, large_arc, sweep)
                {
                    bounds.include(center, half_w + half, half_h + half);
                }
                bounds.include(to, half, half);
                current = to;
            }
            PathSegment::Close => current = subpath_start,
        }
    }
}

/// Return the center and half extents of the ellipse an SVG arc lies on.
///
/// Follows the SVG endpoint-to-center conversion (SVG 1.1 §F.6.5), including
/// the radius scale-up when the radii are too small to span the endpoints.
/// Returns `None` when the arc is drawn as a line or omitted (zero radius,
/// coincident endpoints); the endpoints alone bound those.
fn arc_ellipse_extent(
    from: Point,
    to: Point,
    rx: f64,
    ry: f64,
    rotation: f64,
    large_arc: bool,
    sweep: bool,
) -> Option<(Point, f64, f64)> {
    let mut rx = rx.abs();
    let mut ry = ry.abs();
    if rx == 0.0 || ry == 0.0 {
        return None;
    }
    let (sin, cos) = rotation.to_radians().sin_cos();
    let half_dx = (from.x - to.x) / 2.0;
    let half_dy = (from.y - to.y) / 2.0;
    let x1 = cos * half_dx + sin * half_dy;
    let y1 = -sin * half_dx + cos * half_dy;

    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }

    let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    if denominator == 0.0 {
        return None;
    }
    let numerator = rx * rx * ry * ry - denominator;
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let coefficient = sign * (numerator / denominator).max(0.0).sqrt();
    let cx1 = coefficient * rx * y1 / ry;
    let cy1 = -coefficient * ry * x1 / rx;

    let center = Point::new(
        cos * cx1 - sin * cy1 + (from.x + to.x) / 2.0,
        sin * cx1 + cos * cy1 + (from.y + to.y) / 2.0,
    );
    let half_w = ((rx * cos).powi(2) + (ry * sin).powi(2)).sqrt();
    let half_h = ((rx * sin).powi(2) + (ry * cos).powi(2)).sqrt();
    Some((center, half_w, half_h))
}

/// Map `point` through an SVG group transform.
fn apply_transform(transform: &Transform, point: Point) -> Point {
    match *transform {
        Transform::Identity => point,
        Transform::Translate(x, y) => Point::new(point.x + x, point.y + y),
        Transform::Scale(sx, sy) => Point::new(point.x * sx, point.y * sy),
        Transform::Rotate { degrees, cx, cy } => {
            let (sin, cos) = degrees.to_radians().sin_cos();
            let dx = point.x - cx;
            let dy = point.y - cy;
            Point::new(cx + dx * cos - dy * sin, cy + dx * sin + dy * cos)
        }
    }
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
        content_mins, diagram_theme, padded_theme, MermaidRender, Render, RenderPdf,
        MERMAID_ERROR_CLASS,
    };
    use crate::constants::{MERMAID_STROKE_SLOP, PDF_FONT_FAMILY};
    use crate::theme::{ColorTheme, ThemeGuard};
    use rusty_mermaid::{
        render, BBox, Color, Point, Primitive, Scene, Style, TextAnchor, TextStyle, Theme,
    };
    use rusty_mermaid_core::{PathSegment, SimpleTextMeasure, TextMeasure, Transform};

    /// Return a scene holding one stroke-free rect spanning (10,10)-(50,30).
    fn scene_with_inner_rect() -> Scene {
        let mut scene = Scene::new(100.0, 100.0);
        // `BBox` is center-based.
        scene.push(Primitive::Rect {
            bbox: BBox::new(30.0, 20.0, 40.0, 20.0),
            rx: 0.0,
            ry: 0.0,
            style: Style::default(),
        });
        scene
    }

    #[test]
    /// Assert an edge path left of the origin grows the padding.
    fn edge_path_overhang_is_included_in_padding() {
        let mut scene = scene_with_inner_rect();
        scene.push(Primitive::Path {
            segments: vec![
                PathSegment::MoveTo(Point::new(-12.0, 15.0)),
                PathSegment::LineTo(Point::new(10.0, 15.0)),
            ],
            style: Style {
                stroke_width: Some(2.0),
                ..Style::default()
            },
            marker_start: None,
            marker_end: None,
        });
        assert_eq!(content_mins(&scene), (-13.0, 0.0));
        let theme = diagram_theme();
        let padded = padded_theme(theme.clone(), &scene).padding;
        assert_eq!(padded, theme.padding.max(13.0 + MERMAID_STROKE_SLOP));
        assert!(padded >= 13.0, "padding {padded} must cover the edge");
    }

    #[test]
    /// Assert a cubic edge is bounded by its control points, not its ends.
    fn cubic_control_points_bound_the_curve() {
        let mut scene = scene_with_inner_rect();
        scene.push(Primitive::Path {
            segments: vec![
                PathSegment::MoveTo(Point::new(10.0, 5.0)),
                PathSegment::CubicTo {
                    cp1: Point::new(10.0, -8.0),
                    cp2: Point::new(30.0, -8.0),
                    to: Point::new(30.0, 5.0),
                },
            ],
            style: Style::default(),
            marker_start: None,
            marker_end: None,
        });
        assert_eq!(content_mins(&scene), (0.0, -8.0));
    }

    #[test]
    /// Assert an arc that bulges past the origin is bounded by its ellipse.
    fn arc_bulge_is_included() {
        // Semicircle from (5, 10) to (5, 30), radius 10, center (5, 20):
        // one sweep bulges to x = -5, the other to x = 15.
        for sweep in [false, true] {
            let mut scene = scene_with_inner_rect();
            scene.push(Primitive::Path {
                segments: vec![
                    PathSegment::MoveTo(Point::new(5.0, 10.0)),
                    PathSegment::ArcTo {
                        rx: 10.0,
                        ry: 10.0,
                        rotation: 0.0,
                        large_arc: false,
                        sweep,
                        to: Point::new(5.0, 30.0),
                    },
                ],
                style: Style::default(),
                marker_start: None,
                marker_end: None,
            });
            let (min_x, min_y) = content_mins(&scene);
            assert!((min_x + 5.0).abs() < 1e-9, "sweep={sweep} min_x={min_x}");
            assert_eq!(min_y, 0.0);
        }
    }

    #[test]
    /// Assert end-anchored label ink left of its anchor is included.
    fn end_anchored_text_width_is_included() {
        let mut scene = scene_with_inner_rect();
        let style = TextStyle::default();
        let content = "Long label";
        let width = SimpleTextMeasure::default().measure(content, &style).width;
        scene.push(Primitive::Text {
            position: Point::new(5.0, 20.0),
            content: content.to_string(),
            anchor: TextAnchor::End,
            style,
        });
        let (min_x, _) = content_mins(&scene);
        assert!(width > 5.0, "label must be wider than its anchor x");
        assert!((min_x - (5.0 - width)).abs() < 1e-9, "min_x={min_x}");
    }

    #[test]
    /// Assert a rotated group is bounded by its transformed children.
    fn rotated_group_is_included() {
        let mut scene = scene_with_inner_rect();
        // Rotating (0,0)-(20,10) by 90° about the origin maps it to (-10,0)-(0,20).
        scene.push(Primitive::Group {
            transform: Transform::Rotate {
                degrees: 90.0,
                cx: 0.0,
                cy: 0.0,
            },
            children: vec![Primitive::Rect {
                bbox: BBox::new(10.0, 5.0, 20.0, 10.0),
                rx: 0.0,
                ry: 0.0,
                style: Style::default(),
            }],
        });
        let (min_x, min_y) = content_mins(&scene);
        assert!((min_x + 10.0).abs() < 1e-9, "min_x={min_x}");
        assert!(min_y.abs() < 1e-9, "min_y={min_y}");
    }

    #[test]
    /// Assert an empty scene does not overhang.
    fn empty_scene_has_no_overhang() {
        assert_eq!(content_mins(&Scene::new(10.0, 10.0)), (0.0, 0.0));
    }

    #[test]
    /// Assert a pie chart keeps the theme padding unchanged.
    fn pie_padding_is_unchanged() {
        let theme = diagram_theme();
        let scene = render("pie title Pets\n\"Dogs\" : 2\n\"Cats\" : 3\n", &theme).unwrap();
        assert_eq!(content_mins(&scene), (0.0, 0.0));
    }

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
