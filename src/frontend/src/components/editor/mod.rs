pub mod block;

use crate::state::AppState;
use block::{BlockComponent, TitleBlockComponent};
use leptos::*;

#[component]
pub fn Editor() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let entries = state.entries;

    view! {
        <div class="editor-area">
            <For
                each=move || entries.get()
                key=|entry| entry.title.id
                children=move |entry| view! {
                    <hr class="entry-divider"/>
                    <TitleBlockComponent title=entry.title/>
                    <BlockComponent block=entry.content/>
                }
            />
        </div>
    }
}
