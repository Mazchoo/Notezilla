use crate::info_messages::{
    ACTIVITY_FILES_TITLE, ACTIVITY_PROMPT_TITLE, ACTIVITY_SEARCH_TITLE, ACTIVITY_SETTINGS_TITLE,
    ACTIVITY_SYNC_TITLE, ACTIVITY_TEMPLATES_TITLE,
};
use crate::state::{ActivePanel, AppState};
use icondata as id;
use leptos::prelude::*;
use leptos_icons::Icon;

/// Render the activity-bar panel switcher.
///
/// Panels that need the backend are disabled while it is unavailable.
#[component]
pub fn ActivityBar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let mcp_available = state.mcp_available;
    let mcp_probing = state.mcp_probing;

    let is_disabled =
        move |panel: ActivePanel| panel.needs_backend() && !mcp_available.get();

    // Close a backend panel once a probe has confirmed the backend is down.
    Effect::new(move |_| {
        if mcp_available.get() || mcp_probing.get() {
            return;
        }
        if state
            .active_panel
            .get_untracked()
            .is_some_and(ActivePanel::needs_backend)
        {
            state.active_panel.set(None);
        }
    });

    let toggle = move |panel: ActivePanel| {
        if panel.needs_backend() && !mcp_available.get_untracked() {
            return;
        }
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
                prop:disabled=move || is_disabled(ActivePanel::Files)
                on:click=move |_| toggle(ActivePanel::Files)
            >
                <Icon icon=id::LuFiles/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Templates))
                title=ACTIVITY_TEMPLATES_TITLE
                prop:disabled=move || is_disabled(ActivePanel::Templates)
                on:click=move |_| toggle(ActivePanel::Templates)
            >
                <Icon icon=id::LuCopy/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Search))
                title=ACTIVITY_SEARCH_TITLE
                prop:disabled=move || is_disabled(ActivePanel::Search)
                on:click=move |_| toggle(ActivePanel::Search)
            >
                <Icon icon=id::LuSearch/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Prompt))
                title=ACTIVITY_PROMPT_TITLE
                prop:disabled=move || is_disabled(ActivePanel::Prompt)
                on:click=move |_| toggle(ActivePanel::Prompt)
            >
                <Icon icon=id::LuMessageSquare/>
            </button>
            <button
                class=move || AppState::activity_btn_class(is_active(ActivePanel::Sync))
                title=ACTIVITY_SYNC_TITLE
                prop:disabled=move || is_disabled(ActivePanel::Sync)
                on:click=move |_| toggle(ActivePanel::Sync)
            >
                <Icon icon=id::LuDatabase/>
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
