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
                let (_x1, _y1) = take_xy(&tokens, &mut i)?;
                let (x2, y2) = take_xy(&tokens, &mut i)?;
                let (x, y) = take_xy(&tokens, &mut i)?;
                let (x2, y2, x, y) = if cmd == 'c' {
                    (cur.0 + x2, cur.1 + y2, cur.0 + x, cur.1 + y)
                } else {
                    (x2, y2, x, y)
                };
                // The last control point sets the tangent at the curve end.
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
                let _rx = take_num(&tokens, &mut i)?;
                let _ry = take_num(&tokens, &mut i)?;
                let _rotation = take_num(&tokens, &mut i)?;
                let _large_arc = take_num(&tokens, &mut i)?;
                let _sweep = take_num(&tokens, &mut i)?;
                let (x, y) = take_xy(&tokens, &mut i)?;
                prev = cur;
                cur = if cmd == 'a' {
                    (cur.0 + x, cur.1 + y)
                } else {
                    (x, y)
                };
                second.get_or_insert(cur);
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
    /// Assert `Z` returns the current point to the subpath start.
    fn close_returns_to_start() {
        let ends = path_ends("M 0 0 L 10 0 Z").expect("path ends");
        assert_eq!(ends.end, (0.0, 0.0));
        assert_eq!(ends.end_dir, (-10.0, 0.0));
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
