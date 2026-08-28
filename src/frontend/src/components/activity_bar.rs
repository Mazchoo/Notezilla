use crate::info_messages::{
    ACTIVITY_FILES_TITLE, ACTIVITY_PROMPT_TITLE, ACTIVITY_SEARCH_TITLE, ACTIVITY_SETTINGS_TITLE,
    ACTIVITY_TEMPLATES_TITLE,
};
use crate::state::{ActivePanel, AppState};
use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;

/// Render the activity-bar panel switcher.
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
                title=ACTIVITY_FILES_TITLE
                on:click=move |_| toggle(ActivePanel::Files)
            >
                <Icon icon=id::LuFiles/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Templates))
                title=ACTIVITY_TEMPLATES_TITLE
                on:click=move |_| toggle(ActivePanel::Templates)
            >
                <Icon icon=id::LuCopy/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Search))
                title=ACTIVITY_SEARCH_TITLE
                on:click=move |_| toggle(ActivePanel::Search)
            >
                <Icon icon=id::LuSearch/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Prompt))
                title=ACTIVITY_PROMPT_TITLE
                on:click=move |_| toggle(ActivePanel::Prompt)
            >
                <Icon icon=id::LuMessageSquare/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Settings))
                title=ACTIVITY_SETTINGS_TITLE
                on:click=move |_| toggle(ActivePanel::Settings)
            >
                <Icon icon=id::LuSettings/>
            </button>
        </div>
    }
}
