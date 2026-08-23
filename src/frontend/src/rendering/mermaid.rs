use super::escape_html;
use super::pdf_colors::{TEXT, TEXT_SUBTLE};
use rusty_mermaid::svg::SvgRenderer;
use rusty_mermaid::{render, Color, Primitive, Scene, Theme};

/// rusty-mermaid measures labels as Intel One Mono (~0.6em). Ironpress maps
/// that family to Helvetica, so PDF uses Courier (standard 14 monospace).
const PDF_FONT_FAMILY: &str = "Courier, monospace";
/// Matches rusty_mermaid_core::constants::BASELINE_ASCENT_RATIO.
const BASELINE_ASCENT_RATIO: f64 = 0.3;
const ARROW_SIZE: f64 = 8.0;

/// Render a Mermaid diagram source string to an inline SVG string.
pub fn render_mermaid(src: &str) -> String {
    // The dark theme's text matches `pdf_colors::TEXT` so labels read
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
            let svg = SvgRenderer::with_theme(&svg_theme).render_themed(&scene, &svg_theme);
            prepare_mermaid_svg(&svg)
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

/// Flatten mermaid SVG labels and expand marker arrows for PDF layout.
fn prepare_mermaid_svg(svg: &str) -> String {
    let labeled = rewrite_text_elements(svg);
    expand_path_markers(&labeled)
}

/// Replace mermaid `<text>` elements with flattened baseline labels.
fn rewrite_text_elements(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text") {
        out.push_str(&rest[..start]);
        let Some(end_rel) = rest[start..].find("</text>") else {
            out.push_str(rest);
            return out;
        };
        let end = start + end_rel + "</text>".len();
        out.push_str(&flatten_label(&rest[start..end]));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Flatten one mermaid `<text>` element onto alphabetic baselines.
fn flatten_label(elem: &str) -> String {
    let gt = match elem.find('>') {
        Some(i) => i,
        None => return elem.to_string(),
    };
    let open = &elem[..gt];
    let Some(x) = attr(open, "x").and_then(|v| v.parse::<f64>().ok()) else {
        return elem.to_string();
    };
    let Some(y) = attr(open, "y").and_then(|v| v.parse::<f64>().ok()) else {
        return elem.to_string();
    };
    let font_size = attr(open, "font-size")
        .and_then(|v| parse_font_px(&v))
        .unwrap_or(14.0);
    let anchor = attr(open, "text-anchor").unwrap_or_else(|| "middle".into());
    let fill = attr(open, "fill").unwrap_or_else(|| TEXT.to_string());
    let parent_bold = attr(open, "font-weight").as_deref() == Some("bold");
    let parent_italic = attr(open, "font-style").as_deref() == Some("italic");
    let lines = label_lines(&elem[gt + 1..], x, y);
    if lines.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for line in lines {
        let baseline = line.y + font_size * BASELINE_ASCENT_RATIO;
        let bold = parent_bold || line.bold;
        let italic = parent_italic || line.italic;
        out.push_str(&format!(
            "<text text-anchor=\"{anchor}\" x=\"{}\" y=\"{baseline}\" font-size=\"{font_size}px\" \
             font-family=\"{PDF_FONT_FAMILY}\" fill=\"{fill}\"",
            line.x
        ));
        if bold {
            out.push_str(" font-weight=\"bold\"");
        }
        if italic {
            out.push_str(" font-style=\"italic\"");
        }
        out.push_str(&format!(">{}</text>", line.text));
    }
    out
}

struct LabelLine {
    x: f64,
    y: f64,
    text: String,
    bold: bool,
    italic: bool,
}

/// Collect label lines from `<tspan>` children or plain text.
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
        let self_closing = open.ends_with('/');
        let (content, next) = if self_closing {
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
        let text = strip_tags(content);
        let x = attr(open, "x")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(default_x);
        if let Some(dy) = attr(open, "dy").and_then(|v| v.parse::<f64>().ok()) {
            if dy != 0.0 {
                saw_dy = true;
            }
            y_cursor += dy;
        }
        if !text.is_empty() {
            lines.push(LabelLine {
                x,
                y: y_cursor,
                text,
                bold: attr(open, "font-weight").as_deref() == Some("bold"),
                italic: attr(open, "font-style").as_deref() == Some("italic"),
            });
        }
        rest = next;
    }
    if lines.is_empty() {
        let text = strip_tags(inner.trim().trim_end_matches("</text>").trim());
        if !text.is_empty() {
            lines.push(LabelLine {
                x: default_x,
                y: parent_y,
                text,
                bold: false,
                italic: false,
            });
        }
        return lines;
    }
    if !saw_dy {
        let bold = lines.iter().all(|l| l.bold);
        let italic = lines.iter().all(|l| l.italic);
        let text = lines.iter().map(|l| l.text.as_str()).collect::<String>();
        let x = lines[0].x;
        return vec![LabelLine {
            x,
            y: parent_y,
            text,
            bold,
            italic,
        }];
    }
    lines
}

/// Strip HTML/SVG tags from `s`, keeping text content.
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

/// Parse a CSS `font-size` value in pixels.
fn parse_font_px(v: &str) -> Option<f64> {
    v.trim()
        .trim_end_matches("px")
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|n| *n > 0.0)
}

