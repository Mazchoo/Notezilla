pub mod file_tree;
pub mod search_panel;

use crate::state::{ActivePanel, AppState};
use file_tree::FileTree;
use leptos::*;
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
            {move || match state.active_panel.get() {
                Some(ActivePanel::Files)  => view! { <FileTree/> }.into_view(),
                Some(ActivePanel::Search) => view! { <SearchPanel/> }.into_view(),
                None => view! {}.into_view(),
            }}
        </div>
    }
}
