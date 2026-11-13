//! Renders a Mermaid diagram block to inline SVG.

use super::marker_arrow::expand_path_markers;
use super::mermaid_directive::apply_directives;
use super::mermaid_label::flatten_label;
use super::render_error::render_error_html;
use super::{Render, RenderPdf};
use crate::constants::{MERMAID_ERROR_CLASS, MERMAID_STROKE_SLOP};
use crate::rendering::svg_text_elements::rewrite_text_elements;
use crate::theme::{current_theme, ColorTheme};
use rusty_mermaid::svg::SvgRenderer;
use rusty_mermaid::{
    render, Color, ParseError, Point, Primitive, Scene, Style, TextAnchor, TextStyle, Theme,
};
use rusty_mermaid_core::{PathSegment, SimpleTextMeasure, TextMeasure, Transform};

/// Mermaid diagram render.
pub struct MermaidRender;

impl MermaidRender {
    /// Render Mermaid source to an inline SVG string.
    ///
    /// Returns the rusty-mermaid parse error when `source` is not a diagram.
    pub fn render_svg(&self, source: &str) -> Result<String, String> {
        let mut theme = diagram_theme();
        let directives = apply_directives(source, &mut theme);
        let source = &directives.body;
        let scene = match directives.node_label_color {
            None => render(source, &theme),
            Some(color) => render_with_node_labels(source, &theme, color),
        }
        .map_err(|e| e.to_string())?;
        let svg_theme = padded_theme(theme, &scene);
        let svg = SvgRenderer::with_theme(&svg_theme).render_themed(&scene, &svg_theme);
        Ok(prepare_svg(&svg, source))
    }
}

/// Stand-in composite-state label colour that marks those labels in a scene.
///
/// Fully transparent near-black, which no theme or directive produces.
const COMPOSITE_LABEL_MARKER: Color = Color::rgba(1, 2, 3, 0);

