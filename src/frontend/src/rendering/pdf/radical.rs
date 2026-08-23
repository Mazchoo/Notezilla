//! Corrects the height of the radical sign in ironpress math output.
//!
//! Delete this module, and its single call in [`super`], once ironpress paints
//! the radical sign at the right height. Its layout anchors a radical at the
//! radicand baseline (`MathGlyph::Radical { y: 0.0, .. }`), but its PDF
//! renderer paints the sign downwards from that anchor as if it were the top of
//! the sign, so the surd lands a full radical height below the overbar it has
//! to meet. Confirmed in ironpress 1.5.2 and 1.5.3.
//!
//! The painted shape is correct, so each radical path is translated vertically
//! until its apex meets the overbar rectangle that follows it in the content
//! stream. A path that already meets its overbar is left untouched, so the pass
//! becomes a no-op rather than a second error once ironpress is fixed.

/// Fraction of the sign width at which ironpress starts the tick.
const TICK_X_RATIO: f32 = 0.15;
/// Fraction of the sign width at which the tick meets the diagonal.
const FOOT_X_RATIO: f32 = 0.35;
/// Fraction of the sign height at which the tick starts below the apex.
const TICK_Y_RATIO: f32 = 0.3;
/// Share of the sign height by which the parsed points may miss the ratios.
///
/// The points are printed as shortest-round-trip `f32`, so an exact match is
/// only lost to the ratio arithmetic itself.
const SHAPE_TOLERANCE: f32 = 0.02;
/// Vertical error in PDF units below which a path is treated as correct.
const PLACED_TOLERANCE: f32 = 0.01;

/// A radical sign path as ironpress paints it.
struct RadicalPath {
    /// Index just past the `S` operator that strokes the path.
    end: usize,
    tick: (f32, f32),
    foot: (f32, f32),
    apex: (f32, f32),
}

/// The overbar rectangle ironpress paints directly after a radical sign.
struct Overbar {
    center_y: f32,
    thickness: f32,
}

/// Translate every radical sign path onto the overbar that follows it.
pub fn correct_radicals(content: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(content.len());
    let mut i = 0;
    while i < content.len() {
        match placed_radical_at(content, i) {
            Some((path, end)) => {
                out.extend_from_slice(path.as_bytes());
                i = end;
            }
            None => {
                out.push(content[i]);
                i += 1;
            }
        }
    }
    out
}

/// Return the corrected path at `i` and the index just past the original.
fn placed_radical_at(content: &[u8], i: usize) -> Option<(String, usize)> {
    if !is_operator_start(content, i) {
        return None;
    }
    let path = parse_radical_path(content, i)?;
    let overbar = parse_overbar(content, path.end)?;
    // Ironpress lays the sign out `total_height - descent` above the baseline,
    // which is one rule thickness above the center of the overbar.
    let apex_y = overbar.center_y + overbar.thickness;
    let dy = apex_y - path.apex.1;
    if dy.abs() < PLACED_TOLERANCE {
        return None;
    }
    let text = format!(
        "{} {} m\n{} {} l\n{} {} l\nS",
        path.tick.0,
        path.tick.1 + dy,
        path.foot.0,
        path.foot.1 + dy,
        path.apex.0,
        apex_y
    );
    Some((text, path.end))
}

/// Parse the three-point stroked path ironpress paints for a radical sign.
///
/// Returns `None` for any other path: the tick and apex have to sit at the
/// ratios of the sign that ironpress draws.
fn parse_radical_path(content: &[u8], i: usize) -> Option<RadicalPath> {
    let (tick, i) = parse_point(content, i, b"m")?;
    let (foot, i) = parse_point(content, i, b"l")?;
    let (apex, i) = parse_point(content, i, b"l")?;
    let end = parse_operator(content, skip_whitespace(content, i), b"S")?;

    let width = (foot.0 - tick.0) / (FOOT_X_RATIO - TICK_X_RATIO);
    let height = apex.1 - foot.1;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let x = apex.0 - width;
    let tolerance = height * SHAPE_TOLERANCE;
    if (tick.0 - (x + width * TICK_X_RATIO)).abs() > tolerance
        || (tick.1 - (apex.1 - height * TICK_Y_RATIO)).abs() > tolerance
    {
        return None;
    }
    Some(RadicalPath {
        end,
        tick,
        foot,
        apex,
    })
}

