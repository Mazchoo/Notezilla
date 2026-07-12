use super::escape_html;
use rusty_mermaid::svg::SvgRenderer;
use rusty_mermaid::{render, Color, Primitive, Scene, Theme};

/// Render a Mermaid diagram source string to an inline SVG string.
///
/// Uses the pure-Rust `rusty-mermaid` pipeline (parse → layout → SVG) so the
/// same code runs in WASM and on native, with no JavaScript runtime, no CDN
/// script, and no DOM post-processing step.
pub fn render_mermaid(src: &str) -> String {
    // The dark theme's text (#cdd6f4) matches the app's --text so labels read
    // correctly both on shape fills and on the diagram background. Setting
    // `background` to white suppresses the background <rect> in the SVG (the
    // renderer skips it for white), letting the editor surface show through.
    let theme = Theme {
        background: Color::WHITE,
        ..Theme::dark()
    };
    match render(src, &theme) {
        Ok(scene) => {
            // rusty-mermaid can place subgraphs at negative coordinates while
            // still reporting the scene origin at (0, 0). Grow SVG padding so
            // the translate keeps strokes inside the viewBox (avoids left/top
            // clipping on subgraph-only diagrams).
            let (min_x, min_y) = content_mins(&scene);
            let stroke_slop = 1.0;
            let mut svg_theme = theme;
            if min_x < 0.0 {
                svg_theme.padding = svg_theme.padding.max(-min_x + stroke_slop);
            }
            if min_y < 0.0 {
                svg_theme.padding = svg_theme.padding.max(-min_y + stroke_slop);
            }
            SvgRenderer::with_theme(&svg_theme).render_themed(&scene, &svg_theme)
        }
        Err(e) => {
            let escaped = escape_html(src);
            format!(
                "<pre class=\"mermaid-error\"><code>{escaped}</code></pre>\
                 <!-- mermaid error: {e} -->"
            )
        }
    }
}

/// Lowest x/y among shape primitives. Starts at 0 so only overhang past the
/// reported scene origin is returned (enough to size SVG padding).
fn content_mins(scene: &Scene) -> (f64, f64) {
    let mut min_x: f64 = 0.0;
    let mut min_y: f64 = 0.0;

    for elem in scene.elements() {
        match &elem.primitive {
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
                let half_stroke = style.stroke_width.unwrap_or(0.0) / 2.0;
                let r = radius + half_stroke;
                min_x = min_x.min(center.x - r);
                min_y = min_y.min(center.y - r);
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
                for p in points {
                    min_x = min_x.min(p.x - half_stroke);
                    min_y = min_y.min(p.y - half_stroke);
                }
            }
            // Paths / groups / arcs: subgraph clipping is driven by Rect bounds;
            // edge paths sit inside those boxes for the layouts we care about.
            _ => {}
        }
    }

    (min_x, min_y)
}