/// Render `source`, painting flowchart and state node labels `color`.
///
/// rusty-mermaid ignores `theme.node_text` for those labels on very dark or
/// light fills and uses white or black instead, so an explicit text colour
/// would be lost. Node labels are the only text drawn in the default label
/// style apart from state composite titles, which are rendered with a marker
/// colour here so they can be told apart and restored.
fn render_with_node_labels(source: &str, theme: &Theme, color: Color) -> Result<Scene, ParseError> {
    let marked = Theme {
        composite_label: COMPOSITE_LABEL_MARKER,
        ..theme.clone()
    };
    let scene = render(source, &marked)?;
    let label = TextStyle::default();
    let mut painted = Scene::new(scene.width, scene.height);
    for element in scene.elements() {
        let mut primitive = element.primitive.clone();
        if let Primitive::Text {
            anchor: TextAnchor::Middle,
            style,
            ..
        } = &mut primitive
        {
            if style.fill == Some(COMPOSITE_LABEL_MARKER) {
                style.fill = Some(theme.composite_label);
            } else if style.font_size == label.font_size && style.font_weight == label.font_weight {
                style.fill = Some(color);
            }
        }
        match &element.id {
            Some(id) => painted.push_identified(primitive, id.clone()),
            None => painted.push(primitive),
        }
    }
    Ok(painted)
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
    /// Assert a multi-line `init` directive renders instead of failing to parse.
    fn multiline_init_directive_renders() {
        let source = "%%{init: {\n  'theme': 'base',\n  'themeVariables': {\n    \
                      'primaryColor': '#4f46e5'\n  }\n}}%%\nflowchart LR\n    \
                      Primary[Primary Color Node] --> Secondary[Secondary Color Node]\n";
        let svg = MermaidRender.render_svg(source).unwrap();
        assert!(svg.contains("Primary Color Node"), "{svg}");
    }

    #[test]
    /// Assert a multi-line directive before `pie` still applies day separators.
    fn multiline_directive_keeps_pie_detection() {
        let _guard = ThemeGuard::set(ColorTheme::Day);
        let svg = MermaidRender
            .render_svg("%%{init: {\n'theme': 'base'\n}}%%\npie\n\"A\" : 1\n\"B\" : 2\n")
            .unwrap();
        assert!(!svg.contains("stroke=\"#ffffff\""), "{svg}");
    }

    /// Flowchart with a subgraph and a labelled edge.
    const FLOWCHART: &str =
        "flowchart LR\n    subgraph G\n        A[One] -->|label| B[Two]\n    end\n";
    /// Sequence diagram with an activation, a loop frame and a note.
    const SEQUENCE: &str = "sequenceDiagram\n    participant A\n    participant B\n    \
                            A->>B: hello\n    activate B\n    loop Every minute\n        \
                            B-->>A: reply\n    end\n    deactivate B\n    \
                            Note right of A: a note\n";
    /// State diagram with start/end states, a composite state and a note.
    const STATE: &str = "stateDiagram-v2\n    [*] --> Still\n    Still --> Moving: push\n    \
                         Moving --> [*]\n    state Comp {\n        X --> Y\n    }\n    \
                         note right of Still: a note\n";
    /// Class diagram with one class.
    const CLASS: &str = "classDiagram\n    class Animal {\n        +name\n    }\n";
    /// ER diagram with two attribute rows.
    const ER: &str =
        "erDiagram\n    CUSTOMER {\n        string name\n        string id\n    }\n    \
                      CUSTOMER ||--o{ ORDER : places\n";
    /// Requirement diagram with one relation.
    const REQUIREMENT: &str = "requirementDiagram\n    requirement r {\n        id: 1\n        \
                               text: t\n    }\n    element e {\n        type: sim\n    }\n    \
                               e - satisfies -> r\n";
    /// Gantt chart with one task.
    const GANTT: &str = "gantt\n    title Plan\n    dateFormat YYYY-MM-DD\n    section S\n    \
                         Task :a1, 2024-01-01, 3d\n";
    /// Git graph with a tagged commit.
    const GIT: &str = "gitGraph\n    commit id: \"one\" tag: \"v1\"\n    commit\n";
    /// User journey with one task.
    const JOURNEY: &str = "journey\n    title Day\n    section Work\n      Code: 5: Me\n";
    /// Pie chart with a title and legend.
    const PIE: &str = "pie title Pets\n    \"Dogs\" : 2\n    \"Cats\" : 3\n";
    /// Quadrant chart with one point.
    const QUADRANT: &str = "quadrantChart\n    title Reach\n    x-axis Low --> High\n    \
                            y-axis Low --> High\n    A: [0.3, 0.6]\n";
    /// XY chart with one bar series.
    const XYCHART: &str = "xychart-beta\n    title \"Sales\"\n    x-axis [jan, feb]\n    \
                           y-axis \"Revenue\" 0 --> 100\n    bar [10, 20]\n";
    /// Radar chart with one curve.
    const RADAR: &str = "radar-beta\n    axis a, b, c\n    curve c1{1, 2, 3}\n";

    /// Each theme variable, a diagram that draws it, and the colour it sets.
    const VARIABLE_CASES: &[(&str, &str, &str)] = &[
        ("'primaryColor': '#a10001'", FLOWCHART, "#a10001"),
        ("'primaryTextColor': '#a10002'", FLOWCHART, "#a10002"),
        ("'primaryBorderColor': '#a10003'", FLOWCHART, "#a10003"),
        ("'secondaryColor': '#a10004'", FLOWCHART, "#a10004"),
        ("'tertiaryColor': '#a10005'", FLOWCHART, "#a10005"),
        ("'tertiaryBorderColor': '#a10006'", FLOWCHART, "#a10006"),
        ("'tertiaryTextColor': '#a10007'", FLOWCHART, "#a10007"),
        ("'lineColor': '#a10008'", FLOWCHART, "#a10008"),
        ("'textColor': '#a10009'", FLOWCHART, "#a10009"),
        ("'background': '#a1000a'", FLOWCHART, "#a1000a"),
        ("'mainBkg': '#a1000b'", FLOWCHART, "#a1000b"),
        ("'nodeBkg': '#a1000c'", FLOWCHART, "#a1000c"),
        ("'nodeBorder': '#a1000d'", FLOWCHART, "#a1000d"),
        ("'nodeTextColor': '#a1000e'", FLOWCHART, "#a1000e"),
        ("'clusterBkg': '#a1000f'", FLOWCHART, "#a1000f"),
        ("'clusterBorder': '#a10010'", FLOWCHART, "#a10010"),
        ("'titleColor': '#a10011'", FLOWCHART, "#a10011"),
        ("'defaultLinkColor': '#a10012'", FLOWCHART, "#a10012"),
        ("'edgeLabelBackground': '#a10013'", FLOWCHART, "#a10013"),
        ("'actorBkg': '#a20001'", SEQUENCE, "#a20001"),
        ("'actorBorder': '#a20002'", SEQUENCE, "#a20002"),
        ("'actorTextColor': '#a20003'", SEQUENCE, "#a20003"),
        ("'actorLineColor': '#a20004'", SEQUENCE, "#a20004"),
        ("'signalColor': '#a20005'", SEQUENCE, "#a20005"),
        ("'signalTextColor': '#a20006'", SEQUENCE, "#a20006"),
        ("'labelBoxBkgColor': '#a20007'", SEQUENCE, "#a20007"),
        ("'labelBoxBorderColor': '#a20008'", SEQUENCE, "#a20008"),
        ("'labelTextColor': '#a20009'", SEQUENCE, "#a20009"),
        ("'loopTextColor': '#a2000a'", SEQUENCE, "#a2000a"),
        ("'activationBkgColor': '#a2000b'", SEQUENCE, "#a2000b"),
        ("'activationBorderColor': '#a2000c'", SEQUENCE, "#a2000c"),
        ("'noteBkgColor': '#a2000d'", SEQUENCE, "#a2000d"),
        ("'noteBorderColor': '#a2000e'", SEQUENCE, "#a2000e"),
        ("'noteTextColor': '#a2000f'", SEQUENCE, "#a2000f"),
        ("'secondaryColor': '#a20010'", SEQUENCE, "#a20010"),
        ("'stateBkg': '#a30001'", STATE, "#a30001"),
        ("'stateBorder': '#a30002'", STATE, "#a30002"),
        ("'stateLabelColor': '#a30003'", STATE, "#a30003"),
        ("'transitionColor': '#a30004'", STATE, "#a30004"),
        ("'transitionLabelColor': '#a30005'", STATE, "#a30005"),
        ("'specialStateColor': '#a30006'", STATE, "#a30006"),
        ("'innerEndBackground': '#a30007'", STATE, "#a30007"),
        ("'compositeBackground': '#a30008'", STATE, "#a30008"),
        ("'compositeBorder': '#a30009'", STATE, "#a30009"),
        ("'noteBkgColor': '#a3000a'", STATE, "#a3000a"),
        ("'noteBorderColor': '#a3000b'", STATE, "#a3000b"),
        ("'noteTextColor': '#a3000c'", STATE, "#a3000c"),
        ("'classText': '#a40001'", CLASS, "#a40001"),
        ("'primaryColor': '#a40002'", CLASS, "#a40002"),
        ("'attributeBackgroundColorEven': '#a50001'", ER, "#a50001"),
        ("'attributeBackgroundColorOdd': '#a50002'", ER, "#a50002"),
        ("'requirementBackground': '#a60001'", REQUIREMENT, "#a60001"),
        (
            "'requirementBorderColor': '#a60002'",
            REQUIREMENT,
            "#a60002",
        ),
        ("'requirementTextColor': '#a60003'", REQUIREMENT, "#a60003"),
        ("'relationColor': '#a60004'", REQUIREMENT, "#a60004"),
        (
            "'relationLabelBackground': '#a60005'",
            REQUIREMENT,
            "#a60005",
        ),
        ("'relationLabelColor': '#a60006'", REQUIREMENT, "#a60006"),
        ("'titleColor': '#a70001'", GANTT, "#a70001"),
        ("'taskTextColor': '#a70002'", GANTT, "#a70002"),
        ("'taskTextOutsideColor': '#a70003'", GANTT, "#a70003"),
        ("'gridColor': '#a70004'", GANTT, "#a70004"),
        ("'commitLabelColor': '#a80001'", GIT, "#a80001"),
        ("'tagLabelBackground': '#a80002'", GIT, "#a80002"),
        ("'tagLabelColor': '#a80003'", GIT, "#a80003"),
        ("'faceColor': '#a90001'", JOURNEY, "#a90001"),
        ("'pieTitleTextColor': '#aa0001'", PIE, "#aa0001"),
        ("'pieLegendTextColor': '#aa0002'", PIE, "#aa0002"),
        ("'quadrantTitleFill': '#ab0001'", QUADRANT, "#ab0001"),
        ("'quadrantXAxisTextFill': '#ab0002'", QUADRANT, "#ab0002"),
        ("'quadrantYAxisTextFill': '#ab0003'", QUADRANT, "#ab0003"),
        ("'quadrantPointTextFill': '#ab0004'", QUADRANT, "#ab0004"),
        (
            "'quadrantInternalBorderStrokeFill': '#ab0005'",
            QUADRANT,
            "#ab0005",
        ),
        (
            "'quadrantExternalBorderStrokeFill': '#ab0006'",
            QUADRANT,
            "#ab0006",
        ),
        (
            "'xyChart': {'backgroundColor': '#ac0001'}",
            XYCHART,
            "#ac0001",
        ),
        ("'xyChart': {'titleColor': '#ac0002'}", XYCHART, "#ac0002"),
        (
            "'xyChart': {'xAxisLabelColor': '#ac0003'}",
            XYCHART,
            "#ac0003",
        ),
        (
            "'xyChart': {'xAxisTitleColor': '#ac0004'}",
            XYCHART,
            "#ac0004",
        ),
        (
            "'xyChart': {'yAxisLabelColor': '#ac0005'}",
            XYCHART,
            "#ac0005",
        ),
        (
            "'xyChart': {'yAxisTitleColor': '#ac0006'}",
            XYCHART,
            "#ac0006",
        ),
        (
            "'xyChart': {'xAxisLineColor': '#ac0007'}",
            XYCHART,
            "#ac0007",
        ),
        (
            "'xyChart': {'xAxisTickColor': '#ac0008'}",
            XYCHART,
            "#ac0008",
        ),
        (
            "'xyChart': {'yAxisLineColor': '#ac0009'}",
            XYCHART,
            "#ac0009",
        ),
        (
            "'xyChart': {'yAxisTickColor': '#ac000a'}",
            XYCHART,
            "#ac000a",
        ),
        ("'radar': {'axisColor': '#ad0001'}", RADAR, "#ad0001"),
        ("'radar': {'graticuleColor': '#ad0002'}", RADAR, "#ad0002"),
    ];

    /// Return the SVG for `diagram` with `variables` in a multi-line `init` directive.
    fn render_with_variables(variables: &str, diagram: &str) -> String {
        let source = format!(
            "%%{{init: {{\n  'themeVariables': {{\n    {variables}\n  }}\n}}}}%%\n{diagram}"
        );
        MermaidRender
            .render_svg(&source)
            .unwrap_or_else(|error| panic!("{variables} on {diagram}: {error}"))
            .to_ascii_lowercase()
    }

    /// Return each `VARIABLE_CASES` entry whose colour is missing from its SVG.
    fn undrawn_variables() -> Vec<String> {
        VARIABLE_CASES
            .iter()
            .filter(|(variables, diagram, colour)| {
                !render_with_variables(variables, diagram).contains(colour)
            })
            .map(|(variables, diagram, _)| {
                format!(
                    "{variables} on {}",
                    diagram.lines().next().unwrap_or_default()
                )
            })
            .collect()
    }

    #[test]
    /// Assert every supported theme variable reaches the SVG in night mode.
    fn every_theme_variable_reaches_the_night_svg() {
        let missing = undrawn_variables();
        assert!(missing.is_empty(), "not drawn:\n{}", missing.join("\n"));
    }

    #[test]
    /// Assert every supported theme variable reaches the SVG in day mode.
    fn every_theme_variable_reaches_the_day_svg() {
        let _guard = ThemeGuard::set(ColorTheme::Day);
        let missing = undrawn_variables();
        assert!(missing.is_empty(), "not drawn:\n{}", missing.join("\n"));
    }

    #[test]
    /// Assert explicit state label colour leaves composite titles their own.
    fn state_label_colour_keeps_composite_title_colour() {
        let svg = render_with_variables(
            "'stateBkg': '#101010', 'primaryTextColor': '#b10001'",
            "stateDiagram-v2\n    [*] --> Still\n    state Comp {\n        X --> Y\n    }\n",
        );
        let composite = Theme::dark().composite_label.to_string();
        assert!(svg.contains("#b10001"), "{svg}");
        assert!(
            svg.contains(&composite),
            "composite title {composite}: {svg}"
        );
        assert!(!svg.contains("#01020300"), "marker colour leaked: {svg}");
    }

    #[test]
    /// Assert the multi-line flowchart example takes its colours.
    fn multiline_init_colours_the_flowchart() {
        let svg = render_with_variables(
            "'primaryColor': '#4f46e5',\n    'primaryBorderColor': '#312e81',\n    \
             'lineColor': '#f59e0b',\n    'secondaryColor': '#ec4899',\n    \
             'tertiaryColor': '#f3f4f6'",
            FLOWCHART,
        );
        assert!(svg.contains("fill=\"#4f46e5\""), "{svg}");
        assert!(svg.contains("stroke=\"#312e81\""), "{svg}");
        assert!(svg.contains("#f59e0b"), "{svg}");
        assert!(svg.contains("fill=\"#ec4899\""), "{svg}");
        assert!(svg.contains("fill=\"#f3f4f6\""), "{svg}");
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
