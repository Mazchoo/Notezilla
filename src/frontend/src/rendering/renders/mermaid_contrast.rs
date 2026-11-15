//! Repaints diagram labels that are unreadable on the fill beneath them.
//!
//! rusty-mermaid draws many labels in a theme text colour on hardcoded fills:
//! pale section tints (mindmap, journey, block, architecture), quadrant
//! tints, packet blocks and kanban cards filled with the theme background.
//! With the night palette those labels come out pale on pale. Each label is
//! tested against the composited fills under its centre, and only labels
//! below [`MERMAID_MIN_TEXT_CONTRAST`] are switched to black or white.

use super::mermaid::{apply_transform, arc_ellipse_extent};
use crate::constants::{
    MERMAID_ARC_SAMPLES as ARC_SAMPLES, MERMAID_MIN_TEXT_CONTRAST,
    WCAG_CONTRAST_FLARE as CONTRAST_FLARE,
};
use rusty_mermaid::{BBox, Color, Point, Primitive, Scene, TextAnchor};
use rusty_mermaid_core::{PathSegment, SimpleTextMeasure, TextMeasure, Transform};
use std::f64::consts::TAU;

/// A filled area painted before later labels.
enum Fill {
    Rect(BBox, Color),
    Ellipse(Point, f64, f64, Color),
    Polygon(Vec<Point>, Color),
}

impl Fill {
    /// Return the filled area `primitive` paints, if it paints one.
    ///
    /// Paths become the polygon through their segment end points, which
    /// is close enough for the node shapes they draw. Shapes inside groups
    /// and pie arcs are skipped; labels over them keep their colour.
    fn of(primitive: &Primitive) -> Option<Self> {
        match primitive {
            Primitive::Rect { bbox, style, .. } => Some(Self::Rect(*bbox, visible(style.fill)?)),
            Primitive::Circle {
                center,
                radius,
                style,
            } => Some(Self::Ellipse(*center, *radius, *radius, visible(style.fill)?)),
            Primitive::Ellipse {
                center,
                rx,
                ry,
                style,
            } => Some(Self::Ellipse(*center, *rx, *ry, visible(style.fill)?)),
            Primitive::Polygon { points, style } => {
                Some(Self::Polygon(points.clone(), visible(style.fill)?))
            }
            Primitive::Path {
                segments, style, ..
            } => {
                let color = visible(style.fill)?;
                let points = path_outline(segments);
                (points.len() >= 3).then_some(Self::Polygon(points, color))
            }
            _ => None,
        }
    }

    /// Return the fill colour when `point` lies inside the area.
    fn color_at(&self, point: Point) -> Option<Color> {
        let (inside, color) = match self {
            Self::Rect(bbox, color) => (bbox.contains(point), color),
            Self::Ellipse(center, rx, ry, color) => {
                let dx = (point.x - center.x) / rx;
                let dy = (point.y - center.y) / ry;
                (dx * dx + dy * dy <= 1.0, color)
            }
            Self::Polygon(points, color) => (polygon_contains(points, point), color),
        };
        inside.then_some(*color)
    }
}

/// Return `fill` when it is set and not fully transparent.
fn visible(fill: Option<Color>) -> Option<Color> {
    fill.filter(|color| color.a > 0)
}

/// Return a polygon approximating the area a path encloses.
///
/// Lines and curves contribute their end points. Circular arcs, such as
/// pie slice rims, are sampled along the circle so labels near the rim
/// still land inside the slice.
fn path_outline(segments: &[PathSegment]) -> Vec<Point> {
    let mut points = Vec::with_capacity(segments.len() + ARC_SAMPLES);
    let mut current = Point::new(0.0, 0.0);
    for segment in segments {
        match *segment {
            PathSegment::MoveTo(to)
            | PathSegment::LineTo(to)
            | PathSegment::CubicTo { to, .. }
            | PathSegment::QuadTo { to, .. } => {
                points.push(to);
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
                let circle = arc_ellipse_extent(current, to, rx, ry, rotation, large_arc, sweep)
                    .filter(|(_, half_w, half_h)| (half_w - half_h).abs() < f64::EPSILON.sqrt());
                if let Some((center, radius, _)) = circle {
                    push_arc_samples(&mut points, center, radius, current, to, sweep);
                }
                points.push(to);
                current = to;
            }
            PathSegment::Close => {}
        }
    }
    points
}

