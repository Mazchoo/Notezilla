//! Computes the endpoints and endpoint directions of an SVG path.

/// Endpoints of a path and the direction the path travels at each one.
pub struct PathEnds {
    pub start: (f64, f64),
    /// Direction from `start` towards the next point on the path.
    pub start_dir: (f64, f64),
    pub end: (f64, f64),
    /// Direction from the previous point on the path towards `end`.
    pub end_dir: (f64, f64),
}

/// Compute start/end points and directions from an SVG path `d` string.
///
/// Returns `None` for an empty path or an unsupported command.
pub fn path_ends(d: &str) -> Option<PathEnds> {
    let tokens = tokenize_path(d);
    let mut i = 0;
    let mut start = (0.0, 0.0);
    let mut cur = (0.0, 0.0);
    let mut prev = (0.0, 0.0);
    let mut second: Option<(f64, f64)> = None;
    let mut moved = false;
    let mut cmd = 'M';
    while i < tokens.len() {
        let token = &tokens[i];
        if token.len() == 1 && token.chars().next()?.is_ascii_alphabetic() {
            cmd = token.chars().next()?;
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
                cur = if cmd == 'm' && moved {
                    (cur.0 + x, cur.1 + y)
                } else {
                    (x, y)
                };
                start = cur;
                prev = cur;
                second = None;
                moved = true;
                // Repeated coordinate pairs after a moveto are implicit linetos.
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
                second.get_or_insert(cur);
            }
            'C' | 'c' => {
                let (x1, y1) = take_xy(&tokens, &mut i)?;
                let (x2, y2) = take_xy(&tokens, &mut i)?;
                let (x, y) = take_xy(&tokens, &mut i)?;
                let (x1, y1, x2, y2, x, y) = if cmd == 'c' {
                    (
                        cur.0 + x1,
                        cur.1 + y1,
                        cur.0 + x2,
                        cur.1 + y2,
                        cur.0 + x,
                        cur.1 + y,
                    )
                } else {
                    (x1, y1, x2, y2, x, y)
                };
                // The first control point sets the tangent at the curve start.
                second.get_or_insert((x1, y1));
                // The last control point sets the tangent at the curve end.
                prev = (x2, y2);
                cur = (x, y);
            }
            // Smooth cubics omit the reflected first control; two pairs remain.
            'S' | 's' => {
                let (x2, y2) = take_xy(&tokens, &mut i)?;
                let (x, y) = take_xy(&tokens, &mut i)?;
                let (x2, y2, x, y) = if cmd == 's' {
                    (cur.0 + x2, cur.1 + y2, cur.0 + x, cur.1 + y)
                } else {
                    (x2, y2, x, y)
                };
                prev = (x2, y2);
                cur = (x, y);
                second.get_or_insert(cur);
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
                second.get_or_insert(cur);
            }
            'A' | 'a' => {
                let rx = take_num(&tokens, &mut i)?;
                let ry = take_num(&tokens, &mut i)?;
                let rotation = take_num(&tokens, &mut i)?;
                let large_arc = take_num(&tokens, &mut i)? != 0.0;
                let sweep = take_num(&tokens, &mut i)? != 0.0;
                let (x, y) = take_xy(&tokens, &mut i)?;
                let to = if cmd == 'a' {
                    (cur.0 + x, cur.1 + y)
                } else {
                    (x, y)
                };
                let arc = Arc {
                    from: cur,
                    to,
                    radii: (rx, ry),
                    rotation,
                    large_arc,
                    sweep,
                };
                let (start_tangent, end_tangent) = arc_tangents(&arc);
                second.get_or_insert((cur.0 + start_tangent.0, cur.1 + start_tangent.1));
                // Step back along the end tangent so `end_dir` follows the curve.
                prev = (to.0 - end_tangent.0, to.1 - end_tangent.1);
                cur = to;
            }
            _ => return None,
        }
    }
    if !moved {
        return None;
    }
    let second = second.unwrap_or(cur);
    Some(PathEnds {
        start,
        start_dir: (second.0 - start.0, second.1 - start.1),
        end: cur,
        end_dir: (cur.0 - prev.0, cur.1 - prev.1),
    })
}

/// An SVG elliptical arc segment in endpoint form.
struct Arc {
    from: (f64, f64),
    to: (f64, f64),
    radii: (f64, f64),
    /// Rotation of the ellipse x-axis, in degrees.
    rotation: f64,
    large_arc: bool,
    sweep: bool,
}

