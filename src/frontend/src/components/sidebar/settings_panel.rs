use crate::components::hotkeys::{format_ctrl_hotkey, normalize_hotkey_key};
use crate::state::AppState;
use leptos::prelude::*;

/// Capture a printable key press into a hotkey settings signal.
fn on_hotkey_keydown(ev: web_sys::KeyboardEvent, target: RwSignal<char>) {
    if ev.key() == "Tab" || ev.key() == "Escape" {
        return;
    }
    ev.prevent_default();
    if let Some(key) = normalize_hotkey_key(&ev.key()) {
        target.set(key);
    }
}

/// Parse a TCP port from a settings input, rejecting 0 and non-numeric values.
fn parse_ollama_port(raw: &str) -> Option<u16> {
    let n = raw.parse::<u16>().ok()?;
    (n != 0).then_some(n)
}

/// Render the settings form for search page size, Ollama port, and hotkeys.
#[component]
pub fn SettingsPanel() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let number_results_per_page = state.number_results_per_page;
    let ollama_port = state.ollama_port;
    let save_hotkey_key = state.save_hotkey_key;
    let new_file_hotkey_key = state.new_file_hotkey_key;
    let import_hotkey_key = state.import_hotkey_key;
    let export_hotkey_key = state.export_hotkey_key;
    let toggle_markdown_editing_hotkey_key = state.toggle_markdown_editing_hotkey_key;

    let on_results_input = move |ev| {
        let raw = event_target_value(&ev);
        let Ok(n) = raw.parse::<usize>() else {
            return;
        };
        if n == 0 {
            return;
        }
        number_results_per_page.set(n);
    };

    let on_ollama_port_input = move |ev| {
        let Some(port) = parse_ollama_port(&event_target_value(&ev)) else {
            return;
        };
        ollama_port.set(port);
    };

    view! {
        <div class="p-3">
            <p class="menu-label mt-2">"SETTINGS"</p>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    "Results per page"
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="number"
                        min="1"
                        prop:value=move || number_results_per_page.get().to_string()
                        on:input=on_results_input
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    "Ollama port"
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="number"
                        min="1"
                        max="65535"
                        prop:value=move || ollama_port.get().to_string()
                        on:input=on_ollama_port_input
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!("Save hotkey ({})", format_ctrl_hotkey(save_hotkey_key.get()))}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || save_hotkey_key.get().to_string()
                        on:keydown=move |ev| on_hotkey_keydown(ev, save_hotkey_key)
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!(
                        "New file hotkey ({})",
                        format_ctrl_hotkey(new_file_hotkey_key.get())
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || new_file_hotkey_key.get().to_string()
                        on:keydown=move |ev| on_hotkey_keydown(ev, new_file_hotkey_key)
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!(
                        "Import hotkey ({})",
                        format_ctrl_hotkey(import_hotkey_key.get())
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || import_hotkey_key.get().to_string()
                        on:keydown=move |ev| on_hotkey_keydown(ev, import_hotkey_key)
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!(
                        "Export hotkey ({})",
                        format_ctrl_hotkey(export_hotkey_key.get())
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || export_hotkey_key.get().to_string()
                        on:keydown=move |ev| on_hotkey_keydown(ev, export_hotkey_key)
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!(
                        "Toggle markdown editing ({})",
                        format_ctrl_hotkey(toggle_markdown_editing_hotkey_key.get())
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || toggle_markdown_editing_hotkey_key.get().to_string()
                        on:keydown=move |ev| {
                            on_hotkey_keydown(ev, toggle_markdown_editing_hotkey_key)
                        }
                    />
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::parse_ollama_port;
    use crate::default_settings::DEFAULT_OLLAMA_PORT;

    #[test]
    /// Assert the Ollama port input accepts 1..=65535 and rejects 0 and non-numeric values.
    fn parse_ollama_port_accepts_valid_tcp_ports() {
        assert_eq!(parse_ollama_port("11434"), Some(DEFAULT_OLLAMA_PORT));
        assert_eq!(parse_ollama_port("1"), Some(1));
        assert_eq!(parse_ollama_port("65535"), Some(65535));
        assert_eq!(parse_ollama_port("0"), None);
        assert_eq!(parse_ollama_port(""), None);
        assert_eq!(parse_ollama_port("abc"), None);
        assert_eq!(parse_ollama_port("65536"), None);
    }
}
