pub mod context_menu;
pub mod file_tree;
pub mod file_tree_backend;
pub mod new_folder_modal;
pub mod prompt_panel;
pub mod rename_modal;
pub mod search_panel;
pub mod settings_panel;
mod settings_validation;

use crate::constants::{SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH};
use crate::info_messages::{
    NOTES_HEADING, NOTE_FOLDER_ROOT_LABEL, TEMPLATES_HEADING, TEMPLATE_FOLDER_ROOT_LABEL,
};
use crate::state::{ActivePanel, AppState};
use file_tree::FileTree;
use file_tree_backend::FileTreeBackend;
use leptos::prelude::*;
use prompt_panel::PromptPanel;
use search_panel::SearchPanel;
use settings_panel::SettingsPanel;

/// Render the resizable sidebar and the active files, templates, search, prompt, or settings panel.
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
                <FileTree
                    backend=FileTreeBackend::Notes
                    heading=NOTES_HEADING
                    root_label=NOTE_FOLDER_ROOT_LABEL
                    epoch=state.file_tree_epoch
                />
            </div>
            <div class=move || {
                if state.active_panel.get() == Some(ActivePanel::Templates) {
                    ""
                } else {
                    "is-hidden"
                }
            }>
                <FileTree
                    backend=FileTreeBackend::Templates
                    heading=TEMPLATES_HEADING
                    root_label=TEMPLATE_FOLDER_ROOT_LABEL
                    epoch=state.template_tree_epoch
                />
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
            <div class=move || {
                if state.active_panel.get() == Some(ActivePanel::Prompt) {
                    ""
                } else {
                    "is-hidden"
                }
            }>
                <PromptPanel/>
            </div>
            <div class=move || {
                if state.active_panel.get() == Some(ActivePanel::Settings) {
                    ""
                } else {
                    "is-hidden"
                }
            }>
                <SettingsPanel/>
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