/// Push points along the circle at `center` strictly between `from` and `to`.
///
/// `sweep` follows SVG: true runs toward increasing angles (clockwise on a
/// y-down canvas).
fn push_arc_samples(
    points: &mut Vec<Point>,
    center: Point,
    radius: f64,
    from: Point,
    to: Point,
    sweep: bool,
) {
    let start = (from.y - center.y).atan2(from.x - center.x);
    let mut end = (to.y - center.y).atan2(to.x - center.x);
    if sweep && end <= start {
        end += TAU;
    } else if !sweep && end >= start {
        end -= TAU;
    }
    let step = (end - start) / ARC_SAMPLES as f64;
    for i in 1..ARC_SAMPLES {
        let angle = start + step * i as f64;
        points.push(Point::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        ));
    }
}

/// Return whether `point` is inside `polygon` (even-odd rule).
fn polygon_contains(polygon: &[Point], point: Point) -> bool {
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if (current.y > point.y) != (previous.y > point.y) {
            let crossing = previous.x
                + (point.y - previous.y) * (current.x - previous.x) / (current.y - previous.y);
            if point.x < crossing {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}

/// Repaint labels in `scene` that lack contrast with what lies beneath them.
///
/// `page` is the surface the SVG is shown on, under every fill. Labels in a
/// `chosen` colour (one a directive set) are left as they are.
pub(super) fn fix_label_contrast(scene: &Scene, page: Color, chosen: &[Color]) -> Scene {
    let mut canvas = Canvas {
        page,
        chosen,
        fills: Vec::new(),
    };
    let mut fixed = Scene::new(scene.width, scene.height);
    for element in scene.elements() {
        let mut primitive = element.primitive.clone();
        canvas.repaint(&mut primitive, &Transform::Identity);
        if let Some(fill) = Fill::of(&primitive) {
            canvas.fills.push(fill);
        }
        match &element.id {
            Some(id) => fixed.push_identified(primitive, id.clone()),
            None => fixed.push(primitive),
        }
    }
    fixed
}

/// What has been painted so far, and the label colours to leave alone.
struct Canvas<'a> {
    /// Surface under every fill.
    page: Color,
    /// Label colours a directive set.
    chosen: &'a [Color],
    /// Fills painted so far, in paint order.
    fills: Vec<Fill>,
}

impl Canvas<'_> {
    /// Repaint `primitive`, or the labels in a group, if they lack contrast.
    fn repaint(&self, primitive: &mut Primitive, transform: &Transform) {
        match primitive {
            Primitive::Text {
                position,
                content,
                anchor,
                style,
            } => {
                let Some(color) = style.fill.filter(|color| !self.chosen.contains(color)) else {
                    return;
                };
                let width = SimpleTextMeasure::default().measure(content, style).width;
                let center_x = match anchor {
                    TextAnchor::Start => position.x + width / 2.0,
                    TextAnchor::Middle => position.x,
                    TextAnchor::End => position.x - width / 2.0,
                };
                let center = apply_transform(transform, Point::new(center_x, position.y));
                let backdrop = self.backdrop_at(center);
                if contrast(blend(color, backdrop), backdrop) < MERMAID_MIN_TEXT_CONTRAST {
                    style.fill = Some(readable_on(backdrop));
                }
            }
            Primitive::Group {
                transform,
                children,
            } => {
                for child in children {
                    self.repaint(child, transform);
                }
            }
            _ => {}
        }
    }

    /// Return the colour seen at `point` after compositing every fill.
    fn backdrop_at(&self, point: Point) -> Color {
        self.fills
            .iter()
            .filter_map(|fill| fill.color_at(point))
            .fold(self.page, |below, above| blend(above, below))
    }
}

