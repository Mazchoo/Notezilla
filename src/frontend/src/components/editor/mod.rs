pub mod actions;
pub mod block;

use crate::state::AppState;
use block::{BlockComponent, FrontMatterBlockComponent, TitleBlockComponent};
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
                children=move |entry| {
                    let entry_id = entry.title.id;
                    view! {
                        <hr class="entry-divider"/>
                        <TitleBlockComponent title=entry.title/>
                        {entry.front_matter.map(|fm| view! {
                            <FrontMatterBlockComponent block=fm/>
                        })}
                        <BlockComponent block=entry.content entry_id=entry_id/>
                    }
                }
            />
        </div>
    }
}
