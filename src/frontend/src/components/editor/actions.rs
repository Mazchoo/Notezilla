use crate::models::block::{EditorEntry, FrontMatterBlock};
use crate::state::AppState;
use leptos::prelude::*;

/// Removes the [`EditorEntry`](crate::models::block::EditorEntry) whose
/// `title.id` matches `entry_id` from [`AppState::entries`].
///
/// This deletes the entire block group: the divider rendered before it,
/// the file-path title, the optional front-matter block, and the markdown
/// content block are all gone once the entry is removed from the signal.
pub fn delete_entry(state: &AppState, entry_id: u64) {
    state.entries.update(|entries: &mut Vec<EditorEntry>| {
        entries.retain(|e| e.title.id != entry_id);
    });
}

/// Clears the front matter for the entry identified by `entry_id`.
/// Sets the entry's `front_matter` signal to `None`, removing it from the UI.
pub fn delete_front_matter(state: &AppState, entry_id: u64) {
    state.entries.with_untracked(|entries: &Vec<EditorEntry>| {
        if let Some(entry) = entries.iter().find(|e| e.title.id == entry_id) {
            entry.front_matter.set(None);
        }
    });
}

/// Adds default front matter (`tags: []`) to the entry identified by `entry_id`.
/// Only has an effect when the entry currently has no front matter.
pub fn add_front_matter(state: &AppState, entry_id: u64) {
    state.entries.with_untracked(|entries: &Vec<EditorEntry>| {
        if let Some(entry) = entries.iter().find(|e| e.title.id == entry_id) {
            entry
                .front_matter
                .set(Some(FrontMatterBlock::new("tags: []")));
        }
    });
}
