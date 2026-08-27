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

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::{GetUntracked, Owner};

    #[test]
    /// Assert delete_entry removes the matching title id and ignores unknown ids.
    fn delete_entry_removes_matching_id() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            let id = state.entries.get_untracked()[0].title.id;
            delete_entry(&state, id);
            assert!(state.entries.get_untracked().is_empty());
            delete_entry(&state, id);
            assert!(state.entries.get_untracked().is_empty());
        });
    }

    #[test]
    /// Assert add_front_matter writes default `tags: []` YAML onto the entry.
    fn add_front_matter_inserts_default_tags() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            let id = state.entries.get_untracked()[0].title.id;
            add_front_matter(&state, id);
            let fm = state.entries.get_untracked()[0]
                .front_matter
                .get_untracked()
                .expect("front matter");
            assert_eq!(fm.raw.get_untracked(), "tags: []");
        });
    }

    #[test]
    /// Assert delete_front_matter clears front matter for the matching entry.
    fn delete_front_matter_clears_the_block() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            let id = state.entries.get_untracked()[0].title.id;
            add_front_matter(&state, id);
            delete_front_matter(&state, id);
            assert!(state.entries.get_untracked()[0]
                .front_matter
                .get_untracked()
                .is_none());
        });
    }
}
