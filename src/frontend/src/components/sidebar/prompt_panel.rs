use crate::components::file_io::{display_note_path, normalize_note_path};
use crate::components::toast::{show_error_toast, show_toast};
use crate::constants::OLLAMA_URL;
use crate::info_messages::{
    ollama_send_failed_toast, ollama_status_title, prompt_response_saved_toast, save_failed_toast,
    CLIPBOARD_COPY_FAILED_TOAST, COPY_BUTTON, COPY_PROMPT_TITLE, ENTER_OLLAMA_MODEL_TOAST,
    ENTER_OUTPUT_PATH_TOAST, ENTER_PROMPT_TOAST, MCP_SESSION_NOT_READY_TOAST, OLLAMA_STATUS_LABEL,
    PROMPT_COPIED_TOAST, PROMPT_HEADING, PROMPT_OUTPUT_PATH_TITLE, PROMPT_PLACEHOLDER,
    SENDING_PROMPT_LABEL, SEND_BUTTON,
};
use crate::mcp::tools::upsert_note;
use crate::prompting::{build_prompt, check_connection, send_prompt};
use crate::state::AppState;
use icondata as id;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use serde_json::json;
use wasm_bindgen_futures::JsFuture;

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

/// Return the CSS class for the Ollama availability badge.
fn ollama_status_class(available: bool) -> &'static str {
    if available {
        "ollama-status-badge is-available"
    } else {
        "ollama-status-badge is-unavailable"
    }
}

/// Return the console message for a successful Ollama connection.
fn ollama_ready_log() -> String {
    format!("Ollama connection ready: {OLLAMA_URL}")
}

/// Return the normalised save path when it is non-empty.
fn prompt_save_path(raw: &str) -> Option<String> {
    let path = normalize_note_path(raw);
    (!path.is_empty()).then_some(path)
}

/// Probe Ollama on `port` and store whether the API responded.
fn probe_ollama(port: u16, available: RwSignal<bool>) {
    spawn_local(async move {
        match check_connection(port).await {
            Ok(()) => {
                web_sys::console::log_1(&ollama_ready_log().into());
                available.set(true);
            }
            Err(e) => {
                web_sys::console::warn_1(
                    &format!("Ollama init failed: {e}. Prompt send will be unavailable.").into(),
                );
                available.set(false);
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
    let ollama_port = state.ollama_port;
    let ollama_available = RwSignal::new(false);
    let sending = RwSignal::new(false);

    Effect::new(move |_| {
        probe_ollama(ollama_port.get(), ollama_available);
    });

    let state_send = state.clone();
    let on_send = move |_| {
        if sending.get_untracked() {
            return;
        }
        let Some(prompt) = assembled_prompt(&state_send) else {
            return;
        };
        let model = state_send.ollama_model.get_untracked();
        if model.trim().is_empty() {
            show_error_toast(state_send.error_toast, ENTER_OLLAMA_MODEL_TOAST);
            return;
        }
        let Some(path) = prompt_save_path(&state_send.prompt_output_path.get_untracked()) else {
            show_error_toast(state_send.error_toast, ENTER_OUTPUT_PATH_TOAST);
            return;
        };
        let Some(session_id) = state_send.session_id.get_untracked() else {
            web_sys::console::warn_1(&"MCP session not ready".into());
            show_error_toast(state_send.error_toast, MCP_SESSION_NOT_READY_TOAST);
            return;
        };
        let port = state_send.ollama_port.get_untracked();
        let toast = state_send.toast;
        let error_toast = state_send.error_toast;
        let file_tree_epoch = state_send.file_tree_epoch;
        sending.set(true);
        spawn_local(async move {
            match send_prompt(port, &model, &prompt).await {
                Ok(text) => {
                    ollama_available.set(true);
                    write_prompt_response(
                        &session_id,
                        &path,
                        &text,
                        toast,
                        error_toast,
                        file_tree_epoch,
                    )
                    .await;
                }
                Err(e) => {
                    ollama_available.set(false);
                    web_sys::console::error_1(&e.clone().into());
                    show_error_toast(error_toast, ollama_send_failed_toast(e));
                }
            }
            sending.set(false);
        });
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
                        title=PROMPT_OUTPUT_PATH_TITLE
                        prop:value=move || prompt_output_path.get()
                        on:input=move |ev| prompt_output_path.set(event_target_value(&ev))
                    />
                </div>
            </div>
            <div class="prompt-actions mt-2">
                <button
                    class="button is-small is-dark"
                    title=SEND_BUTTON
                    prop:disabled=move || sending.get()
                    on:click=on_send
                >
                    <Icon icon=id::LuSend/>
                    {SEND_BUTTON}
                </button>
                <span
                    class=move || ollama_status_class(ollama_available.get())
                    title=move || ollama_status_title(ollama_available.get())
                    role="status"
                >
                    <Icon icon=id::LuCircle/>
                    {OLLAMA_STATUS_LABEL}
                </span>
                <Show when=move || sending.get()>
                    <span
                        class="prompt-send-spinner"
                        role="status"
                        aria-label=SENDING_PROMPT_LABEL
                    ></span>
                </Show>
            </div>
        </div>
    }
}

/// Write the Ollama completion to `path` and toast when the note is saved.
async fn write_prompt_response(
    session_id: &str,
    path: &str,
    text: &str,
    toast: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
    file_tree_epoch: RwSignal<u64>,
) {
    match upsert_note(session_id, path, text, json!({})).await {
        Ok(_) => {
            file_tree_epoch.update(|n| *n = n.wrapping_add(1));
            show_toast(toast, prompt_response_saved_toast(&display_note_path(path)));
        }
        Err(e) => {
            web_sys::console::error_1(&format!("Write failed for {path}: {e}").into());
            show_error_toast(error_toast, save_failed_toast(path, e));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ollama_ready_log, ollama_status_class, prompt_save_path};
    use crate::constants::OLLAMA_URL;
    use crate::default_settings::DEFAULT_PROMPT_OUTPUT_PATH;

    #[test]
    /// Assert the response path field is a save target, not an empty display value.
    fn prompt_save_path_keeps_the_response_file() {
        assert_eq!(
            prompt_save_path(DEFAULT_PROMPT_OUTPUT_PATH).as_deref(),
            Some("prompt_response.md")
        );
        assert_eq!(prompt_save_path("  ").as_deref(), None);
    }

    #[test]
    /// Assert a successful Ollama probe logs the same-origin proxy path.
    fn ollama_ready_log_names_the_proxy_path() {
        assert_eq!(
            ollama_ready_log(),
            format!("Ollama connection ready: {OLLAMA_URL}")
        );
    }

    #[test]
    /// Assert the Ollama badge class switches between available and unavailable.
    fn ollama_status_class_marks_available_or_unavailable() {
        assert_eq!(
            ollama_status_class(true),
            "ollama-status-badge is-available"
        );
        assert_eq!(
            ollama_status_class(false),
            "ollama-status-badge is-unavailable"
        );
    }
}
