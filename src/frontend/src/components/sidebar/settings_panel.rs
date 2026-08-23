use crate::components::hotkeys::{format_ctrl_hotkey, normalize_hotkey_key};
use crate::state::AppState;
use leptos::prelude::*;

/// Capture a printable key press into a hotkey settings signal.
fn on_hotkey_keydown(ev: web_sys::KeyboardEvent, target: RwSignal<String>) {
    if ev.key() == "Tab" || ev.key() == "Escape" {
        return;
    }
    ev.prevent_default();
    if let Some(key) = normalize_hotkey_key(&ev.key()) {
        target.set(key);
    }
}

/// Render the settings form for search page size and hotkeys.
#[component]
pub fn SettingsPanel() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let number_results_per_page = state.number_results_per_page;
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
                    {move || format!("Save hotkey ({})", format_ctrl_hotkey(&save_hotkey_key.get()))}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || save_hotkey_key.get()
                        on:keydown=move |ev| on_hotkey_keydown(ev, save_hotkey_key)
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!(
                        "New file hotkey ({})",
                        format_ctrl_hotkey(&new_file_hotkey_key.get())
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || new_file_hotkey_key.get()
                        on:keydown=move |ev| on_hotkey_keydown(ev, new_file_hotkey_key)
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!(
                        "Import hotkey ({})",
                        format_ctrl_hotkey(&import_hotkey_key.get())
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || import_hotkey_key.get()
                        on:keydown=move |ev| on_hotkey_keydown(ev, import_hotkey_key)
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!(
                        "Export hotkey ({})",
                        format_ctrl_hotkey(&export_hotkey_key.get())
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || export_hotkey_key.get()
                        on:keydown=move |ev| on_hotkey_keydown(ev, export_hotkey_key)
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || format!(
                        "Toggle markdown editing ({})",
                        format_ctrl_hotkey(&toggle_markdown_editing_hotkey_key.get())
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || toggle_markdown_editing_hotkey_key.get()
                        on:keydown=move |ev| {
                            on_hotkey_keydown(ev, toggle_markdown_editing_hotkey_key)
                        }
                    />
                </div>
            </div>
        </div>
    }
}
