mod components;
mod constants;
mod default_settings;
mod info_messages;
mod mcp;
mod models;
mod prompting;
mod rendering;
mod state;

use components::app_shell::AppShell;
use leptos::prelude::*;
use leptos::task::spawn_local;
use state::AppState;

/// Mount the Leptos app and install the WASM panic hook.
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}

/// Provide app state, start the MCP session, and render the shell.
#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
    components::toast::bind_warning_toast(state.warning_toast);
    provide_context(state.clone());

    let session_id = state.session_id;
    spawn_local(async move {
        match mcp::client::initialize_session().await {
            Ok(id) => {
                web_sys::console::log_1(&format!("MCP session ready: {id}").into());
                session_id.set(Some(id));
            }
            Err(e) => {
                web_sys::console::warn_1(
                    &format!("MCP init failed: {e}. Search will be unavailable.").into(),
                );
            }
        }
    });

    view! { <AppShell/> }
}