/// Replace SVG marker refs on paths with painted arrow polygons.
fn expand_path_markers(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len() + 256);
    let mut rest = svg;
    while let Some(start) = rest.find("<path") {
        out.push_str(&rest[..start]);
        let rel_sc = rest[start..].find("/>");
        let rel_close = rest[start..].find("</path>");
        let (elem_end, after) = match (rel_sc, rel_close) {
            (Some(sc), Some(cl)) if sc < cl => (start + sc + 2, start + sc + 2),
            (Some(sc), None) => (start + sc + 2, start + sc + 2),
            (_, Some(cl)) => (start + cl + 7, start + cl + 7),
            _ => {
                out.push_str(rest);
                return out;
            }
        };
        let elem = &rest[start..elem_end];
        let marker_end = attr(elem, "marker-end");
        let marker_start = attr(elem, "marker-start");
        if marker_end.is_none() && marker_start.is_none() {
            out.push_str(elem);
            rest = &rest[after..];
            continue;
        }
        let stripped = strip_attr(&strip_attr(elem, "marker-end"), "marker-start");
        out.push_str(&stripped);
        if let Some(d) = attr(elem, "d") {
            if let Some(ends) = path_ends(&d) {
                let stroke = attr(elem, "stroke");
                if let Some(url) = marker_end.as_deref() {
                    let color = marker_color(url, stroke.as_deref());
                    out.push_str(&arrow_polygon(ends.end, ends.end_dir, &color));
                }
                if let Some(url) = marker_start.as_deref() {
                    let color = marker_color(url, stroke.as_deref());
                    out.push_str(&arrow_polygon(
                        ends.start,
                        (-ends.start_dir.0, -ends.start_dir.1),
                        &color,
                    ));
                }
            }
        }
        rest = &rest[after..];
    }
    out.push_str(rest);
    out
}

/// Remove attribute `name` from an SVG element string.
fn strip_attr(elem: &str, name: &str) -> String {
    let Some(val) = attr(elem, name) else {
        return elem.to_string();
    };
    let needle_dq = format!("{name}=\"{val}\"");
    let needle_sq = format!("{name}='{val}'");
    elem.replace(&needle_dq, "").replace(&needle_sq, "")
}

/// Resolve an arrow color from a marker URL or the path stroke.
fn marker_color(url: &str, stroke: Option<&str>) -> String {
    let id = url.trim().trim_start_matches("url(#").trim_end_matches(')');
    if let Some((_, hex)) = id.rsplit_once('-') {
        if hex.chars().all(|c| c.is_ascii_hexdigit()) && matches!(hex.len(), 3 | 6 | 8) {
            return format!("#{hex}");
        }
    }
    stroke.unwrap_or(TEXT_SUBTLE).to_string()
}

struct PathEnds {
    start: (f64, f64),
    start_dir: (f64, f64),
    end: (f64, f64),
    end_dir: (f64, f64),
}

