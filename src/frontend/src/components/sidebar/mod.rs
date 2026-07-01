pub mod context_menu;
pub mod file_tree;
pub mod search_panel;

use crate::state::{ActivePanel, AppState};
use file_tree::FileTree;
use leptos::prelude::*;
use search_panel::SearchPanel;

#[component]
pub fn Sidebar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");

    view! {
        <div class=move || {
            if state.active_panel.get().is_none() {
                "sidebar-panel collapsed"
            } else {
                "sidebar-panel"
            }
        }>
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
        </div>
    }
}
