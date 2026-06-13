use crate::state::AppState;
use leptos::*;

/// Removes the [`EditorEntry`](crate::models::block::EditorEntry) whose
/// `title.id` matches `entry_id` from [`AppState::entries`].
///
/// This deletes the entire block group: the divider rendered before it,
/// the file-path title, the optional front-matter block, and the markdown
/// content block are all gone once the entry is removed from the signal.
pub fn delete_entry(state: &AppState, entry_id: u64) {
    state.entries.update(|entries| {
        entries.retain(|e| e.title.id != entry_id);
    });
}
