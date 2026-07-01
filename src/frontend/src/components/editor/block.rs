use crate::components::editor::actions::{add_front_matter, delete_entry, delete_front_matter};
use crate::models::block::{FrontMatterBlock, MarkdownBlock, TitleBlock};
use crate::state::AppState;
use leptos::either::Either;
use leptos::html::{Input, Textarea};
use leptos::prelude::*;

/// Renders the file-path title for an editor entry.
/// Displays the path as a styled label; click to edit inline, blur to confirm.
/// Always one line — distinct from markdown `#` headings.
#[component]
pub fn TitleBlockComponent(
    title: TitleBlock,
    entry_id: u64,
    front_matter: RwSignal<Option<FrontMatterBlock>>,
) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let input_ref = NodeRef::<Input>::new();

    // Focus the input when entering edit mode.
    Effect::new(move |_| {
        if title.focused.try_get().unwrap_or(false) {
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

    // Clone state into signals so closures can be Fn (not FnOnce).
    let state_add = state.clone();
    let state_del = state.clone();

    view! {
        <div class="editor-block-row title-block-row">
            <div class="editor-block">
                {move || if title.focused.try_get().unwrap_or(false) {
                    Either::Left(view! {
                        <input
                            node_ref=input_ref
                            class="entry-title-input"
                            type="text"
                            prop:value=move || title.path.try_get().unwrap_or_default()
                            on:input=on_input
                            on:blur=on_blur
                            on:keydown=on_keydown
                        />
                    })
                } else {
                    Either::Right(view! {
                        <div class="entry-title" on:click=on_click>
                            {move || title.path.try_get().unwrap_or_default()}
                        </div>
                    })
                }}
            </div>
            <div class="block-actions block-actions-row">
                {move || {
                    if front_matter.try_get().unwrap_or(None).is_none() {
                        let s = state_add.clone();
                        Either::Left(view! {
                            <button
                                class="block-action-btn block-add-frontmatter-btn"
                                title="Add frontmatter"
                                on:mousedown=|ev: web_sys::MouseEvent| ev.prevent_default()
                                on:click=move |_| add_front_matter(&s, entry_id)
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M8 3H7a2 2 0 0 0-2 2v5a2 2 0 0 1-2 2 2 2 0 0 1 2 2v5c0 1.1.9 2 2 2h1"/>
                                    <path d="M16 21h1a2 2 0 0 0 2-2v-5c0-1.1.9-2 2-2a2 2 0 0 1-2-2V5a2 2 0 0 0-2-2h-1"/>
                                </svg>
                            </button>
                        })
                    } else {
                        Either::Right(view! { <span/> })
                    }
                }}
                <button
                    class="block-action-btn block-delete-btn"
                    title="Delete block"
                    on:mousedown=|ev: web_sys::MouseEvent| ev.prevent_default()
                    on:click=move |_| delete_entry(&state_del, entry_id)
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

/// Renders YAML front matter as a key-value table (view mode) or editable textarea (edit mode).
/// Visually distinct from markdown blocks: left-accented border, monospace key-value grid.
/// Includes a delete button that removes only the front matter from the entry.
#[component]
pub fn FrontMatterBlockComponent(block: FrontMatterBlock, entry_id: u64) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let textarea_ref = NodeRef::<Textarea>::new();

    Effect::new(move |_| {
        if block.focused.try_get().unwrap_or(false) {
            request_animation_frame(move || {
                if let Some(el) = textarea_ref.get_untracked() {
                    let _ = (*el).focus();
                    let _ = (*el).style().set_property("height", "auto");
                    let h = (*el).scroll_height();
                    let _ = (*el).style().set_property("height", &format!("{}px", h));
                }
            });
        }
    });

    let on_blur = move |_: web_sys::FocusEvent| {
        block.focused.set(false);
    };

    let on_input = move |ev: web_sys::Event| {
        block.raw.set(event_target_value(&ev));
        if let Some(el) = textarea_ref.get_untracked() {
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
        <div class="editor-block-row">
            <div class="frontmatter-block" on:click=on_click>
                {move || if block.focused.try_get().unwrap_or(false) {
                    Either::Left(view! {
                        <textarea
                            node_ref=textarea_ref
                            class="frontmatter-textarea"
                            prop:value=move || block.raw.try_get().unwrap_or_default()
                            on:blur=on_blur
                            on:input=on_input
                        />
                    })
                } else {
                    let raw = block.raw.try_get().unwrap_or_default();
                    let fields = FrontMatterBlock::parse_fields(&raw);
                    Either::Right(view! {
                        <div class="frontmatter-table">
                            {fields.into_iter().map(|(k, v)| view! {
                                <span class="frontmatter-key">{k}</span>
                                <span class="frontmatter-value">{v}</span>
                            }).collect_view()}
                        </div>
                    })
                }}
            </div>
            <div class="block-actions">
                <button
                    class="block-action-btn block-delete-btn"
                    title="Delete front matter"
                    on:mousedown=|ev: web_sys::MouseEvent| ev.prevent_default()
                    on:click=move |ev: web_sys::MouseEvent| {
                        ev.stop_propagation();
                        delete_front_matter(&state, entry_id);
                    }
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

#[component]
pub fn BlockComponent(block: MarkdownBlock) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let markdown_editing_enabled = state.markdown_editing_enabled;
    let textarea_ref = NodeRef::<Textarea>::new();

    // Leave edit mode when main-text editing is turned off.
    Effect::new(move |_| {
        if !markdown_editing_enabled.get() && block.focused.try_get().unwrap_or(false) {
            block.rerender();
            block.focused.set(false);
        }
    });

    // Focus & auto-size the textarea once it's switched into edit mode.
    //
    // The resize is deferred to `request_animation_frame` so the new
    // textarea has already been mounted and the browser has laid it out
    // before we read `scrollHeight`. Without this, the user Effect
    // (scheduled before the view's RenderEffect under Leptos 0.8's async
    // scheduler) can run while `textarea_ref` is still empty, leaving the
    // textarea stuck at the HTML default of two visible rows.
    Effect::new(move |_| {
        if markdown_editing_enabled.get() && block.focused.try_get().unwrap_or(false) {
            request_animation_frame(move || {
                if let Some(el) = textarea_ref.get_untracked() {
                    let _ = (*el).focus();
                    let _ = (*el).style().set_property("height", "auto");
                    let h = (*el).scroll_height();
                    let _ = (*el).style().set_property("height", &format!("{}px", h));
                }
            });
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
        if let Some(el) = textarea_ref.get_untracked() {
            let _ = (*el).style().set_property("height", "auto");
            let h = (*el).scroll_height();
            let _ = (*el).style().set_property("height", &format!("{}px", h));
        }
    };

    // Click anywhere on the block wrapper to enter edit mode.
    let on_block_click = move |_: web_sys::MouseEvent| {
        if markdown_editing_enabled.get_untracked() && !block.focused.get_untracked() {
            block.focused.set(true);
        }
    };

    view! {
        <div
            class=move || if markdown_editing_enabled.get() {
                "editor-block"
            } else {
                "editor-block editor-block-frozen"
            }
            on:click=on_block_click
        >
            {move || if block.focused.try_get().unwrap_or(false) {
                Either::Left(view! {
                    <textarea
                        node_ref=textarea_ref
                        class="block-textarea"
                        prop:value=move || block.text.try_get().unwrap_or_default()
                        on:blur=on_blur
                        on:input=on_input
                    />
                })
            } else {
                Either::Right(view! {
                    <div
                        class="block-render content"
                        inner_html=move || block.html.try_get().unwrap_or_default()
                    />
                })
            }}
        </div>
    }
}