/// Parse the filled rectangle ironpress paints for the overbar.
///
/// The fill color operator that precedes the rectangle is optional so the pass
/// does not depend on the color ironpress happens to emit.
fn parse_overbar(content: &[u8], i: usize) -> Option<Overbar> {
    let i = skip_whitespace(content, i);
    let i = parse_fill_color(content, i).unwrap_or(i);
    let (_x, i) = parse_number(content, i)?;
    let (y, i) = parse_number(content, skip_whitespace(content, i))?;
    let (_width, i) = parse_number(content, skip_whitespace(content, i))?;
    let (thickness, i) = parse_number(content, skip_whitespace(content, i))?;
    let i = parse_operator(content, skip_whitespace(content, i), b"re")?;
    parse_operator(content, skip_whitespace(content, i), b"f")?;
    if thickness <= 0.0 {
        return None;
    }
    Some(Overbar {
        center_y: y + thickness / 2.0,
        thickness,
    })
}

/// Return the index past a `r g b rg` operator, if one starts at `i`.
fn parse_fill_color(content: &[u8], i: usize) -> Option<usize> {
    let (_red, i) = parse_number(content, i)?;
    let (_green, i) = parse_number(content, skip_whitespace(content, i))?;
    let (_blue, i) = parse_number(content, skip_whitespace(content, i))?;
    let i = parse_operator(content, skip_whitespace(content, i), b"rg")?;
    Some(skip_whitespace(content, i))
}

/// Parse `x y operator` and return the point and the index past the operator.
fn parse_point(content: &[u8], i: usize, operator: &[u8]) -> Option<((f32, f32), usize)> {
    let i = skip_whitespace(content, i);
    let (x, i) = parse_number(content, i)?;
    let (y, i) = parse_number(content, skip_whitespace(content, i))?;
    let i = parse_operator(content, skip_whitespace(content, i), operator)?;
    Some(((x, y), i))
}

/// Parse one PDF real number and return it with the index past its digits.
fn parse_number(content: &[u8], i: usize) -> Option<(f32, usize)> {
    let end = i + content[i..]
        .iter()
        .take_while(|b| matches!(**b, b'0'..=b'9' | b'-' | b'+' | b'.'))
        .count();
    let text = std::str::from_utf8(content.get(i..end)?).ok()?;
    Some((text.parse().ok()?, end))
}

/// Return the index past `operator` when it is the whole token at `i`.
fn parse_operator(content: &[u8], i: usize, operator: &[u8]) -> Option<usize> {
    if !content.get(i..)?.starts_with(operator) {
        return None;
    }
    let end = i + operator.len();
    match content.get(end) {
        None => Some(end),
        Some(byte) if is_whitespace(*byte) => Some(end),
        Some(_) => None,
    }
}

/// Return the index of the first byte at or after `i` that is not whitespace.
fn skip_whitespace(content: &[u8], i: usize) -> usize {
    i + content[i..]
        .iter()
        .take_while(|b| is_whitespace(**b))
        .count()
}

/// Return whether index `i` starts a PDF operand or operator token.
fn is_operator_start(content: &[u8], i: usize) -> bool {
    i == 0 || is_whitespace(content[i - 1])
}

/// Return whether `byte` separates PDF tokens.
fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b'\n' | b'\r' | b' ' | b'\t')
}

#[cfg(test)]
mod tests {
    use super::{correct_radicals, parse_radical_path, FOOT_X_RATIO, TICK_X_RATIO, TICK_Y_RATIO};

    /// Ironpress geometry of one radical: baseline, ascent, descent, thickness.
    struct Sign {
        x: f32,
        baseline: f32,
        width: f32,
        ascent: f32,
        descent: f32,
        thickness: f32,
    }

    impl Sign {
        /// Return the sign ironpress paints for an 11pt radicand.
        fn sample() -> Self {
            Self {
                x: 293.9,
                baseline: 762.36,
                width: 6.6,
                ascent: 11.59,
                descent: 2.42,
                thickness: 0.44,
            }
        }

        /// Return the full height ironpress gives the sign.
        fn height(&self) -> f32 {
            self.ascent + self.descent + self.thickness
        }

