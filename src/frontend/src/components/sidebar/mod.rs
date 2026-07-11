pub mod context_menu;
pub mod file_tree;
pub mod search_panel;

use crate::state::{ActivePanel, AppState};
use file_tree::FileTree;
use leptos::prelude::*;
use search_panel::SearchPanel;

const SIDEBAR_MIN_WIDTH: f64 = 160.0;
const SIDEBAR_MAX_WIDTH: f64 = 600.0;

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let sidebar_width = state.sidebar_width;
    let resizing = RwSignal::new(false);
    let drag_start = RwSignal::new(None::<(i32, f64)>);

    let on_resize_start = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        drag_start.set(Some((ev.client_x(), sidebar_width.get_untracked())));
        resizing.set(true);
    };

    let on_resize_move = move |ev: web_sys::MouseEvent| {
        let Some((start_x, start_width)) = drag_start.get_untracked() else {
            return;
        };
        let delta = f64::from(ev.client_x() - start_x);
        let next = (start_width + delta).clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
        sidebar_width.set(next);
    };

    let on_resize_end = move |_| {
        resizing.set(false);
        drag_start.set(None);
    };

    view! {
        <div
            class=move || {
                let collapsed = state.active_panel.get().is_none();
                let resizing_now = resizing.get();
                match (collapsed, resizing_now) {
                    (true, _) => "sidebar-panel collapsed",
                    (false, true) => "sidebar-panel resizing",
                    (false, false) => "sidebar-panel",
                }
            }
            style=move || {
                if state.active_panel.get().is_none() {
                    String::new()
                } else {
                    let w = sidebar_width.get();
                    format!("width:{w}px;min-width:{w}px;")
                }
            }
        >
            <div class=move || {
                if state.active_panel.get() == Some(ActivePanel::Files) {
                    ""
                } else {
                    "is-hidden"
                }
            }>
                <FileTree/>
            </div>
            <div class=move || {
                if state.active_panel.get() == Some(ActivePanel::Search) {
                    ""
                } else {
                    "is-hidden"
                }
            }>
                <SearchPanel/>
            </div>
            <Show when=move || state.active_panel.get().is_some()>
                <div
                    class="sidebar-resize-handle"
                    on:mousedown=on_resize_start
                />
            </Show>
        </div>
        <Show when=move || resizing.get()>
            <div
                class="sidebar-resize-overlay"
                on:mousemove=on_resize_move
                on:mouseup=on_resize_end
                on:mouseleave=on_resize_end
            />
        </Show>
    }
}
