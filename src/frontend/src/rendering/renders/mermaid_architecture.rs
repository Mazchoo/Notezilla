//! Fits architecture service labels inside their service boxes.
//!
//! rusty-mermaid draws each architecture service label on one line under the
//! service icon, so a long name runs past the left and right edges of its
//! fixed-width box. Those labels are wrapped, and shrunk if need be, to fit
//! the part of the box below the icon.

use super::mermaid_fit::{fit_label, push_element};
use crate::constants::{
    MERMAID_ARCHITECTURE_ICON_OFFSET as ICON_OFFSET,
    MERMAID_ARCHITECTURE_LABEL_OFFSET as LABEL_OFFSET, MERMAID_ARCHITECTURE_SERVICE_HEIGHT,
    MERMAID_ARCHITECTURE_SERVICE_WIDTH, MERMAID_LABEL_PADDING,
};
use rusty_mermaid::{BBox, Point, Primitive, Scene};
use rusty_mermaid_core::scene::Element;

/// Return `scene` with every service label wrapped to fit its service box.
///
/// A service is a service-sized rect followed by its icon circle, the icon
/// text, and the label. Anything that does not follow that shape is kept.
pub(super) fn fit_service_labels(scene: Scene) -> Scene {
    let elements = scene.elements();
    let mut fitted = Scene::new(scene.width, scene.height);
    for (index, element) in elements.iter().enumerate() {
        let label = index
            .checked_sub(LABEL_OFFSET)
            .and_then(|start| service_label(&elements[start..=index]));
        match label {
            Some(primitive) => fitted.push(primitive),
            None => push_element(&mut fitted, element),
        }
    }
    fitted
}

/// Return the fitted label when `service` is a service box through its label.
fn service_label(service: &[Element]) -> Option<Primitive> {
    let Primitive::Rect { bbox, .. } = &service[0].primitive else {
        return None;
    };
    if bbox.width != MERMAID_ARCHITECTURE_SERVICE_WIDTH
        || bbox.height != MERMAID_ARCHITECTURE_SERVICE_HEIGHT
    {
        return None;
    }
    let Primitive::Circle { center, radius, .. } = &service[ICON_OFFSET].primitive else {
        return None;
    };
    let Primitive::Text {
        content, style, ..
    } = &service[LABEL_OFFSET].primitive
    else {
        return None;
    };
    let (top, bottom) = label_band(bbox, center.y + radius);
    let fitted = fit_label(
        content,
        style,
        bbox.width - MERMAID_LABEL_PADDING,
        bottom - top - MERMAID_LABEL_PADDING,
    );
    Some(fitted.at(Point::new(bbox.x, (top + bottom) / 2.0)))
}

/// Return the top and bottom of the band below the icon inside `bbox`.
///
/// rusty-mermaid boxes are positioned by their centre.
fn label_band(bbox: &BBox, icon_bottom: f64) -> (f64, f64) {
    (icon_bottom, bbox.y + bbox.height / 2.0)
}

#[cfg(test)]
mod tests {
    use super::fit_service_labels;
    use crate::constants::{MERMAID_ARCHITECTURE_SERVICE_WIDTH, MERMAID_LABEL_PADDING};
    use rusty_mermaid::{render, Primitive, Theme};
    use rusty_mermaid_core::SimpleTextMeasure;

    /// Architecture diagram from the manual, with long service names.
    const ARCHITECTURE: &str = "architecture-beta\n    \
        group cloud(cloud)[AWS Virtual Private Cloud]\n    \
        service internet(internet)[Public Internet]\n    \
        service alb(server)[Application Load Balancer] in cloud\n    \
        service api(server)[ECS API Service] in cloud\n    \
        service db(database)[Aurora PostgreSQL] in cloud\n    \
        internet:R -- L:alb\n    alb:B -- T:api\n    api:R -- L:db\n";

    /// Return each service label line and its measured width.
    fn label_lines(source: &str) -> Vec<(String, f64)> {
        let scene = fit_service_labels(render(source, &Theme::dark()).unwrap());
        scene
            .elements()
            .iter()
            .filter_map(|element| match &element.primitive {
                Primitive::Text { content, style, .. } if content.len() > 3 => {
                    Some((content.clone(), style.clone()))
                }
                _ => None,
            })
            .flat_map(|(content, style)| {
                content
                    .lines()
                    .map(|line| {
                        let width = SimpleTextMeasure::measure_raw(line, &style).width;
                        (line.to_string(), width)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    #[test]
    /// Assert long service labels wrap inside the service box.
    fn long_service_labels_wrap_inside_box() {
        let lines = label_lines(ARCHITECTURE);
        let room = MERMAID_ARCHITECTURE_SERVICE_WIDTH - MERMAID_LABEL_PADDING;
        for name in ["Application", "Load Balancer", "PostgreSQL"] {
            assert!(
                lines.iter().any(|(line, _)| line == name),
                "{name} not on its own line: {lines:?}"
            );
        }
        for (line, width) in lines.iter().filter(|(line, _)| !line.starts_with("AWS")) {
            assert!(*width < room, "{line} is {width} wide: {lines:?}");
        }
    }

    #[test]
    /// Assert a short service label is left on one line.
    fn short_service_label_is_unchanged() {
        let lines = label_lines("architecture-beta\n    service db(database)[Database]\n");
        assert!(lines.iter().any(|(line, _)| line == "Database"), "{lines:?}");
    }
}
