pub mod block;

use crate::state::AppState;
use block::BlockComponent;
use leptos::*;

#[component]
pub fn Editor() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let blocks = state.blocks;

    view! {
        <div class="editor-area">
            <For
                each=move || blocks.get()
                key=|block| block.id
                children=move |block| view! {
                    <BlockComponent block=block/>
                }
            />
        </div>
    }
}
