pub mod block;

use crate::models::block::MarkdownBlock;
use crate::state::AppState;
use block::BlockComponent;
use leptos::*;

#[component]
pub fn Editor() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let blocks = state.blocks;

    // Clicking the editor background (not on a block) appends + focuses a new empty block.
    // BlockComponent stops propagation on its own click events.
    let on_bg_click = move |_| {
        blocks.update(|b| {
            if let Some(last) = b.last() {
                if last.text.get_untracked().is_empty() {
                    last.focused.set(true);
                    return;
                }
            }
            let new_block = MarkdownBlock::empty();
            new_block.focused.set(true);
            b.push(new_block);
        });
    };

    view! {
        <div class="editor-area" on:click=on_bg_click>
            <For
                each=move || blocks.get()
                key=|block| block.id
                children=move |block| view! {
                    <BlockComponent block=block blocks=blocks/>
                }
            />
        </div>
    }
}