/// Return the travel direction of `arc` at its start and at its end.
///
/// Converts the arc to centre form (SVG 1.1 appendix F.6.5). A zero radius
/// draws a straight line, so both tangents are the chord.
fn arc_tangents(arc: &Arc) -> ((f64, f64), (f64, f64)) {
    let chord = (arc.to.0 - arc.from.0, arc.to.1 - arc.from.1);
    let (mut rx, mut ry) = (arc.radii.0.abs(), arc.radii.1.abs());
    if rx == 0.0 || ry == 0.0 || chord == (0.0, 0.0) {
        return (chord, chord);
    }
    let (sin, cos) = arc.rotation.to_radians().sin_cos();
    // Half the chord, rotated into the ellipse frame.
    let (hx, hy) = (-chord.0 / 2.0, -chord.1 / 2.0);
    let x1 = cos * hx + sin * hy;
    let y1 = -sin * hx + cos * hy;
    // Radii too small to span the endpoints are scaled up until they do.
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }
    let numerator = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let sign = if arc.large_arc == arc.sweep { -1.0 } else { 1.0 };
    let coef = sign * (numerator / denominator).max(0.0).sqrt();
    let (cx, cy) = (coef * rx * y1 / ry, -coef * ry * x1 / rx);
    let start_angle = ((y1 - cy) / ry).atan2((x1 - cx) / rx);
    let end_angle = ((-y1 - cy) / ry).atan2((-x1 - cx) / rx);
    // A positive sweep travels towards increasing angle.
    let direction = if arc.sweep { 1.0 } else { -1.0 };
    let tangent = |angle: f64| {
        let (s, c) = angle.sin_cos();
        let (tx, ty) = (-rx * s * direction, ry * c * direction);
        (cos * tx - sin * ty, sin * tx + cos * ty)
    };
    (tangent(start_angle), tangent(end_angle))
}