        /// Return the content stream ironpress emits, with the apex at `apex_y`.
        fn stream(&self, apex_y: f32) -> Vec<u8> {
            let height = self.height();
            let overbar_y = self.baseline + self.ascent - self.thickness / 2.0;
            format!(
                "0.44 w\n0 0 0 RG\n{tick_x} {tick_y} m\n{foot_x} {foot_y} l\n{apex_x} {apex_y} l\nS\n\
                 0 0 0 rg\n{bar_x} {overbar_y} 36.59 {thickness} re\nf\n",
                tick_x = self.x + self.width * TICK_X_RATIO,
                tick_y = apex_y - height * TICK_Y_RATIO,
                foot_x = self.x + self.width * FOOT_X_RATIO,
                foot_y = apex_y - height,
                apex_x = self.x + self.width,
                bar_x = self.x + self.width,
                thickness = self.thickness,
            )
            .into_bytes()
        }

        /// Return the apex ironpress paints: the radicand baseline.
        fn painted_apex(&self) -> f32 {
            self.baseline
        }

        /// Return the apex the sign needs: one thickness above the bar center.
        fn correct_apex(&self) -> f32 {
            self.baseline + self.ascent + self.thickness
        }
    }

    /// Return the apex y of the only radical path in `content`.
    fn apex_of(content: &[u8]) -> f32 {
        let at = content
            .windows(2)
            .position(|w| w == b"m ")
            .or_else(|| content.windows(2).position(|w| w == b" m"))
            .expect("path start");
        let start = content[..at]
            .iter()
            .rposition(|b| *b == b'\n')
            .map(|nl| nl + 1)
            .unwrap_or(0);
        parse_radical_path(content, start)
            .expect("radical path")
            .apex
            .1
    }

    #[test]
    /// Assert the painted apex sits a full sign height below the overbar.
    fn ironpress_paints_the_apex_below_the_overbar() {
        let sign = Sign::sample();
        let error = sign.correct_apex() - sign.painted_apex();
        assert!(
            (error - (sign.ascent + sign.thickness)).abs() < 0.01,
            "expected the painted apex to be off by ascent + thickness, got {error}"
        );
    }

    #[test]
    /// Assert the apex is lifted onto the overbar that follows the path.
    fn apex_is_lifted_onto_the_overbar() {
        let sign = Sign::sample();
        let out = correct_radicals(&sign.stream(sign.painted_apex()));
        let apex = apex_of(&out);
        assert!(
            (apex - sign.correct_apex()).abs() < 0.01,
            "expected apex {}, got {apex}",
            sign.correct_apex()
        );
    }

    #[test]
    /// Assert only the height changes: the shape and width are preserved.
    fn shape_and_width_are_preserved() {
        let sign = Sign::sample();
        let before = sign.stream(sign.painted_apex());
        let after = correct_radicals(&before);
        let (a, b) = (
            parse_radical_path(&before, first_point_at(&before)).expect("before"),
            parse_radical_path(&after, first_point_at(&after)).expect("after"),
        );
        let (before_height, after_height) = (a.apex.1 - a.foot.1, b.apex.1 - b.foot.1);
        assert!(
            (before_height - after_height).abs() < 0.01,
            "{before_height} {after_height}"
        );
        assert_eq!(a.tick.0, b.tick.0);
        assert_eq!(a.foot.0, b.foot.0);
        assert_eq!(a.apex.0, b.apex.0);
    }

    /// Return the index at which the radical path starts in `content`.
    fn first_point_at(content: &[u8]) -> usize {
        let prefix = b"0 0 0 RG\n";
        let at = content
            .windows(prefix.len())
            .position(|w| w == prefix)
            .expect("stroke color");
        at + prefix.len()
    }

    #[test]
    /// Assert a path that already meets its overbar is left untouched.
    ///
    /// The pass has to become a no-op, not a second displacement, when
    /// ironpress starts painting the sign at the right height.
    fn correctly_placed_path_is_unchanged() {
        let sign = Sign::sample();
        let stream = sign.stream(sign.correct_apex());
        assert_eq!(correct_radicals(&stream), stream);
    }

