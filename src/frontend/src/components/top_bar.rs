use leptos::*;
use leptos_icons::Icon;
use icondata as id;
use crate::state::AppState;

#[component]
pub fn TopBar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");

    let on_save = move |_| {
        let content = state
            .blocks
            .get()
            .iter()
            .map(|b| b.text.get_untracked())
            .collect::<Vec<_>>()
            .join("\n\n");
        web_sys::console::log_1(&content.into());
    };

    view! {
        <div class="top-bar">
            // Import (no-op for now)
            <button class="activity-btn" title="Import">
                <Icon icon=id::LuUpload/>
            </button>
            // Save — logs full markdown to console
            <button class="activity-btn" title="Save (Ctrl+S)" on:click=on_save>
                <Icon icon=id::LuSave/>
            </button>
        </div>
    }
}