/// Composite `top` over opaque `bottom` using `top`'s alpha.
fn blend(top: Color, bottom: Color) -> Color {
    let alpha = f64::from(top.a) / 255.0;
    let mix = |over: u8, under: u8| {
        (f64::from(over) * alpha + f64::from(under) * (1.0 - alpha)).round() as u8
    };
    Color::rgb(mix(top.r, bottom.r), mix(top.g, bottom.g), mix(top.b, bottom.b))
}

/// Return the WCAG contrast ratio between two opaque colours.
fn contrast(a: Color, b: Color) -> f64 {
    let (la, lb) = (a.luminance(), b.luminance());
    (la.max(lb) + CONTRAST_FLARE) / (la.min(lb) + CONTRAST_FLARE)
}

/// Return black or white, whichever reads better on `backdrop`.
fn readable_on(backdrop: Color) -> Color {
    if contrast(Color::BLACK, backdrop) >= contrast(Color::WHITE, backdrop) {
        Color::BLACK
    } else {
        Color::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::{blend, contrast, fix_label_contrast, readable_on};
    use crate::constants::MERMAID_MIN_TEXT_CONTRAST;
    use rusty_mermaid::{BBox, Color, Point, Primitive, Scene, Style, TextAnchor, TextStyle};

    /// Night page colour, #1e1e2e.
    const PAGE: Color = Color::rgb(30, 30, 46);
    /// Night theme label colour, #cdd6f4.
    const PALE: Color = Color::rgb(205, 214, 244);

    /// Return a label at `(x, 20)` painted `fill`.
    fn label(x: f64, fill: Color) -> Primitive {
        Primitive::Text {
            position: Point::new(x, 20.0),
            content: "Label".to_string(),
            anchor: TextAnchor::Middle,
            style: TextStyle {
                fill: Some(fill),
                ..TextStyle::default()
            },
        }
    }

    /// Return the label fills in `scene`, in order.
    fn label_fills(scene: &Scene) -> Vec<Option<Color>> {
        scene
            .elements()
            .iter()
            .filter_map(|element| match &element.primitive {
                Primitive::Text { style, .. } => Some(style.fill),
                _ => None,
            })
            .collect()
    }

    #[test]
    /// Assert a pale label on a pale box turns black, one on the page stays.
    fn pale_label_on_pale_fill_turns_black() {
        let mut scene = Scene::new(400.0, 40.0);
        scene.push(Primitive::Rect {
            bbox: BBox::new(50.0, 20.0, 100.0, 40.0),
            rx: 0.0,
            ry: 0.0,
            style: Style {
                fill: Some(Color::rgb(235, 240, 248)),
                ..Style::default()
            },
        });
        scene.push(label(50.0, PALE));
        scene.push(label(300.0, PALE));
        let fixed = fix_label_contrast(&scene, PAGE, &[]);
        assert_eq!(label_fills(&fixed), [Some(Color::BLACK), Some(PALE)]);
        // A colour a directive chose is kept even where it reads poorly.
        let kept = fix_label_contrast(&scene, PAGE, &[PALE]);
        assert_eq!(label_fills(&kept), [Some(PALE), Some(PALE)]);
    }

    #[test]
    /// Assert a faint translucent fill is composited over the dark page.
    fn translucent_fill_keeps_pale_label() {
        let mut scene = Scene::new(100.0, 40.0);
        scene.push(Primitive::Rect {
            bbox: BBox::new(50.0, 20.0, 100.0, 40.0),
            rx: 0.0,
            ry: 0.0,
            style: Style {
                fill: Some(Color::rgba(78, 121, 167, 20)),
                ..Style::default()
            },
        });
        scene.push(label(50.0, PALE));
        assert_eq!(label_fills(&fix_label_contrast(&scene, PAGE, &[])), [Some(PALE)]);
    }

    #[test]
    /// Assert the replacement colour always meets the minimum contrast.
    fn readable_colour_meets_minimum() {
        for backdrop in [PAGE, PALE, Color::WHITE, Color::rgb(124, 111, 189)] {
            let ink = readable_on(backdrop);
            assert!(contrast(ink, backdrop) >= MERMAID_MIN_TEXT_CONTRAST, "{backdrop:?}");
        }
        assert_eq!(blend(Color::rgba(255, 255, 255, 0), PAGE), PAGE);
    }
}
