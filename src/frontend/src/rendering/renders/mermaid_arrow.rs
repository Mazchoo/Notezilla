//! Replaces SVG marker arrowheads with painted polygons.
//!
//! Ironpress does not resolve `marker-start` / `marker-end` references, so each
//! marked path keeps its geometry and gains an explicit arrowhead polygon.

use crate::constants::{ARROW_SIZE, TEXT_SUBTLE};
use crate::rendering::svg_attr::{strip_svg_attr, svg_attr};
use crate::rendering::svg_path_ends::path_ends;

/// Replace marker references on every path with painted arrow polygons.
pub fn expand_path_markers(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len() + 256);
    let mut rest = svg;
    while let Some(start) = rest.find("<path") {
        out.push_str(&rest[..start]);
        let Some(end) = path_element_end(&rest[start..]) else {
            out.push_str(&rest[start..]);
            return out;
        };
        let end = start + end;
        out.push_str(&expand_element_markers(&rest[start..end]));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Return the length of the `<path>` element at the start of `rest`.
fn path_element_end(rest: &str) -> Option<usize> {
    let self_closing = rest.find("/>");
    let closing_tag = rest.find("</path>");
    match (self_closing, closing_tag) {
        (Some(at), None) => Some(at + "/>".len()),
        (Some(at), Some(tag_at)) if at < tag_at => Some(at + "/>".len()),
        (_, Some(at)) => Some(at + "</path>".len()),
        (None, None) => None,
    }
}

/// Return `element` without marker references, followed by its arrow polygons.
fn expand_element_markers(element: &str) -> String {
    let marker_end = svg_attr(element, "marker-end");
    let marker_start = svg_attr(element, "marker-start");
    if marker_end.is_none() && marker_start.is_none() {
        return element.to_string();
    }

    let mut out = strip_svg_attr(&strip_svg_attr(element, "marker-end"), "marker-start");
    let Some(geometry) = svg_attr(element, "d") else {
        return out;
    };
    let Some(ends) = path_ends(&geometry) else {
        return out;
    };
    let stroke = svg_attr(element, "stroke");
    if let Some(url) = marker_end.as_deref() {
        let color = marker_color(url, stroke.as_deref());
        out.push_str(&arrow_polygon(ends.end, ends.end_dir, &color));
    }
    if let Some(url) = marker_start.as_deref() {
        let color = marker_color(url, stroke.as_deref());
        // The start arrowhead points back out of the path.
        let direction = (-ends.start_dir.0, -ends.start_dir.1);
        out.push_str(&arrow_polygon(ends.start, direction, &color));
    }
    out
}

/// Resolve an arrow color from a marker URL, else from the path stroke.
///
/// rusty-mermaid encodes the marker paint as a hex suffix of the marker id.
fn marker_color(url: &str, stroke: Option<&str>) -> String {
    let id = url.trim().trim_start_matches("url(#").trim_end_matches(')');
    if let Some((_, hex)) = id.rsplit_once('-') {
        if hex.chars().all(|c| c.is_ascii_hexdigit()) && matches!(hex.len(), 3 | 6 | 8) {
            return format!("#{hex}");
        }
    }
    stroke.unwrap_or(TEXT_SUBTLE).to_string()
}

/// Build an SVG polygon for an arrowhead with its tip at `tip` facing `dir`.
///
/// Returns an empty string when `dir` has no length, leaving the path bare.
fn arrow_polygon(tip: (f64, f64), dir: (f64, f64), color: &str) -> String {
    let length = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
    if length < 1e-6 {
        return String::new();
    }
    let (ux, uy) = (dir.0 / length, dir.1 / length);
    let half_base = ARROW_SIZE * 0.45;
    let (base_x, base_y) = (tip.0 - ux * ARROW_SIZE, tip.1 - uy * ARROW_SIZE);
    let (offset_x, offset_y) = (-uy * half_base, ux * half_base);
    format!(
        "<polygon points=\"{},{} {},{} {},{}\" fill=\"{color}\" />",
        tip.0,
        tip.1,
        base_x + offset_x,
        base_y + offset_y,
        base_x - offset_x,
        base_y - offset_y
    )
}

#[cfg(test)]
mod tests {
    use super::{arrow_polygon, expand_path_markers, marker_color, path_element_end};
    use crate::constants::{ARROW_SIZE, TEXT_SUBTLE};

    #[test]
    /// Assert a self-closing path element ends at `/>`.
    fn finds_self_closing_element_end() {
        let svg = r#"<path d="M0 0" /><rect/>"#;
        let end = path_element_end(svg).expect("element end");
        assert_eq!(&svg[..end], r#"<path d="M0 0" />"#);
    }

    #[test]
    /// Assert a path element with a closing tag ends at `</path>`.
    fn finds_closing_tag_element_end() {
        let svg = r#"<path d="M0 0"></path><rect/>"#;
        let end = path_element_end(svg).expect("element end");
        assert_eq!(&svg[..end], r#"<path d="M0 0"></path>"#);
    }

    #[test]
    /// Assert an unterminated path element has no end.
    fn unterminated_element_has_no_end() {
        assert_eq!(path_element_end("<path d=\"M0 0\""), None);
    }

    #[test]
    /// Assert a marker id's hex suffix becomes the arrow color.
    fn marker_id_hex_suffix_is_the_arrow_color() {
        assert_eq!(marker_color("url(#arrow-ff0000)", None), "#ff0000");
    }

    #[test]
    /// Assert a marker id without a hex suffix falls back to the path stroke.
    fn marker_without_hex_suffix_uses_stroke() {
        assert_eq!(marker_color("url(#arrow)", Some("#123456")), "#123456");
    }

    #[test]
    /// Assert a marker with neither hex suffix nor stroke uses the subtle text color.
    fn marker_without_stroke_uses_subtle_text() {
        assert_eq!(marker_color("url(#arrow)", None), TEXT_SUBTLE);
    }

    #[test]
    /// Assert the arrowhead is a triangle of `ARROW_SIZE` behind the tip.
    fn arrow_polygon_is_a_triangle_behind_the_tip() {
        let polygon = arrow_polygon((10.0, 0.0), (1.0, 0.0), "#fff");
        assert!(polygon.starts_with("<polygon points=\"10,0 "), "{polygon}");
        assert!(polygon.contains(&format!("{},", 10.0 - ARROW_SIZE)), "{polygon}");
        assert!(polygon.contains("fill=\"#fff\""), "{polygon}");
    }

    #[test]
    /// Assert a zero-length direction yields no arrowhead.
    fn zero_length_direction_yields_no_arrow() {
        assert_eq!(arrow_polygon((0.0, 0.0), (0.0, 0.0), "#fff"), "");
    }

    #[test]
    /// Assert a marked path loses its marker refs and gains a polygon.
    fn marked_path_gains_polygon() {
        let out = expand_path_markers(
            r##"<path d="M0 0 L10 0" stroke="#abc" marker-end="url(#a-ff0000)" />"##,
        );
        assert!(!out.contains("marker-end"), "{out}");
        assert!(out.contains("<polygon"), "{out}");
        assert!(out.contains("fill=\"#ff0000\""), "{out}");
        assert!(out.contains(r#"d="M0 0 L10 0""#), "{out}");
    }

    #[test]
    /// Assert a start marker points back out of the path.
    fn start_marker_points_backwards() {
        let out =
            expand_path_markers(r##"<path d="M10 0 L20 0" marker-start="url(#a-ff0000)" />"##);
        assert!(out.contains("<polygon points=\"10,0 "), "{out}");
        assert!(out.contains(&format!("{},", 10.0 + ARROW_SIZE)), "{out}");
    }

    #[test]
    /// Assert a path without markers is copied through unchanged.
    fn unmarked_path_is_unchanged() {
        let svg = r#"<g><path d="M0 0 L1 1" /></g>"#;
        assert_eq!(expand_path_markers(svg), svg);
    }

    #[test]
    /// Assert a marked path with an unusable `d` still loses its marker refs.
    fn marked_path_without_geometry_drops_markers() {
        let out = expand_path_markers(r##"<path d="" marker-end="url(#a-ff0000)" />"##);
        assert!(!out.contains("marker-end"), "{out}");
        assert!(!out.contains("<polygon"), "{out}");
    }

    #[test]
    /// Assert markup outside path elements is preserved.
    fn keeps_surrounding_markup() {
        let out = expand_path_markers(r#"<svg><path d="M0 0 L1 0" /><rect x="1"/></svg>"#);
        assert!(out.starts_with("<svg>"), "{out}");
        assert!(out.ends_with("</svg>"), "{out}");
        assert!(out.contains(r#"<rect x="1"/>"#), "{out}");
    }
}
