mod components;
mod constants;
mod default_settings;
mod info_messages;
mod mcp;
mod models;
mod prompting;
mod rendering;
mod state;

use crate::info_messages::PAGE_TITLE;
use components::app_shell::AppShell;
use leptos::prelude::*;
use leptos::task::spawn_local;
use state::AppState;

/// Return a URL with a trailing FQDN dot stripped from `hostname`, if present.
fn url_without_trailing_dot_host(
    protocol: &str,
    hostname: &str,
    port: &str,
    rest: &str,
) -> Option<String> {
    let hostname = hostname.strip_suffix('.')?;
    if hostname.is_empty() {
        return None;
    }
    let host = if port.is_empty() {
        hostname.to_string()
    } else {
        format!("{hostname}:{port}")
    };
    Some(format!("{protocol}//{host}{rest}"))
}

/// Reload when the hostname ends with a DNS-root trailing dot.
fn redirect_trailing_dot_hostname() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let location = window.location();
    let (Ok(protocol), Ok(hostname), Ok(port), Ok(pathname), Ok(search), Ok(hash)) = (
        location.protocol(),
        location.hostname(),
        location.port(),
        location.pathname(),
        location.search(),
        location.hash(),
    ) else {
        return false;
    };
    let Some(url) = url_without_trailing_dot_host(
        &protocol,
        &hostname,
        &port,
        &format!("{pathname}{search}{hash}"),
    ) else {
        return false;
    };
    let _ = location.replace(&url);
    true
}

/// Mount the Leptos app and install the WASM panic hook.
fn main() {
    if redirect_trailing_dot_hostname() {
        return;
    }
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}

/// Provide app state, start the MCP session, and render the shell.
#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
    components::toast::bind_warning_toast(state.warning_toast);
    provide_context(state.clone());

    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title(PAGE_TITLE);
    }

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

#[cfg(test)]
mod tests {
    use super::url_without_trailing_dot_host;

    #[test]
    /// Assert a trailing FQDN dot is stripped from the hostname and left unchanged otherwise.
    fn url_without_trailing_dot_host_strips_dns_root_dot() {
        assert_eq!(
            url_without_trailing_dot_host("http:", "localhost.", "8080", "/"),
            Some("http://localhost:8080/".into())
        );
        assert_eq!(
            url_without_trailing_dot_host("http:", "localhost", "8080", "/"),
            None
        );
    }
}
