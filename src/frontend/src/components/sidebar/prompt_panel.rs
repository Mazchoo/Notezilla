use crate::components::sidebar::file_tree_backend::FileTreeBackend;
use crate::components::toast::{show_error_toast, show_toast};
use crate::info_messages::{
    CLIPBOARD_COPY_FAILED_TOAST, COPY_BUTTON, COPY_PROMPT_TITLE, ENTER_PROMPT_TOAST,
    PROMPT_COPIED_TOAST, PROMPT_HEADING, PROMPT_PLACEHOLDER, SEND_BUTTON,
};
use crate::models::block::EditorEntry;
use crate::prompting::create_prompt;
use crate::state::AppState;
use icondata as id;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use wasm_bindgen_futures::JsFuture;

/// Serialize an editor entry to markdown, including front matter when present.
fn entry_markdown(entry: EditorEntry) -> String {
    let body = entry.content.text.get_untracked();
    match entry.front_matter.get_untracked() {
        Some(fm) => {
            let raw = fm.raw.get_untracked();
            if raw.is_empty() {
                body
            } else {
                format!("---\n{raw}\n---\n{body}")
            }
        }
        None => body,
    }
}

/// Fill the prompt template from the user text and open editor entries.
fn build_prompt(user_prompt: &str, entries: &[EditorEntry]) -> String {
    let mut context_parts = Vec::new();
    let mut response_template = String::new();
    for entry in entries {
        let markdown = entry_markdown(*entry);
        match entry.backend {
            FileTreeBackend::Notes => {
                let path = entry.title.path.get_untracked();
                context_parts.push(format!("## {path}\n\n{markdown}"));
            }
            FileTreeBackend::Templates => response_template = markdown,
        }
    }
    create_prompt(user_prompt, &context_parts.join("\n\n"), &response_template)
}

/// Return the filled prompt, or show an error toast when the user text is empty.
fn assembled_prompt(state: &AppState) -> Option<String> {
    let user_prompt = state.prompt_text.get_untracked();
    if user_prompt.trim().is_empty() {
        show_error_toast(state.error_toast, ENTER_PROMPT_TOAST);
        return None;
    }
    let entries = state.entries.get_untracked();
    Some(build_prompt(&user_prompt, &entries))
}

/// Copy `text` to the clipboard and show a success or error toast.
fn copy_text_to_clipboard(
    text: String,
    toast: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
) {
    let Some(window) = web_sys::window() else {
        web_sys::console::error_1(&"Clipboard unavailable: no window".into());
        show_error_toast(error_toast, CLIPBOARD_COPY_FAILED_TOAST);
        return;
    };
    let clipboard = window.navigator().clipboard();
    spawn_local(async move {
        match JsFuture::from(clipboard.write_text(&text)).await {
            Ok(_) => show_toast(toast, PROMPT_COPIED_TOAST),
            Err(e) => {
                web_sys::console::error_1(&e);
                show_error_toast(error_toast, CLIPBOARD_COPY_FAILED_TOAST);
            }
        }
    });
}

/// Render the Send prompt form: prompt text, copy, output path, and send.
#[component]
pub fn PromptPanel() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let prompt_text = state.prompt_text;
    let prompt_output_path = state.prompt_output_path;

    let state_send = state.clone();
    let on_send = move |_| {
        let _ = assembled_prompt(&state_send);
    };

    let state_copy = state.clone();
    let on_clipboard = move |_| {
        let Some(prompt) = assembled_prompt(&state_copy) else {
            return;
        };
        copy_text_to_clipboard(prompt, state_copy.toast, state_copy.error_toast);
    };

    view! {
        <div class="p-3 prompt-panel">
            <p class="menu-label mt-2">{PROMPT_HEADING}</p>
            <div class="field prompt-input-field">
                <div class="control">
                    <textarea
                        class="textarea prompt-textarea"
                        placeholder=PROMPT_PLACEHOLDER
                        prop:value=move || prompt_text.get()
                        on:input=move |ev| prompt_text.set(event_target_value(&ev))
                    />
                </div>
            </div>
            <div class="prompt-actions mt-2">
                <button
                    class="button is-small is-dark"
                    title=COPY_PROMPT_TITLE
                    on:click=on_clipboard
                >
                    <Icon icon=id::LuClipboard/>
                    {COPY_BUTTON}
                </button>
            </div>
            <hr class="prompt-divider"/>
            <div class="field">
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        prop:value=move || prompt_output_path.get()
                        on:input=move |ev| prompt_output_path.set(event_target_value(&ev))
                    />
                </div>
            </div>
            <div class="prompt-actions mt-2">
                <button class="button is-small is-dark" title=SEND_BUTTON on:click=on_send>
                    <Icon icon=id::LuSend/>
                    {SEND_BUTTON}
                </button>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::build_prompt;
    use crate::components::sidebar::file_tree_backend::FileTreeBackend;
    use crate::models::block::EditorEntry;
    use leptos::prelude::Owner;

    #[test]
    /// Assert open notes fill context and the open template fills the response slot.
    fn build_prompt_uses_open_notes_and_template() {
        let owner = Owner::new();
        owner.with(|| {
            let note = EditorEntry::new("./a.md", "note body");
            let mut tmpl = EditorEntry::new("./t.md", "template body");
            tmpl.backend = FileTreeBackend::Templates;
            let out = build_prompt("Ask this", &[note, tmpl]);
            assert!(out.contains("Ask this"));
            assert!(out.contains("./a.md"));
            assert!(out.contains("note body"));
            assert!(out.contains("template body"));
            assert!(!out.contains("{{USER_PROMPT}}"));
            assert!(!out.contains("{{CONTEXT}}"));
            assert!(!out.contains("{{RESPONSE_TEMPLATE}}"));
        });
    }
}
