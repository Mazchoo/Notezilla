//! Handles clicks on links inside rendered markdown.

use crate::components::editor::edit_area::editor_area;
use crate::constants::{ANCHORED_HEADING_SELECTOR, LINK_HREF_ATTR, LINK_SELECTOR};
use wasm_bindgen::JsCast;
use web_sys::{Element, MouseEvent};

/// Return the link a click landed in, if any.
pub(crate) fn clicked_link(ev: &MouseEvent) -> Option<Element> {
    ev.target()?
        .dyn_into::<Element>()
        .ok()?
        .closest(LINK_SELECTOR)
        .ok()?
}

/// Return the decoded `#fragment` of an in-document link.
pub(crate) fn link_fragment(link: &Element) -> Option<String> {
    let href = link.get_attribute(LINK_HREF_ATTR)?;
    let fragment = href.strip_prefix('#')?;
    Some(
        js_sys::decode_uri_component(fragment)
            .map(String::from)
            .unwrap_or_else(|_| fragment.to_string()),
    )
}

/// Scroll the heading with id `fragment` into view.
///
/// The block holding the clicked link (the event's current target) is
/// searched first, then every open note, so a note's own heading wins over a
/// same-named heading in another note.
pub(crate) fn scroll_to_heading(ev: &MouseEvent, fragment: &str) {
    let block = ev
        .current_target()
        .and_then(|target| target.dyn_into::<Element>().ok());
    let area = editor_area().map(Element::from);
    if let Some(heading) = [block, area]
        .into_iter()
        .flatten()
        .find_map(|root| heading_with_id(&root, fragment))
    {
        heading.scroll_into_view();
    }
}

/// Return the heading inside `root` whose id is `id`.
fn heading_with_id(root: &Element, id: &str) -> Option<Element> {
    let headings = root.query_selector_all(ANCHORED_HEADING_SELECTOR).ok()?;
    (0..headings.length())
        .filter_map(|index| headings.item(index)?.dyn_into::<Element>().ok())
        .find(|heading| heading.id() == id)
}