/// Compute start/end points and directions from an SVG path `d` string.
fn path_ends(d: &str) -> Option<PathEnds> {
    let tokens = tokenize_path(d);
    let mut i = 0;
    let mut start = (0.0, 0.0);
    let mut cur = (0.0, 0.0);
    let mut prev = (0.0, 0.0);
    let mut first: Option<(f64, f64)> = None;
    let mut has = false;
    let mut cmd = 'M';
    while i < tokens.len() {
        let t = &tokens[i];
        if t.len() == 1 && t.chars().next()?.is_ascii_alphabetic() {
            cmd = t.chars().next()?;
            i += 1;
            if cmd == 'Z' || cmd == 'z' {
                prev = cur;
                cur = start;
                continue;
            }
        }
        match cmd {
            'M' | 'm' => {
                let (x, y) = take_xy(&tokens, &mut i)?;
                cur = if cmd == 'm' && has {
                    (cur.0 + x, cur.1 + y)
                } else {
                    (x, y)
                };
                start = cur;
                prev = cur;
                has = true;
                cmd = if cmd == 'm' { 'l' } else { 'L' };
            }
            'L' | 'l' => {
                let (x, y) = take_xy(&tokens, &mut i)?;
                prev = cur;
                cur = if cmd == 'l' {
                    (cur.0 + x, cur.1 + y)
                } else {
                    (x, y)
                };
                first.get_or_insert(cur);
            }
            'C' | 'c' => {
                let (_x1, _y1) = take_xy(&tokens, &mut i)?;
                let (x2, y2) = take_xy(&tokens, &mut i)?;
                let (x, y) = take_xy(&tokens, &mut i)?;
                let (x2, y2, x, y) = if cmd == 'c' {
                    (cur.0 + x2, cur.1 + y2, cur.0 + x, cur.1 + y)
                } else {
                    (x2, y2, x, y)
                };
                prev = (x2, y2);
                cur = (x, y);
                first.get_or_insert(cur);
            }
            'Q' | 'q' => {
                let (cx, cy) = take_xy(&tokens, &mut i)?;
                let (x, y) = take_xy(&tokens, &mut i)?;
                let (cx, cy, x, y) = if cmd == 'q' {
                    (cur.0 + cx, cur.1 + cy, cur.0 + x, cur.1 + y)
                } else {
                    (cx, cy, x, y)
                };
                prev = (cx, cy);
                cur = (x, y);
                first.get_or_insert(cur);
            }
            'A' | 'a' => {
                let _rx = take_num(&tokens, &mut i)?;
                let _ry = take_num(&tokens, &mut i)?;
                let _rot = take_num(&tokens, &mut i)?;
                let _large = take_num(&tokens, &mut i)?;
                let _sweep = take_num(&tokens, &mut i)?;
                let (x, y) = take_xy(&tokens, &mut i)?;
                prev = cur;
                cur = if cmd == 'a' {
                    (cur.0 + x, cur.1 + y)
                } else {
                    (x, y)
                };
                first.get_or_insert(cur);
            }
            _ => return None,
        }
    }
    if !has {
        return None;
    }
    let first = first.unwrap_or(cur);
    Some(PathEnds {
        start,
        start_dir: (first.0 - start.0, first.1 - start.1),
        end: cur,
        end_dir: (cur.0 - prev.0, cur.1 - prev.1),
    })
}

/// Tokenize an SVG path `d` string into commands and numbers.
fn tokenize_path(d: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in d.chars() {
        if c.is_ascii_alphabetic() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            out.push(c.to_string());
        } else if c == ',' || c.is_whitespace() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Consume the next numeric token from `tokens`.
fn take_num(tokens: &[String], i: &mut usize) -> Option<f64> {
    let n = tokens.get(*i)?.parse().ok()?;
    *i += 1;
    Some(n)
}

/// Consume the next `x,y` pair from `tokens`.
fn take_xy(tokens: &[String], i: &mut usize) -> Option<(f64, f64)> {
    let x = take_num(tokens, i)?;
    let y = take_num(tokens, i)?;
    Some((x, y))
}

/// Build an SVG polygon for an arrowhead at `tip` facing `dir`.
fn arrow_polygon(tip: (f64, f64), dir: (f64, f64), color: &str) -> String {
    let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
    if len < 1e-6 {
        return String::new();
    }
    let ux = dir.0 / len;
    let uy = dir.1 / len;
    let half = ARROW_SIZE * 0.45;
    let bx = tip.0 - ux * ARROW_SIZE;
    let by = tip.1 - uy * ARROW_SIZE;
    let px = -uy * half;
    let py = ux * half;
    format!(
        "<polygon points=\"{},{} {},{} {},{}\" fill=\"{color}\" />",
        tip.0,
        tip.1,
        bx + px,
        by + py,
        bx - px,
        by - py
    )
}

/// Read a quoted attribute value from an SVG tag string.
fn attr(tag: &str, name: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let name_b = name.as_bytes();
    let mut i = 0;
    while i + name_b.len() < bytes.len() {
        if bytes[i..].starts_with(name_b) && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric()) {
            let mut j = i + name_b.len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'=' {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                    let quote = bytes[j];
                    j += 1;
                    let start = j;
                    while j < bytes.len() && bytes[j] != quote {
                        j += 1;
                    }
                    return Some(tag[start..j].to_string());
                }
            }
        }
        i += 1;
    }
    None
}

/// Return the lowest x/y among scene primitives that overhang the origin.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assert flowchart labels use an alphabetic baseline and Courier.
    fn flowchart_labels_use_baseline_and_courier() {
        let svg = render_mermaid("graph LR\n    A[Square Rect] --> B((Circle))\n");
        assert!(
            !svg.contains("dominant-baseline"),
            "ironpress ignores dominant-baseline: {svg}"
        );
        assert!(svg.contains("font-family=\"Courier, monospace\""), "{svg}");
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
        let svg = render_mermaid("pie title Pets\n\"Dogs\" : 2\n\"Cats\" : 3\n");
        assert!(svg.contains("<svg"), "{svg}");
        assert!(!svg.contains("dominant-baseline"), "{svg}");
        assert!(svg.contains("font-family=\"Courier, monospace\""), "{svg}");
    }
}
