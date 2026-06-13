use crate::models::block::MarkdownBlock;
use leptos::html::Textarea;
use leptos::*;

#[component]
pub fn BlockComponent(block: MarkdownBlock) -> impl IntoView {
    let textarea_ref = create_node_ref::<Textarea>();

    // Focus the textarea whenever this block switches into edit mode.
    create_effect(move |_| {
        if block.focused.get() {
            if let Some(el) = textarea_ref.get() {
                let _ = (*el).focus();
                // Auto-resize to fit existing content on initial render.
                let _ = (*el).style().set_property("height", "auto");
                let h = (*el).scroll_height();
                let _ = (*el).style().set_property("height", &format!("{}px", h));
            }
        }
    });

    // On blur: re-render markdown and leave edit mode.
    let on_blur = move |_: web_sys::FocusEvent| {
        block.rerender();
        block.focused.set(false);
    };

    let on_input = move |ev: web_sys::Event| {
        block.text.set(event_target_value(&ev));
        // Auto-resize textarea to fit content.
        if let Some(el) = textarea_ref.get() {
            let _ = (*el).style().set_property("height", "auto");
            let h = (*el).scroll_height();
            let _ = (*el).style().set_property("height", &format!("{}px", h));
        }
    };

    // Click anywhere on the block wrapper to enter edit mode.
    let on_block_click = move |_: web_sys::MouseEvent| {
        if !block.focused.get_untracked() {
            block.focused.set(true);
        }
    };

    view! {
        <div class="editor-block" on:click=on_block_click>
            {move || if block.focused.get() {
                view! {
                    <textarea
                        node_ref=textarea_ref
                        class="block-textarea"
                        prop:value=move || block.text.get()
                        on:blur=on_blur
                        on:input=on_input
                    />
                }.into_view()
            } else {
                view! {
                    <div
                        class="block-render content"
                        inner_html=move || block.html.get()
                    />
                }.into_view()
            }}
        </div>
    }
}
