pub mod actions;
pub mod block;
pub mod edit_area;

use crate::info_messages::APP_STATE_MISSING_PANIC;
use crate::state::AppState;
use block::EditorEntryComponent;
use leptos::prelude::*;

/// Render every open editor entry in the main pane.
#[component]
pub fn Editor() -> impl IntoView {
    let state = use_context::<AppState>().expect(APP_STATE_MISSING_PANIC);
    let entries = state.entries;

    view! {
        <div class="editor-area">
            <For
                each=move || entries.get()
                key=|entry| entry.title.id
                children=move |entry| {
                    view! { <EditorEntryComponent entry=entry/> }
                }
            />
        </div>
    }
}
