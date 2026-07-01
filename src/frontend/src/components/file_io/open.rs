use crate::mcp::tools::get_note;
use crate::models::block::EditorEntry;
use leptos::prelude::*;
use leptos::task::spawn_local;

use super::import::{entry_from_note, open_note_in_editor};
use super::save::display_note_path;

/// Fetch a note from the MCP backend and open it in the editor.
pub fn open_note_at_path(
    path: String,
    current_path: RwSignal<Option<String>>,
    entries: RwSignal<Vec<EditorEntry>>,
    session: RwSignal<Option<String>>,
) {
    current_path.set(Some(path.clone()));

    let sid = match session.get_untracked() {
        Some(s) => s,
        None => {
            web_sys::console::warn_1(&"MCP session not ready".into());
            return;
        }
    };

    spawn_local(async move {
        match get_note(&sid, &path).await {
            Ok(note) => {
                let display_path = display_note_path(&path);
                let entry = entry_from_note(display_path, &note.text, &note.metadata);
                open_note_in_editor(entries, entry);
            }
            Err(e) => web_sys::console::error_1(&e.into()),
        }
    });
}
