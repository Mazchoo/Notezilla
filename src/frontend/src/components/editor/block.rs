use crate::components::editor::actions::delete_entry;
use crate::models::block::{FrontMatterBlock, MarkdownBlock, TitleBlock};
use crate::state::AppState;
use leptos::html::{Input, Textarea};
use leptos::*;

/// Renders the file-path title for an editor entry.
/// Displays the path as a styled label; click to edit inline, blur to confirm.
/// Always one line — distinct from markdown `#` headings.
#[component]
pub fn TitleBlockComponent(title: TitleBlock) -> impl IntoView {
    let input_ref = create_node_ref::<Input>();

    // Focus the input when entering edit mode.
    create_effect(move |_| {
        if title.focused.get() {
            if let Some(el) = input_ref.get() {
                let _ = (*el).focus();
            }
        }
    });

    let on_click = move |_: web_sys::MouseEvent| {
        if !title.focused.get_untracked() {
            title.focused.set(true);
        }
    };

    let on_input = move |ev: web_sys::Event| {
        title.path.set(event_target_value(&ev));
    };

    let on_blur = move |_: web_sys::FocusEvent| {
        title.focused.set(false);
    };

    // Prevent Enter from doing anything unexpected — just blur.
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            if let Some(el) = input_ref.get() {
                let _ = (*el).blur();
            }
        }
    };

    view! {
        <div>
            {move || if title.focused.get() {
                view! {
                    <input
                        node_ref=input_ref
                        class="entry-title-input"
                        type="text"
                        prop:value=move || title.path.get()
                        on:input=on_input
                        on:blur=on_blur
                        on:keydown=on_keydown
                    />
                }.into_view()
            } else {
                view! {
                    <div class="entry-title" on:click=on_click>
                        {move || title.path.get()}
                    </div>
                }.into_view()
            }}
        </div>
    }
}

/// Renders YAML front matter as a key-value table (view mode) or editable textarea (edit mode).
/// Visually distinct from markdown blocks: left-accented border, monospace key-value grid.
#[component]
pub fn FrontMatterBlockComponent(block: FrontMatterBlock) -> impl IntoView {
    let textarea_ref = create_node_ref::<Textarea>();

    create_effect(move |_| {
        if block.focused.get() {
            if let Some(el) = textarea_ref.get() {
                let _ = (*el).focus();
                let _ = (*el).style().set_property("height", "auto");
                let h = (*el).scroll_height();
                let _ = (*el).style().set_property("height", &format!("{}px", h));
            }
        }
    });

    let on_blur = move |_: web_sys::FocusEvent| {
        block.focused.set(false);
    };

    let on_input = move |ev: web_sys::Event| {
        block.raw.set(event_target_value(&ev));
        if let Some(el) = textarea_ref.get() {
            let _ = (*el).style().set_property("height", "auto");
            let h = (*el).scroll_height();
            let _ = (*el).style().set_property("height", &format!("{}px", h));
        }
    };

    let on_click = move |_: web_sys::MouseEvent| {
        if !block.focused.get_untracked() {
            block.focused.set(true);
        }
    };

    view! {
        <div class="frontmatter-block" on:click=on_click>
            {move || if block.focused.get() {
                view! {
                    <textarea
                        node_ref=textarea_ref
                        class="frontmatter-textarea"
                        prop:value=move || block.raw.get()
                        on:blur=on_blur
                        on:input=on_input
                    />
                }.into_view()
            } else {
                let raw = block.raw.get();
                let fields = FrontMatterBlock::parse_fields(&raw);
                view! {
                    <div class="frontmatter-table">
                        {fields.into_iter().map(|(k, v)| view! {
                            <span class="frontmatter-key">{k}</span>
                            <span class="frontmatter-value">{v}</span>
                        }).collect_view()}
                    </div>
                }.into_view()
            }}
        </div>
    }
}

#[component]
pub fn BlockComponent(block: MarkdownBlock, entry_id: u64) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
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
        <div class="editor-block-row">
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
            <div class="block-actions">
                <button
                    class="block-action-btn block-delete-btn"
                    title="Delete block"
                    on:mousedown=|ev: web_sys::MouseEvent| ev.prevent_default()
                    on:click=move |_| delete_entry(&state, entry_id)
                >
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="3 6 5 6 21 6"/>
                        <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
                        <path d="M10 11v6"/>
                        <path d="M14 11v6"/>
                        <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
                    </svg>
                </button>
            </div>
        </div>
    }
}
