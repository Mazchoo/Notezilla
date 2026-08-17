pub mod actions;
pub mod block;
pub mod edit_area;

use crate::state::AppState;
use block::EditorEntryComponent;
use leptos::prelude::*;

#[component]
pub fn Editor() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
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