    #[test]
    /// Assert correcting a stream twice equals correcting it once.
    fn correction_is_idempotent() {
        let sign = Sign::sample();
        let once = correct_radicals(&sign.stream(sign.painted_apex()));
        assert_eq!(correct_radicals(&once), once);
    }

    #[test]
    /// Assert a three-point path that is not a radical sign is left alone.
    fn other_three_point_paths_are_left_alone() {
        let stream =
            b"0.44 w\n0 0 0 RG\n10 20 m\n30 20 l\n30 40 l\nS\n0 0 0 rg\n30 40 10 0.44 re\nf\n";
        assert_eq!(correct_radicals(stream), stream.to_vec());
    }

    #[test]
    /// Assert a radical without a following overbar is left alone.
    ///
    /// Without the bar there is no height to place the sign against.
    fn radical_without_an_overbar_is_left_alone() {
        let sign = Sign::sample();
        let stream = sign.stream(sign.painted_apex());
        let path_only: Vec<u8> = stream
            .windows(2)
            .position(|w| w == b"S\n")
            .map(|at| stream[..at + 2].to_vec())
            .expect("stroke operator");
        assert_eq!(correct_radicals(&path_only), path_only);
    }

    #[test]
    /// Assert text operators around a radical are copied unchanged.
    fn surrounding_operators_are_preserved() {
        let sign = Sign::sample();
        let mut stream = b"BT\n/Helvetica 11 Tf\n(x) Tj\nET\n".to_vec();
        stream.extend_from_slice(&sign.stream(sign.painted_apex()));
        let out = correct_radicals(&stream);
        let text = String::from_utf8_lossy(&out).to_string();
        assert!(
            text.starts_with("BT\n/Helvetica 11 Tf\n(x) Tj\nET\n"),
            "{text}"
        );
        assert!(text.contains("re\nf\n"), "{text}");
    }

    #[test]
    /// Assert a stream without any path is returned unchanged.
    fn stream_without_paths_is_unchanged() {
        let stream = b"BT\n/Symbol 11 Tf\n(\\326) Tj\nET\n";
        assert_eq!(correct_radicals(stream), stream.to_vec());
    }

    /// Return the PDF of a display `\sqrt{b^2-4ac}`, with this pass applied.
    fn exported_pdf() -> Vec<u8> {
        super::super::html_to_pdf_bytes(&sqrt_document()).expect("PDF conversion")
    }

    /// Return an export document holding one display radical.
    fn sqrt_document() -> String {
        crate::constants::EXPORT_PDF_TEMPLATE
            .replace("{{TITLE}}", "t")
            .replace(
                "{{BODY}}",
                r#"<div class="math-display" data-math="\sqrt{b^2-4ac}">sqrt</div>"#,
            )
    }

    /// Return the apex y and the overbar of the first radical in `pdf`.
    fn first_radical(pdf: &[u8]) -> (f32, super::Overbar) {
        (0..pdf.len())
            .filter(|i| super::is_operator_start(pdf, *i))
            .find_map(|i| {
                let path = super::parse_radical_path(pdf, i)?;
                let overbar = super::parse_overbar(pdf, path.end)?;
                Some((path.apex.1, overbar))
            })
            .expect("radical path and overbar in the PDF")
    }

    #[test]
    /// Assert an exported radical sign meets the overbar of its radicand.
    fn exported_radical_meets_its_overbar() {
        let (apex, overbar) = first_radical(&exported_pdf());
        let expected = overbar.center_y + overbar.thickness;
        assert!(
            (apex - expected).abs() < 0.01,
            "expected apex {expected}, got {apex}"
        );
    }

    #[test]
    /// Assert ironpress itself still paints the sign below the overbar.
    ///
    /// This test fails when ironpress starts placing the sign correctly, which
    /// is the point at which this module and its call in `super` are deleted.
    fn ironpress_still_misplaces_the_radical_sign() {
        let pdf = ironpress::HtmlConverter::new()
            .compress(false)
            .convert(&sqrt_document())
            .expect("PDF conversion");
        let (apex, overbar) = first_radical(&pdf);
        assert!(
            apex < overbar.center_y,
            "ironpress paints the apex at {apex}, at or above the overbar \
             center {}: delete this module",
            overbar.center_y
        );
    }
}
