mod components;
mod mcp;
mod models;
mod state;

use components::app_shell::AppShell;
use leptos::*;
use state::AppState;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}

#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
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
