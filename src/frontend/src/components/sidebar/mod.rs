pub mod file_tree;
pub mod search_panel;

use crate::state::{ActivePanel, AppState};
use file_tree::FileTree;
use leptos::either::EitherOf3;
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
            {move || match state.active_panel.get() {
                Some(ActivePanel::Files)  => EitherOf3::A(view! { <FileTree/> }),
                Some(ActivePanel::Search) => EitherOf3::B(view! { <SearchPanel/> }),
                None => EitherOf3::C(()),
            }}
        </div>
    }
}
