use crate::state::{ActivePanel, AppState};
use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn ActivityBar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");

    let toggle = move |panel: ActivePanel| {
        state.active_panel.update(|current| {
            *current = if *current == Some(panel) {
                None
            } else {
                Some(panel)
            };
        });
    };

    let is_active = move |panel: ActivePanel| state.active_panel.get() == Some(panel);

    view! {
        <div class="activity-bar">
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Files))
                title="Files"
                on:click=move |_| toggle(ActivePanel::Files)
            >
                <Icon icon=id::LuFiles/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Templates))
                title="Templates"
                on:click=move |_| toggle(ActivePanel::Templates)
            >
                <Icon icon=id::LuCopy/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Search))
                title="Search"
                on:click=move |_| toggle(ActivePanel::Search)
            >
                <Icon icon=id::LuSearch/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Settings))
                title="Settings"
                on:click=move |_| toggle(ActivePanel::Settings)
            >
                <Icon icon=id::LuSettings/>
            </button>
        </div>
    }
}
