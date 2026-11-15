use crate::components::editor::caret_placement::{rendered_y_of, CaretPlacement};
use crate::constants::{BLOCK_RENDER_SELECTOR, CARET_MARKER_TEXT, TEXTAREA_MIRROR_STYLE};
use crate::rendering::{source_offset_at_textarea, textarea_text_before};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, FocusOptions, HtmlElement, HtmlTextAreaElement};

/// Return the `.editor-area` element from the document.
pub(crate) fn editor_area() -> Option<HtmlElement> {
    web_sys::window()?
        .document()?
        .query_selector(".editor-area")
        .ok()??
        .dyn_into::<HtmlElement>()
        .ok()
}

/// Return the `.editor-area` ancestor of `el`.
pub(crate) fn editor_area_for(el: &HtmlTextAreaElement) -> Option<HtmlElement> {
    el.closest(".editor-area")
        .ok()
        .flatten()
        .and_then(|node| node.dyn_into::<HtmlElement>().ok())
}

/// Grow or shrink a textarea to fit its content, optionally restoring scroll.
pub(crate) fn autosize_textarea(el: &HtmlTextAreaElement, preserve_scroll: Option<i32>) {
    let scroll_top = preserve_scroll.or_else(|| editor_area_for(el).map(|a| a.scroll_top()));

    let style = HtmlElement::style(el);
    let _ = style.set_property("height", "auto");
    let h = el.scroll_height();
    let _ = style.set_property("height", &format!("{h}px"));

    if let (Some(area), Some(top)) = (editor_area_for(el), scroll_top) {
        area.set_scroll_top(top);
        request_animation_frame(move || {
            area.set_scroll_top(top);
        });
    }
}

/// Focus a textarea without scrolling the page.
pub(crate) fn focus_textarea(el: &HtmlTextAreaElement) {
    let opts = FocusOptions::new();
    opts.set_prevent_scroll(true);
    let _ = HtmlElement::focus_with_options(el, &opts);
}

/// Set the textarea value when it differs from `value`.
pub(crate) fn sync_textarea_value(el: &HtmlTextAreaElement, value: &str) {
    if el.value() != value {
        el.set_value(value);
    }
}

/// Put the textarea caret at `placement` in `src`.
///
/// Return the editor scroll offset that shows the caret line at the height
/// the user clicked, for [`autosize_textarea`] to apply.
pub(crate) fn place_caret(
    el: &HtmlTextAreaElement,
    src: &str,
    placement: CaretPlacement,
) -> Option<i32> {
    let text_before = textarea_text_before(src, placement.source_offset);
    let caret = text_before.encode_utf16().count() as u32;
    let _ = el.set_selection_range(caret, caret);

    let area = editor_area_for(el)?;
    let caret_y = el.get_bounding_client_rect().top() + caret_line_center(el, &text_before)?;
    let scroll_top = f64::from(area.scroll_top()) + caret_y - placement.client_y;
    Some(scroll_top.round().max(0.0) as i32)
}

/// Return the y of the caret line's center, from the top edge of the textarea.
///
/// A textarea does not report where its text wraps, so `text_before` is laid
/// out in a hidden block with the textarea's class and width, and a marker
/// after it is measured. The block is removed before returning.
fn caret_line_center(el: &HtmlTextAreaElement, text_before: &str) -> Option<f64> {
    let document = el.owner_document()?;
    let parent = el.parent_node()?;
    let mirror = document
        .create_element("div")
        .ok()?
        .dyn_into::<HtmlElement>()
        .ok()?;
    let marker = document
        .create_element("span")
        .ok()?
        .dyn_into::<HtmlElement>()
        .ok()?;
    mirror.set_class_name(&el.class_name());
    let style = mirror.style();
    for (property, value) in TEXTAREA_MIRROR_STYLE {
        let _ = style.set_property(property, value);
    }
    let _ = style.set_property("width", &format!("{}px", el.offset_width()));
    mirror.set_text_content(Some(text_before));
    marker.set_text_content(Some(CARET_MARKER_TEXT));
    mirror.append_child(&marker).ok()?;
    parent.append_child(&mirror).ok()?;
    let center = mirror.client_top() + marker.offset_top() + marker.offset_height() / 2;
    mirror.remove();
    Some(f64::from(center))
}

/// A source line and the height it showed at when edit mode was left.
#[derive(Clone, Debug)]
pub(crate) struct ExitAnchor {
    /// Editor block that held the textarea and now holds the rendered HTML.
    block: Element,
    /// Byte offset of the anchored line in the markdown source.
    source_offset: usize,
    /// Viewport y the line's center had in the textarea.
    view_y: f64,
}

/// Return the line of `src` to keep in place when `el` leaves edit mode.
///
/// The caret line is used while it is on screen. If the user scrolled it out
/// of view, the line nearest the middle of the editor area is used instead.
pub(crate) fn exit_anchor(el: &HtmlTextAreaElement, src: &str) -> Option<ExitAnchor> {
    let area_rect = editor_area_for(el)?.get_bounding_client_rect();
    let block = el.parent_element()?;
    let top = el.get_bounding_client_rect().top();
    let line_y = |offset: usize| {
        caret_line_center(el, &textarea_text_before(src, offset)).map(|center| top + center)
    };

    let caret = el.selection_start().ok().flatten().unwrap_or(0);
    let mut source_offset = source_offset_at_textarea(src, caret);
    let mut view_y = line_y(source_offset)?;
    if view_y < area_rect.top() || view_y > area_rect.bottom() {
        let middle = (area_rect.top() + area_rect.bottom()) / 2.0;
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(src.match_indices('\n').map(|(at, _)| at + 1))
            .collect();
        // Last line that starts at or above the middle of the editor area.
        let (mut low, mut high) = (0, line_starts.len() - 1);
        while low < high {
            let probe = (low + high).div_ceil(2);
            if line_y(line_starts[probe])? <= middle {
                low = probe;
            } else {
                high = probe - 1;
            }
        }
        source_offset = line_starts[low];
        view_y = line_y(source_offset)?;
    }
    Some(ExitAnchor {
        block,
        source_offset,
        view_y,
    })
}

/// Scroll the editor so the anchored line of `src` shows in the rendered
/// HTML at the height it had in the textarea.
pub(crate) fn restore_exit_anchor(anchor: &ExitAnchor, src: &str) -> Option<()> {
    let rendered = anchor.block.query_selector(BLOCK_RENDER_SELECTOR).ok()??;
    let area = rendered
        .closest(".editor-area")
        .ok()??
        .dyn_into::<HtmlElement>()
        .ok()?;
    let rendered_y = rendered_y_of(&rendered, src, anchor.source_offset)?;
    let scroll_top = f64::from(area.scroll_top()) + rendered_y - anchor.view_y;
    area.set_scroll_top(scroll_top.round().max(0.0) as i32);
    Some(())
}
