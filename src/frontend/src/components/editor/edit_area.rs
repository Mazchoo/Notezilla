use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{FocusOptions, HtmlElement, HtmlTextAreaElement};

pub(crate) fn editor_area() -> Option<HtmlElement> {
    web_sys::window()?
        .document()?
        .query_selector(".editor-area")
        .ok()??
        .dyn_into::<HtmlElement>()
        .ok()
}

pub(crate) fn editor_area_for(el: &HtmlTextAreaElement) -> Option<HtmlElement> {
    el.closest(".editor-area")
        .ok()
        .flatten()
        .and_then(|node| node.dyn_into::<HtmlElement>().ok())
}

/// Grow or shrink a textarea to fit its content. When `preserve_scroll` is set
/// (entering edit mode), restore that position instead of the post-layout value.
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

pub(crate) fn focus_textarea(el: &HtmlTextAreaElement) {
    let opts = FocusOptions::new();
    opts.set_prevent_scroll(true);
    let _ = HtmlElement::focus_with_options(el, &opts);
}

pub(crate) fn sync_textarea_value(el: &HtmlTextAreaElement, value: &str) {
    if el.value() != value {
        el.set_value(value);
    }
}
