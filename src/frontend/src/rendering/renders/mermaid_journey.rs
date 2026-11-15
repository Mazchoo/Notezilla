//! Restores journey task labels rusty-mermaid drops for being too wide.
//!
//! rusty-mermaid draws a journey task label only when it fits on one line
//! inside the fixed-width task box; longer labels are silently left out.
//! Those labels are put back, wrapped onto as many lines as they need.

use super::mermaid_fit::{push_element, wrap};
use crate::constants::{
    MERMAID_JOURNEY_LABEL_PADDING, MERMAID_JOURNEY_TASK_HEIGHT, MERMAID_JOURNEY_TASK_WIDTH,
};
use rusty_mermaid::{BBox, Primitive, Scene, TextAnchor, TextStyle, Theme};
use rusty_mermaid_diagrams::journey::parser::parse;

/// Return `scene` with a wrapped label in every journey task box left empty.
///
/// Task boxes are matched to the tasks of `source` in order. The scene is
/// returned unchanged when `source` does not parse or the boxes and tasks
/// do not line up.
pub(super) fn label_journey_tasks(scene: Scene, source: &str, theme: &Theme) -> Scene {
    let Ok(diagram) = parse(source) else {
        return scene;
    };
    let mut names = diagram
        .sections
        .iter()
        .flat_map(|section| &section.tasks)
        .map(|task| task.name.as_str());
    let elements = scene.elements();
    let boxes = elements
        .iter()
        .filter(|element| is_task_box(&element.primitive))
        .count();
    if boxes != names.clone().count() {
        return scene;
    }
    let style = TextStyle {
        font_size: theme.font_size_small,
        fill: Some(theme.node_text),
        ..Default::default()
    };
    let mut labelled = Scene::new(scene.width, scene.height);
    for (index, element) in elements.iter().enumerate() {
        push_element(&mut labelled, element);
        let Primitive::Rect { bbox, .. } = &element.primitive else {
            continue;
        };
        if !is_task_box(&element.primitive) {
            continue;
        }
        let Some(name) = names.next() else {
            continue;
        };
        let has_label = matches!(
            elements.get(index + 1).map(|next| &next.primitive),
            Some(Primitive::Text { .. })
        );
        if !has_label {
            labelled.push(task_label(bbox, name, &style));
        }
    }
    labelled
}

/// Return whether `primitive` is a journey task box: an outlined task-sized rect.
fn is_task_box(primitive: &Primitive) -> bool {
    matches!(
        primitive,
        Primitive::Rect { bbox, style, .. }
            if style.stroke.is_some()
                && bbox.width == MERMAID_JOURNEY_TASK_WIDTH
                && bbox.height == MERMAID_JOURNEY_TASK_HEIGHT
    )
}

/// Return `name` wrapped to fit the task box `bbox`, centred in it.
///
/// rusty-mermaid task boxes are positioned by their centre.
fn task_label(bbox: &BBox, name: &str, style: &TextStyle) -> Primitive {
    Primitive::Text {
        position: rusty_mermaid::Point::new(bbox.x, bbox.y),
        content: wrap(name, style, bbox.width - MERMAID_JOURNEY_LABEL_PADDING),
        anchor: TextAnchor::Middle,
        style: style.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::label_journey_tasks;
    use rusty_mermaid::{render, Primitive, Theme};

    /// Journey with one short and two long task labels.
    const JOURNEY: &str = "journey\n    section Discovery\n      Short: 4: Me\n      \
                           Search product keyword: 3: Me\n      \
                           Filter by price and rating: 5: Me\n";

    /// Return every text label in the labelled journey scene.
    fn labels(source: &str) -> Vec<String> {
        let theme = Theme::dark();
        let scene = render(source, &theme).unwrap();
        label_journey_tasks(scene, source, &theme)
            .elements()
            .iter()
            .filter_map(|element| match &element.primitive {
                Primitive::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    /// Assert long task labels are restored on several lines.
    fn long_task_labels_are_wrapped() {
        let labels = labels(JOURNEY);
        assert!(labels.contains(&"Short".to_string()), "{labels:?}");
        assert!(
            labels.contains(&"Search product\nkeyword".to_string()),
            "{labels:?}"
        );
        assert!(
            labels
                .iter()
                .any(|l| l.starts_with("Filter by") && l.contains('\n')),
            "{labels:?}"
        );
    }

    #[test]
    /// Assert labels rusty-mermaid already drew are not duplicated.
    fn short_task_label_is_not_duplicated() {
        let labels = labels(JOURNEY);
        assert_eq!(
            labels.iter().filter(|l| *l == "Short").count(),
            1,
            "{labels:?}"
        );
    }
}
