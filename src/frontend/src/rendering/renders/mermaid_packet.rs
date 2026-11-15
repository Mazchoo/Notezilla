//! Restores packet field labels rusty-mermaid drops for being too wide.
//!
//! rusty-mermaid draws a packet field label only when it fits on one line
//! inside the field's box, so narrow fields such as single-bit flags lose
//! their names. Those labels are put back, shrunk and wrapped to fit.

use super::mermaid_fit::{fit_label, label_room, push_element};
use rusty_mermaid::{Point, Primitive, Scene, TextAnchor, TextStyle, Theme};
use rusty_mermaid_core::scene::Element;
use rusty_mermaid_diagrams::packet::parser::parse;
use std::iter::repeat_n;

/// Return `scene` with a fitted label in every packet field box left empty.
///
/// Field boxes are matched in order to the fields of `source`, split at row
/// ends the way rusty-mermaid splits them. The scene is returned unchanged
/// when `source` does not parse or the boxes and fields do not line up.
pub(super) fn label_packet_fields(scene: Scene, source: &str, theme: &Theme) -> Scene {
    let Ok(diagram) = parse(source) else {
        return scene;
    };
    let row_bits = diagram.bits_per_row.max(1);
    let mut labels = diagram.fields.iter().flat_map(|field| {
        // rusty-mermaid draws no box for a field that ends before it starts.
        let rows = match field.end.checked_sub(field.start) {
            Some(_) => field.end / row_bits - field.start / row_bits + 1,
            None => 0,
        };
        repeat_n(field.label.as_str(), rows)
    });
    let elements = scene.elements();
    let boxes = elements
        .iter()
        .filter(|element| matches!(element.primitive, Primitive::Rect { .. }))
        .count();
    if boxes != labels.clone().count() {
        return scene;
    }
    let style = TextStyle {
        font_size: theme.font_size_node,
        fill: Some(theme.node_text),
        ..Default::default()
    };
    let mut labelled = Scene::new(scene.width, scene.height);
    for (index, element) in elements.iter().enumerate() {
        push_element(&mut labelled, element);
        let Primitive::Rect { bbox, .. } = &element.primitive else {
            continue;
        };
        let Some(label) = labels.next() else {
            continue;
        };
        if label.is_empty() || has_field_label(elements.get(index + 1), &style) {
            continue;
        }
        let (width, height) = label_room(bbox);
        let fitted = fit_label(label, &style, width, height);
        labelled.push(fitted.at(Point::new(bbox.x, bbox.y)));
    }
    labelled
}

/// Return whether `next` is a field label rather than a bit number.
///
/// Field labels are centred in the node font; bit numbers use a smaller one.
fn has_field_label(next: Option<&Element>, label: &TextStyle) -> bool {
    matches!(
        next.map(|element| &element.primitive),
        Some(Primitive::Text {
            anchor: TextAnchor::Middle,
            style,
            ..
        }) if style.font_size == label.font_size
    )
}

#[cfg(test)]
mod tests {
    use super::label_packet_fields;
    use rusty_mermaid::{render, Primitive, Theme};

    /// Packet with a wide field and narrow single-bit flags.
    const PACKET: &str = "packet-beta\n0-15: \"Source Port\"\n16-31: \"Destination Port\"\n\
                          32-39: \"Data offset\"\n40: \"SYN\"\n41: \"ACK\"\n42-63: \"Window\"\n";

    /// Return every centred text label in the labelled packet scene.
    fn labels(source: &str) -> Vec<String> {
        let theme = Theme::dark();
        let scene = render(source, &theme).unwrap();
        label_packet_fields(scene, source, &theme)
            .elements()
            .iter()
            .filter_map(|element| match &element.primitive {
                Primitive::Text { content, .. } => Some(content.replace('\n', " ")),
                _ => None,
            })
            .collect()
    }

    #[test]
    /// Assert narrow field labels are restored, each once.
    fn narrow_field_labels_are_restored() {
        let labels = labels(PACKET);
        for name in [
            "Source Port",
            "Destination Port",
            "Data offset",
            "SYN",
            "ACK",
            "Window",
        ] {
            assert_eq!(
                labels.iter().filter(|l| *l == name).count(),
                1,
                "{name}: {labels:?}"
            );
        }
    }
}
