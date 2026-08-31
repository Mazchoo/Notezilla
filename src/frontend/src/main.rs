mod components;
mod constants;
mod default_settings;
mod info_messages;
mod mcp;
mod models;
mod prompting;
mod rendering;
mod state;
mod utils;

use crate::info_messages::PAGE_TITLE;
use crate::prompting::probe_ollama;
use crate::utils::url::redirect_trailing_dot_hostname;
use components::app_shell::AppShell;
use leptos::prelude::*;
use state::AppState;

/// Mount the Leptos app and install the WASM panic hook.
fn main() {
    if redirect_trailing_dot_hostname() {
        return;
    }
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}

/// Provide app state, probe MCP and Ollama, and render the shell.
#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
    components::toast::bind_warning_toast(state.warning_toast);
    provide_context(state.clone());

    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title(PAGE_TITLE);
    }

    mcp::client::probe_mcp(state.session_id);
    probe_ollama(state.ollama_port, state.ollama_available);

    view! { <AppShell/> }
}
