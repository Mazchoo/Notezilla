use crate::components::top_bar::{
    append_new_file, export_all_as_html, open_import_picker, save_all_entries,
    toggle_markdown_editing,
};
use crate::state::AppState;
use leptos::ev;
use leptos::prelude::*;

/// Normalize a settings hotkey value to a single lowercase character.
pub fn normalize_hotkey_key(raw: &str) -> Option<char> {
    let raw = raw.trim();
    let mut chars = raw.chars();
    let ch = chars.next()?;
    if chars.next().is_some() || ch.is_control() || ch.is_whitespace() {
        return None;
    }
    Some(ch.to_ascii_lowercase())
}

/// Return whether a pressed key matches the configured hotkey letter.
fn keys_match(pressed: &str, configured: char) -> bool {
    normalize_hotkey_key(pressed) == Some(configured)
}

/// Format a Ctrl/Meta hotkey for button titles and settings labels.
pub fn format_ctrl_hotkey(key: char) -> String {
    format!("Ctrl+{}", key.to_ascii_uppercase())
}

/// Bind global Ctrl/Meta hotkeys for save, new file, import, export, and edit toggle.
#[component]
pub fn GlobalHotkeys() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");

    let handle = window_event_listener(ev::keydown, move |ev: web_sys::KeyboardEvent| {
        if !(ev.ctrl_key() || ev.meta_key()) || ev.alt_key() {
            return;
        }

        let pressed = ev.key();
        let save_key = state.save_hotkey_key.get_untracked();
        let new_file_key = state.new_file_hotkey_key.get_untracked();
        let import_key = state.import_hotkey_key.get_untracked();
        let export_key = state.export_hotkey_key.get_untracked();
        let toggle_editing_key = state.toggle_markdown_editing_hotkey_key.get_untracked();

        if keys_match(&pressed, save_key) {
            ev.prevent_default();
            save_all_entries(&state);
            return;
        }

        if keys_match(&pressed, new_file_key) {
            ev.prevent_default();
            append_new_file(&state);
            return;
        }

        if keys_match(&pressed, import_key) {
            ev.prevent_default();
            open_import_picker(&state);
            return;
        }

        if keys_match(&pressed, export_key) {
            ev.prevent_default();
            export_all_as_html(&state);
            return;
        }

        if keys_match(&pressed, toggle_editing_key) {
            ev.prevent_default();
            toggle_markdown_editing(&state);
        }
    });

    on_cleanup(move || handle.remove());

    ()
}
