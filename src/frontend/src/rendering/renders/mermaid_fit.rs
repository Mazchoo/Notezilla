//! Fits labels rusty-mermaid drops for being too wide back into their boxes.
//!
//! Several rusty-mermaid diagrams draw a label only when it fits on one line
//! inside its box and silently leave longer ones out. Restored labels are
//! wrapped onto as many lines as they need and shrunk until they fit.

use crate::constants::{
    MERMAID_LABEL_FONT_STEP, MERMAID_LABEL_PADDING, MERMAID_MIN_LABEL_FONT_SIZE,
};
use rusty_mermaid::{BBox, Point, Primitive, Scene, TextAnchor, TextStyle};
use rusty_mermaid_core::constants::LINE_HEIGHT_MULTIPLIER;
use rusty_mermaid_core::scene::Element;
use rusty_mermaid_core::SimpleTextMeasure;

/// A label wrapped and sized for a box.
pub(super) struct FittedLabel {
    /// Label lines joined by `\n`.
    pub text: String,
    /// Style with the chosen font size.
    pub style: TextStyle,
    /// Whether the label fits the box without splitting a word.
    pub fits: bool,
}

impl FittedLabel {
    /// Return the height the label's lines take up.
    pub fn height(&self) -> f64 {
        line_height(&self.style) * self.text.lines().count().max(1) as f64
    }

    /// Return the label as a text primitive centred on `position`.
    pub fn at(self, position: Point) -> Primitive {
        Primitive::Text {
            position,
            content: self.text,
            anchor: TextAnchor::Middle,
            style: self.style,
        }
    }
}

/// Return `text` fitted to a `width` by `height` box, starting from `style`.
///
/// The font shrinks from `style`'s size down to the minimum until the
/// wrapped lines fit `height` without a word being split. At the minimum
/// size the label is returned anyway, marked as not fitting.
pub(super) fn fit_label(text: &str, style: &TextStyle, width: f64, height: f64) -> FittedLabel {
    let mut style = style.clone();
    loop {
        let wrapped = wrap(text, &style, width);
        let fits = widest_word(text, &style) < width
            && line_height(&style) * wrapped.lines().count().max(1) as f64 <= height;
        if fits || style.font_size - MERMAID_LABEL_FONT_STEP < MERMAID_MIN_LABEL_FONT_SIZE {
            return FittedLabel {
                text: wrapped,
                style,
                fits,
            };
        }
        style.font_size -= MERMAID_LABEL_FONT_STEP;
    }
}

/// Return the room a label has inside `bbox` once padding is taken off.
pub(super) fn label_room(bbox: &BBox) -> (f64, f64) {
    (
        bbox.width - MERMAID_LABEL_PADDING,
        bbox.height - MERMAID_LABEL_PADDING,
    )
}

/// Return the centre-based bounding box of a filled shape primitive.
///
/// Returns `None` for primitives that are not shapes a label sits in.
pub(super) fn shape_bounds(primitive: &Primitive) -> Option<BBox> {
    match primitive {
        Primitive::Rect { bbox, .. } => Some(*bbox),
        Primitive::Circle { center, radius, .. } => {
            Some(BBox::new(center.x, center.y, radius * 2.0, radius * 2.0))
        }
        Primitive::Polygon { points, .. } => {
            let (min_x, max_x, min_y, max_y) = points.iter().fold(
                (
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                ),
                |(min_x, max_x, min_y, max_y), p| {
                    (
                        min_x.min(p.x),
                        max_x.max(p.x),
                        min_y.min(p.y),
                        max_y.max(p.y),
                    )
                },
            );
            (min_x <= max_x).then(|| {
                BBox::new(
                    (min_x + max_x) / 2.0,
                    (min_y + max_y) / 2.0,
                    max_x - min_x,
                    max_y - min_y,
                )
            })
        }
        _ => None,
    }
}

/// Push a copy of `element` onto `scene`, keeping its id.
pub(super) fn push_element(scene: &mut Scene, element: &Element) {
    match &element.id {
        Some(id) => scene.push_identified(element.primitive.clone(), id.clone()),
        None => scene.push(element.primitive.clone()),
    }
}

/// Return the height of one line of text in `style`.
pub(super) fn line_height(style: &TextStyle) -> f64 {
    style.font_size * LINE_HEIGHT_MULTIPLIER
}

/// Return the width rusty-mermaid measures for `text` in `style`.
fn text_width(text: &str, style: &TextStyle) -> f64 {
    SimpleTextMeasure::measure_raw(text, style).width
}

/// Return the width of the widest word of `text` in `style`.
fn widest_word(text: &str, style: &TextStyle) -> f64 {
    text.split_whitespace()
        .map(|word| text_width(word, style))
        .fold(0.0, f64::max)
}

/// Break `text` into `\n`-separated lines no wider than `max_width`.
///
/// Lines break between words; a word wider than `max_width` on its own is
/// split between characters.
pub(super) fn wrap(text: &str, style: &TextStyle, max_width: f64) -> String {
    let fits = |line: &str| text_width(line, style) < max_width;
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        let candidate_len = line.len() + 1 + word.len();
        if !line.is_empty() {
            let mut candidate = String::with_capacity(candidate_len);
            candidate.push_str(&line);
            candidate.push(' ');
            candidate.push_str(word);
            if fits(&candidate) {
                line = candidate;
                continue;
            }
            lines.push(std::mem::take(&mut line));
        }
        for ch in word.chars() {
            line.push(ch);
            if !fits(&line) && line.chars().count() > 1 {
                line.pop();
                lines.push(std::mem::take(&mut line));
                line.push(ch);
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{fit_label, wrap};
    use rusty_mermaid::TextStyle;

    #[test]
    /// Assert a word wider than the box is split between characters.
    fn overlong_word_is_split() {
        let wrapped = wrap(&"x".repeat(60), &TextStyle::default(), 112.0);
        assert!(wrapped.lines().count() > 1, "{wrapped}");
        assert_eq!(wrapped.replace('\n', ""), "x".repeat(60));
    }

    #[test]
    /// Assert a word too wide at full size shrinks rather than splitting.
    fn short_word_shrinks_to_fit_narrow_box() {
        let fitted = fit_label("SYN", &TextStyle::default(), 23.0, 24.0);
        assert!(fitted.fits);
        assert_eq!(fitted.text, "SYN");
        assert!(fitted.style.font_size < TextStyle::default().font_size);
    }

    #[test]
    /// Assert a label too long for a tiny box is marked as not fitting.
    fn tiny_box_does_not_fit() {
        let fitted = fit_label(
            "A very long label indeed",
            &TextStyle::default(),
            10.0,
            10.0,
        );
        assert!(!fitted.fits);
    }
}
