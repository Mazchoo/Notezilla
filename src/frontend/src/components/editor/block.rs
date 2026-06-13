use crate::models::block::MarkdownBlock;
use leptos::html::Textarea;
use leptos::*;

#[component]
pub fn BlockComponent(block: MarkdownBlock, blocks: RwSignal<Vec<MarkdownBlock>>) -> impl IntoView {
    let textarea_ref = create_node_ref::<Textarea>();
    let block_id = block.id;

    // Focus the textarea whenever this block switches into edit mode.
    // create_effect runs after the DOM is updated, so the textarea exists by then.
    // Use (*el) to deref past Leptos's HtmlElement wrapper to the web-sys type.
    create_effect(move |_| {
        if block.focused.get() {
            if let Some(el) = textarea_ref.get() {
                let _ = (*el).focus();
            }
        }
    });

    // On blur: re-render markdown first, then switch out of edit mode.
    // This order ensures the div shows the fresh HTML on its first render.
    let on_blur = move |_: web_sys::FocusEvent| {
        block.rerender();
        block.focused.set(false);
    };

    let on_input = move |ev: web_sys::Event| {
        block.text.set(event_target_value(&ev));
        // Auto-resize: deref past Leptos's HtmlElement to access web-sys CssStyleDeclaration
        if let Some(el) = textarea_ref.get() {
            let _ = (*el).style().set_property("height", "auto");
            let h = (*el).scroll_height();
            let _ = (*el).style().set_property("height", &format!("{}px", h));
        }
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        match key.as_str() {
            // Enter (without Shift) → insert a new block after this one
            "Enter" if !ev.shift_key() => {
                ev.prevent_default();
                blocks.update(|b| {
                    if let Some(pos) = b.iter().position(|blk| blk.id == block_id) {
                        let new_block = MarkdownBlock::empty();
                        b.insert(pos + 1, new_block);
                        // Focus the new block on the next tick via its own create_effect
                        new_block.focused.set(true);
                    }
                });
            }
            // Backspace on an empty block → remove it (keep at least one block)
            "Backspace" if block.text.get_untracked().is_empty() => {
                if blocks.get_untracked().len() > 1 {
                    ev.prevent_default();
                    blocks.update(|b| {
                        if let Some(pos) = b.iter().position(|blk| blk.id == block_id) {
                            b.remove(pos);
                            // Focus the block above if possible
                            if pos > 0 {
                                if let Some(prev) = b.get(pos - 1) {
                                    prev.focused.set(true);
                                }
                            }
                        }
                    });
                }
            }
            _ => {}
        }
    };

    // Stop propagation so clicking a block doesn't bubble to the editor
    // background handler (which would append another empty block).
    let on_render_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        block.focused.set(true);
    };

    view! {
        <div class="editor-block">
            {move || if block.focused.get() {
                view! {
                    <textarea
                        node_ref=textarea_ref
                        class="block-textarea"
                        prop:value=move || block.text.get()
                        on:blur=on_blur
                        on:input=on_input
                        on:keydown=on_keydown
                    />
                }.into_view()
            } else {
                view! {
                    <div
                        class="block-render content"
                        inner_html=move || block.html.get()
                        on:click=on_render_click
                    />
                }.into_view()
            }}
        </div>
    }
}
