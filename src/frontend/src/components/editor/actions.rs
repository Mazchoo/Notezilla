use crate::models::block::{EditorEntry, FrontMatterBlock};
use crate::state::AppState;
use leptos::prelude::*;

/// Remove the editor entry whose `title.id` matches `entry_id`.
pub fn delete_entry(state: &AppState, entry_id: u64) {
    state.entries.update(|entries: &mut Vec<EditorEntry>| {
        entries.retain(|e| e.title.id != entry_id);
    });
}

/// Clear the front matter for the entry identified by `entry_id`.
pub fn delete_front_matter(state: &AppState, entry_id: u64) {
    state.entries.with_untracked(|entries: &Vec<EditorEntry>| {
        if let Some(entry) = entries.iter().find(|e| e.title.id == entry_id) {
            entry.front_matter.set(None);
        }
    });
}

/// Add default front matter (`tags: []`) to the entry identified by `entry_id`.
pub fn add_front_matter(state: &AppState, entry_id: u64) {
    state.entries.with_untracked(|entries: &Vec<EditorEntry>| {
        if let Some(entry) = entries.iter().find(|e| e.title.id == entry_id) {
            let fm = state.root_owner.with(|| FrontMatterBlock::new("tags: []"));
            entry.front_matter.set(Some(fm));
        }
    });
}
