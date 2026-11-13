use crate::info_messages::{APP_STATE_MISSING_PANIC, CLOSE_DIAGRAM_TITLE, DIAGRAM_MODAL_LABEL};
use crate::state::AppState;
use icondata as id;
use leptos::ev;
use leptos::prelude::*;
use leptos_icons::Icon;
use wasm_bindgen::JsCast;
use web_sys::Element;

/// Selector for rendered SVGs that are placeholders rather than diagrams.
const NON_DIAGRAM_SVG_ANCESTOR: &str = ".md-image-missing";

/// Return the outer markup of the diagram SVG clicked by `ev`, if any.
///
/// Walks out to the outermost `<svg>` so clicks on nested SVG content still
/// open the whole diagram. Missing-image placeholders are ignored.
pub fn diagram_svg_from_event(ev: &web_sys::MouseEvent) -> Option<String> {
    let target = ev.target()?.dyn_into::<Element>().ok()?;
    let mut svg = target.closest("svg").ok()??;
    while let Some(outer) = svg
        .parent_element()
        .and_then(|parent| parent.closest("svg").ok().flatten())
    {
        svg = outer;
    }
    if svg.closest(NON_DIAGRAM_SVG_ANCESTOR).ok()?.is_some() {
        return None;
    }
    Some(svg.outer_html())
}

/// Render the full-screen diagram viewer; click or Escape closes it.
#[component]
pub fn DiagramModal() -> impl IntoView {
    let svg = use_context::<AppState>()
        .expect(APP_STATE_MISSING_PANIC)
        .diagram_modal_svg;

    let handle = window_event_listener(ev::keydown, move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" && svg.get_untracked().is_some() {
            ev.prevent_default();
            svg.set(None);
        }
    });
    on_cleanup(move || handle.remove());

    view! {
        <Show when=move || svg.with(Option::is_some)>
            <div
                class="diagram-modal-backdrop"
                role="dialog"
                aria-modal="true"
                aria-label=DIAGRAM_MODAL_LABEL
                on:click=move |_| svg.set(None)
            >
                <button
                    type="button"
                    class="diagram-modal-close"
                    title=CLOSE_DIAGRAM_TITLE
                    aria-label=CLOSE_DIAGRAM_TITLE
                >
                    <Icon icon=id::LuX/>
                </button>
                <div
                    class="diagram-modal-content"
                    inner_html=move || svg.get().unwrap_or_default()
                />
            </div>
        </Show>
    }
}
