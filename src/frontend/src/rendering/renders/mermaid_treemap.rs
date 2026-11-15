//! Restores treemap leaf labels rusty-mermaid drops for being too wide.
//!
//! rusty-mermaid draws a treemap leaf's name and value only when the name
//! fits on one line inside the leaf, so long names leave the leaf blank.
//! Those names are put back wrapped and shrunk, above their value, in every
//! leaf they fit; leaves too small for any label stay blank as before.

use super::mermaid_fit::{fit_label, label_room, line_height, push_element};
use crate::constants::{
    MERMAID_MIN_LABEL_FONT_SIZE, MERMAID_TREEMAP_FONT_RATIO, MERMAID_TREEMAP_LEAF_STROKE,
    MERMAID_TREEMAP_VALUE_SCALE,
};
use rusty_mermaid::{Point, Primitive, Scene, TextAnchor, TextStyle, Theme};
use rusty_mermaid_core::FontWeight;
use rusty_mermaid_diagrams::treemap::ir::TreemapNode;
use rusty_mermaid_diagrams::treemap::parser::parse;

/// Return `scene` with a fitted name and value in every blank treemap leaf.
///
/// Leaf boxes are matched in order to the leaves of `source`, depth first.
/// The scene is returned unchanged when `source` does not parse or the
/// boxes and leaves do not line up.
pub(super) fn label_treemap_leaves(scene: Scene, source: &str, theme: &Theme) -> Scene {
    let Ok(diagram) = parse(source) else {
        return scene;
    };
    let mut leaves = Vec::new();
    collect_leaves(&diagram.roots, &mut leaves);
    let elements = scene.elements();
    let boxes = elements
        .iter()
        .filter(|element| is_leaf_box(&element.primitive))
        .count();
    if boxes != leaves.len() {
        return scene;
    }
    let mut leaves = leaves.into_iter();
    let mut labelled = Scene::new(scene.width, scene.height);
    for (index, element) in elements.iter().enumerate() {
        push_element(&mut labelled, element);
        let Primitive::Rect { bbox, .. } = &element.primitive else {
            continue;
        };
        if !is_leaf_box(&element.primitive) {
            continue;
        }
        let Some(leaf) = leaves.next() else {
            continue;
        };
        let has_label = matches!(
            elements.get(index + 1).map(|next| &next.primitive),
            Some(Primitive::Text { .. })
        );
        if has_label || leaf.name.is_empty() {
            continue;
        }
        let font_size = (bbox.height * MERMAID_TREEMAP_FONT_RATIO)
            .clamp(MERMAID_MIN_LABEL_FONT_SIZE, theme.font_size_node);
        let name_style = TextStyle {
            font_size,
            fill: Some(theme.node_text),
            font_weight: FontWeight::Bold,
            ..Default::default()
        };
        let value_style = TextStyle {
            font_size: font_size * MERMAID_TREEMAP_VALUE_SCALE,
            fill: Some(theme.muted_text),
            ..Default::default()
        };
        let value_height = line_height(&value_style);
        let (width, height) = label_room(bbox);
        let name = fit_label(&leaf.name, &name_style, width, height - value_height);
        if !name.fits {
            continue;
        }
        // Centre the name and value together, name above value.
        let name_y = bbox.y - value_height / 2.0;
        let value_y = bbox.y + name.height() / 2.0;
        labelled.push(name.at(Point::new(bbox.x, name_y)));
        labelled.push(Primitive::Text {
            position: Point::new(bbox.x, value_y),
            content: format!("{:.0}", leaf.total_value()),
            anchor: TextAnchor::Middle,
            style: value_style,
        });
    }
    labelled
}

/// Push the leaves of `nodes` onto `leaves`, depth first.
fn collect_leaves<'a>(nodes: &'a [TreemapNode], leaves: &mut Vec<&'a TreemapNode>) {
    for node in nodes {
        if node.is_leaf() {
            leaves.push(node);
        } else {
            collect_leaves(&node.children, leaves);
        }
    }
}

/// Return whether `primitive` is a treemap leaf box rather than a section.
fn is_leaf_box(primitive: &Primitive) -> bool {
    matches!(
        primitive,
        Primitive::Rect { style, .. } if style.stroke_width == Some(MERMAID_TREEMAP_LEAF_STROKE)
    )
}

#[cfg(test)]
mod tests {
    use super::label_treemap_leaves;
    use rusty_mermaid::{render, Primitive, Theme};

    /// Treemap with one short and one long leaf name.
    const TREEMAP: &str = "treemap-beta\n\"Products\"\n    \"Short\": 40\n    \
                           \"Enterprise Analytics Platform Suite\": 30\n    \"Other\": 30\n";

    /// Return every text label in the labelled treemap scene.
    fn labels(source: &str) -> Vec<String> {
        let theme = Theme::dark();
        let scene = render(source, &theme).unwrap();
        label_treemap_leaves(scene, source, &theme)
            .elements()
            .iter()
            .filter_map(|element| match &element.primitive {
                Primitive::Text { content, .. } => Some(content.replace('\n', " ")),
                _ => None,
            })
            .collect()
    }

    #[test]
    /// Assert a long leaf name is restored with its value, each once.
    fn long_leaf_name_is_restored() {
        let labels = labels(TREEMAP);
        let name = "Enterprise Analytics Platform Suite";
        assert_eq!(
            labels.iter().filter(|l| *l == name).count(),
            1,
            "{labels:?}"
        );
        assert_eq!(
            labels.iter().filter(|l| *l == "Short").count(),
            1,
            "{labels:?}"
        );
        assert_eq!(
            labels.iter().filter(|l| *l == "30").count(),
            2,
            "{labels:?}"
        );
    }
}
