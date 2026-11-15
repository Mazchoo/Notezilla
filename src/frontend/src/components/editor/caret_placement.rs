//! Resolves a click on rendered markdown to a caret position in its source.

use crate::constants::{
    CARET_FROM_POINT_METHODS, SOURCE_KIND_ATTR, SOURCE_MAPPED_SELECTOR, SOURCE_OFFSET_ATTR,
};
use crate::rendering::{resolve_source_offset, SourceKind};
use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element, MouseEvent, Node};

/// Where the textarea caret goes when a rendered block enters edit mode.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CaretPlacement {
    /// Byte offset of the caret in the markdown source.
    pub source_offset: usize,
    /// Viewport y of the click; the caret line is scrolled to this height.
    pub client_y: f64,
}

/// A rendered element tagged with the source offset it came from.
struct MappedElement {
    element: Element,
    offset: usize,
    kind: SourceKind,
}

impl MappedElement {
    /// Return the tagged element at or above `start`, if it is inside `root`.
    fn containing(start: &Element, root: &Element) -> Option<Self> {
        let element = start.closest(SOURCE_MAPPED_SELECTOR).ok()??;
        if !root.contains(Some(&element)) {
            return None;
        }
        Self::from_element(element)
    }

    /// Read the source tag of `element`.
    fn from_element(element: Element) -> Option<Self> {
        let offset = element.get_attribute(SOURCE_OFFSET_ATTR)?.parse().ok()?;
        let kind = SourceKind::from_attr(element.get_attribute(SOURCE_KIND_ATTR).as_deref());
        Some(Self {
            element,
            offset,
            kind,
        })
    }
}

/// Return the caret placement for a click on the rendered markdown of `src`.
///
/// The event's current target is the block that holds the rendered HTML. The
/// caret under the pointer is tried first, then the clicked element, then
/// the tagged element nearest the pointer.
pub(crate) fn caret_placement_at(ev: &MouseEvent, src: &str) -> Option<CaretPlacement> {
    let root = ev.current_target()?.dyn_into::<Element>().ok()?;
    let x = f64::from(ev.client_x());
    let y = f64::from(ev.client_y());
    let source_offset = caret_source_offset(&root, x, y, src)
        .or_else(|| target_source_offset(ev, &root))
        .or_else(|| nearest_source_offset(&root, x, y))?;
    Some(CaretPlacement {
        source_offset,
        client_y: y,
    })
}

/// Return the source offset of the text caret the browser places at `(x, y)`.
fn caret_source_offset(root: &Element, x: f64, y: f64, src: &str) -> Option<usize> {
    let document = root.owner_document()?;
    let (node, offset) = caret_from_point(&document, x, y)?;
    if !root.contains(Some(&node)) {
        return None;
    }
    let start = match node.dyn_ref::<Element>() {
        Some(element) => element.clone(),
        None => node.parent_element()?,
    };
    let mapped = MappedElement::containing(&start, root)?;
    let mut prefix = String::new();
    if !text_before_caret(&mapped.element, &node, offset, &mut prefix) {
        return None;
    }
    let full_text = match mapped.kind {
        SourceKind::Lines => mapped.element.text_content().unwrap_or_default(),
        SourceKind::Verbatim | SourceKind::Atom => String::new(),
    };
    Some(resolve_source_offset(
        src,
        mapped.offset,
        mapped.kind,
        &full_text,
        &prefix,
    ))
}

/// Return the caret node and offset under `(x, y)` from whichever caret
/// lookup the browser provides.
fn caret_from_point(document: &Document, x: f64, y: f64) -> Option<(Node, u32)> {
    CARET_FROM_POINT_METHODS
        .iter()
        .find_map(|(method, node_key, offset_key)| {
            let lookup = Reflect::get(document, &JsValue::from_str(method))
                .ok()?
                .dyn_into::<Function>()
                .ok()?;
            let caret = lookup
                .call2(document, &JsValue::from_f64(x), &JsValue::from_f64(y))
                .ok()?;
            let node = Reflect::get(&caret, &JsValue::from_str(node_key))
                .ok()?
                .dyn_into::<Node>()
                .ok()?;
            let offset = Reflect::get(&caret, &JsValue::from_str(offset_key))
                .ok()?
                .as_f64()?;
            Some((node, offset as u32))
        })
}

/// Append to `out` the text of `current` that precedes the caret.
///
/// The caret is `offset` UTF-16 units into `caret` when it is a text node,
/// else `offset` children into it. Return whether the caret was reached.
fn text_before_caret(current: &Node, caret: &Node, offset: u32, out: &mut String) -> bool {
    let is_text = current.node_type() == Node::TEXT_NODE;
    if current.is_same_node(Some(caret)) {
        if is_text {
            let text = current.text_content().unwrap_or_default();
            let units: Vec<u16> = text.encode_utf16().take(offset as usize).collect();
            out.push_str(&String::from_utf16_lossy(&units));
        } else {
            let mut child = current.first_child();
            for _ in 0..offset {
                let Some(node) = child else { break };
                out.push_str(&node.text_content().unwrap_or_default());
                child = node.next_sibling();
            }
        }
        return true;
    }
    if is_text {
        out.push_str(&current.text_content().unwrap_or_default());
        return false;
    }
    let mut child = current.first_child();
    while let Some(node) = child {
        if text_before_caret(&node, caret, offset, out) {
            return true;
        }
        child = node.next_sibling();
    }
    false
}

/// Return the source offset of the tagged element the click landed in.
fn target_source_offset(ev: &MouseEvent, root: &Element) -> Option<usize> {
    let target = ev.target()?.dyn_into::<Node>().ok()?;
    let start = match target.dyn_ref::<Element>() {
        Some(element) => element.clone(),
        None => target.parent_element()?,
    };
    MappedElement::containing(&start, root).map(|mapped| mapped.offset)
}

/// Return the source offset of the tagged element nearest `(x, y)`.
///
/// Vertical distance decides; horizontal distance breaks ties on a line.
fn nearest_source_offset(root: &Element, x: f64, y: f64) -> Option<usize> {
    let candidates = root.query_selector_all(SOURCE_MAPPED_SELECTOR).ok()?;
    let mut best: Option<((f64, f64), usize)> = None;
    for index in 0..candidates.length() {
        let Some(mapped) = candidates
            .item(index)
            .and_then(|node| node.dyn_into::<Element>().ok())
            .and_then(MappedElement::from_element)
        else {
            continue;
        };
        let rect = mapped.element.get_bounding_client_rect();
        let distance = (
            distance_outside(y, rect.top(), rect.bottom()),
            distance_outside(x, rect.left(), rect.right()),
        );
        if best.is_none_or(|(closest, _)| distance < closest) {
            best = Some((distance, mapped.offset));
        }
    }
    best.map(|(_, offset)| offset)
}

/// Return how far `value` lies outside `low..=high`; zero when inside.
fn distance_outside(value: f64, low: f64, high: f64) -> f64 {
    if value < low {
        low - value
    } else if value > high {
        value - high
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::distance_outside;

    #[test]
    /// Assert distance is zero inside the span and grows outside either end.
    fn distance_outside_measures_from_the_nearest_edge() {
        assert_eq!(distance_outside(5.0, 0.0, 10.0), 0.0);
        assert_eq!(distance_outside(-3.0, 0.0, 10.0), 3.0);
        assert_eq!(distance_outside(14.0, 0.0, 10.0), 4.0);
    }
}
