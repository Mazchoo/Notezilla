pub mod actions;
pub mod block;

use crate::state::AppState;
use block::{BlockComponent, FrontMatterBlockComponent, TitleBlockComponent};
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
                    let entry_id = entry.title.id;
                    let front_matter_signal = entry.front_matter;
                    view! {
                        <hr class="entry-divider"/>
                        <TitleBlockComponent title=entry.title entry_id=entry_id/>
                        {move || front_matter_signal.get().map(|fm| view! {
                            <FrontMatterBlockComponent block=fm entry_id=entry_id/>
                        })}
                        <BlockComponent block=entry.content/>
                    }
                }
            />
        </div>
    }
}
