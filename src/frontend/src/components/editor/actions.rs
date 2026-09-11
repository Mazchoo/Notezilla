use crate::components::toast::{show_error_toast, show_toast};
use crate::info_messages::{CLIPBOARD_COPY_FAILED_TOAST, MARKDOWN_COPIED_TOAST};
use crate::models::block::{EditorEntry, FrontMatterBlock};
use crate::state::AppState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen_futures::JsFuture;

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

/// Return the raw markdown of the entry identified by `entry_id`.
fn entry_markdown(state: &AppState, entry_id: u64) -> Option<String> {
    state.entries.with_untracked(|entries: &Vec<EditorEntry>| {
        entries
            .iter()
            .find(|e| e.title.id == entry_id)
            .map(|e| e.to_markdown())
    })
}

/// Copy the raw markdown of the entry identified by `entry_id` to the clipboard.
pub fn copy_markdown(state: &AppState, entry_id: u64) {
    let Some(text) = entry_markdown(state, entry_id) else {
        return;
    };
    let Some(window) = web_sys::window() else {
        web_sys::console::error_1(&"Clipboard unavailable: no window".into());
        show_error_toast(state.error_toast, CLIPBOARD_COPY_FAILED_TOAST);
        return;
    };
    let clipboard = window.navigator().clipboard();
    let toast = state.toast;
    let error_toast = state.error_toast;
    spawn_local(async move {
        match JsFuture::from(clipboard.write_text(&text)).await {
            Ok(_) => show_toast(toast, MARKDOWN_COPIED_TOAST),
            Err(e) => {
                web_sys::console::error_1(&e);
                show_error_toast(error_toast, CLIPBOARD_COPY_FAILED_TOAST);
            }
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

    #[test]
    /// Assert entry_markdown returns the file source and ignores unknown ids.
    fn entry_markdown_returns_raw_source() {
        use leptos::prelude::Set;

        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            let entry = EditorEntry::new("./a.md", "body");
            let id = entry.title.id;
            state.entries.set(vec![entry]);
            assert_eq!(entry_markdown(&state, id).as_deref(), Some("body"));

            entry
                .front_matter
                .set(Some(FrontMatterBlock::new("title: x")));
            assert_eq!(
                entry_markdown(&state, id).as_deref(),
                Some("---\ntitle: x\n---\nbody")
            );

            entry.front_matter.set(Some(FrontMatterBlock::new("")));
            assert_eq!(entry_markdown(&state, id).as_deref(), Some("body"));

            assert_eq!(entry_markdown(&state, id.wrapping_add(1)), None);
        });
    }
}