/// Tokenize an SVG path `d` string into command letters and numbers.
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
        } else if c == '.' && !cur.is_empty() && (cur.contains('.') || cur.contains(['e', 'E'])) {
            // A second decimal, or a decimal after an exponent, starts a new number.
            out.push(std::mem::take(&mut cur));
            cur.push(c);
        } else if (c == '-' || c == '+') && !cur.is_empty() && !cur.ends_with(['e', 'E']) {
            // A sign starts a new number unless it belongs to an exponent.
            out.push(std::mem::take(&mut cur));
            cur.push(c);
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

#[cfg(test)]
mod tests {
    use super::{path_ends, tokenize_path};

    #[test]
    /// Assert commands and numbers split into separate tokens.
    fn tokenizes_commands_and_numbers() {
        assert_eq!(
            tokenize_path("M0,0 L10 -5"),
            vec!["M", "0", "0", "L", "10", "-5"]
        );
    }

    #[test]
    /// Assert a sign that starts the next number separates coordinates.
    fn minus_separates_coordinates() {
        let ends = path_ends("M0,0L10-5").expect("path ends");
        assert_eq!(ends.end, (10.0, -5.0));
    }

    #[test]
    /// Assert a second decimal point starts a new number.
    fn second_decimal_starts_a_new_number() {
        let ends = path_ends("M0 0 L1.5.5").expect("path ends");
        assert_eq!(ends.end, (1.5, 0.5));
    }

    #[test]
    /// Assert start geometry comes from the last subpath.
    fn last_subpath_owns_start_geometry() {
        let ends = path_ends("M 0 0 L 10 0 M 20 5 L 30 5").expect("path ends");
        assert_eq!(ends.start, (20.0, 5.0));
        assert_eq!(ends.start_dir, (10.0, 0.0));
        assert_eq!(ends.end, (30.0, 5.0));
    }

    #[test]
    /// Assert a horizontal line reports its ends and a rightward direction.
    fn line_reports_ends_and_direction() {
        let ends = path_ends("M 0 0 L 10 0").expect("path ends");
        assert_eq!(ends.start, (0.0, 0.0));
        assert_eq!(ends.end, (10.0, 0.0));
        assert_eq!(ends.end_dir, (10.0, 0.0));
        assert_eq!(ends.start_dir, (10.0, 0.0));
    }

    #[test]
    /// Assert relative linetos accumulate from the current point.
    fn relative_line_accumulates() {
        let ends = path_ends("m 5 5 l 5 0 l 5 0").expect("path ends");
        assert_eq!(ends.start, (5.0, 5.0));
        assert_eq!(ends.end, (15.0, 5.0));
        assert_eq!(ends.end_dir, (5.0, 0.0));
    }

    #[test]
    /// Assert a cubic curve takes its end direction from the last control point.
    fn cubic_end_direction_uses_last_control_point() {
        let ends = path_ends("M 0 0 C 0 10 10 10 10 20").expect("path ends");
        assert_eq!(ends.end, (10.0, 20.0));
        assert_eq!(ends.end_dir, (0.0, 10.0));
    }

    #[test]
    /// Assert a cubic start direction follows the first control point.
    fn cubic_start_direction_uses_first_control_point() {
        let ends = path_ends("M 0 0 C 10 0 10 10 0 10").expect("path ends");
        assert_eq!(ends.start_dir, (10.0, 0.0));
    }

    #[test]
    /// Assert a smooth cubic reports an end point.
    fn smooth_cubic_reports_end() {
        let ends = path_ends("M 0 0 C 0 10 10 10 10 20 S 20 30 20 40").expect("path ends");
        assert_eq!(ends.end, (20.0, 40.0));
    }

    #[test]
    /// Assert `Z` returns the current point to the subpath start.
    fn close_returns_to_start() {
        let ends = path_ends("M 0 0 L 10 0 Z").expect("path ends");
        assert_eq!(ends.end, (0.0, 0.0));
        assert_eq!(ends.end_dir, (-10.0, 0.0));
    }

    /// Assert `actual` points the same way as `expected`.
    fn assert_same_direction(actual: (f64, f64), expected: (f64, f64)) {
        let cross = actual.0 * expected.1 - actual.1 * expected.0;
        let dot = actual.0 * expected.0 + actual.1 * expected.1;
        assert!(
            cross.abs() < 1e-9 && dot > 0.0,
            "expected direction {expected:?}, got {actual:?}"
        );
    }

    #[test]
    /// Assert an arc end direction is not the chord of the endpoints.
    fn arc_end_direction_is_not_the_chord() {
        let ends = path_ends("M 0 0 A 10 10 0 0 1 10 0").expect("path ends");
        assert_eq!(ends.end, (10.0, 0.0));
        assert_same_direction(ends.start_dir, (3f64.sqrt(), -1.0));
        assert_same_direction(ends.end_dir, (3f64.sqrt(), 1.0));
    }

    #[test]
    /// Assert a negative sweep bends the arc the other way.
    fn negative_sweep_arc_mirrors_tangents() {
        let ends = path_ends("M 0 0 A 10 10 0 0 0 10 0").expect("path ends");
        assert_same_direction(ends.start_dir, (3f64.sqrt(), 1.0));
        assert_same_direction(ends.end_dir, (3f64.sqrt(), -1.0));
    }

    #[test]
    /// Assert a semicircle arrives perpendicular to its diameter.
    fn semicircle_ends_perpendicular_to_diameter() {
        let ends = path_ends("M 0 0 a 5 5 0 0 1 10 0").expect("path ends");
        assert_same_direction(ends.start_dir, (0.0, -1.0));
        assert_same_direction(ends.end_dir, (0.0, 1.0));
    }

    #[test]
    /// Assert radii too small for the chord are scaled up to a semicircle.
    fn undersized_radii_scale_to_semicircle() {
        let ends = path_ends("M 0 0 A 1 1 0 0 1 10 0").expect("path ends");
        assert_same_direction(ends.end_dir, (0.0, 1.0));
    }

    #[test]
    /// Assert a large-arc flag travels the long way round.
    fn large_arc_takes_the_long_way() {
        let ends = path_ends("M 0 0 A 10 10 0 1 1 10 0").expect("path ends");
        assert_same_direction(ends.start_dir, (-3f64.sqrt(), -1.0));
        assert_same_direction(ends.end_dir, (-3f64.sqrt(), 1.0));
    }

    #[test]
    /// Assert a rotated ellipse rotates its tangents with it.
    fn rotated_ellipse_rotates_tangents() {
        let ends = path_ends("M 0 0 A 10 5 90 0 1 0 10").expect("path ends");
        // Centre (-2.5√3, 5); the end sits 60° short of the minor-axis vertex.
        assert_same_direction(ends.end_dir, (-1.0, 2.0 * 3f64.sqrt()));
    }

    #[test]
    /// Assert a zero-radius arc is a straight line.
    fn zero_radius_arc_is_a_line() {
        let ends = path_ends("M 0 0 A 0 5 0 0 1 10 0").expect("path ends");
        assert_eq!(ends.end_dir, (10.0, 0.0));
        assert_eq!(ends.start_dir, (10.0, 0.0));
    }

    #[test]
    /// Assert an empty path has no ends.
    fn empty_path_has_no_ends() {
        assert!(path_ends("").is_none());
    }

    #[test]
    /// Assert an unsupported command yields `None` rather than a panic.
    fn unsupported_command_is_none() {
        assert!(path_ends("M 0 0 H 10").is_none());
    }

    #[test]
    /// Assert a truncated coordinate pair yields `None` rather than a panic.
    fn truncated_coordinates_are_none() {
        assert!(path_ends("M 0 0 L 10").is_none());
    }
}
