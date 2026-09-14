use super::import::entry_from_note;
use super::save::normalize_note_path;
use crate::components::sidebar::file_tree_backend::FileTreeBackend;
use crate::models::block::EditorEntry;
use crate::models::note::NoteFile;
use crate::state::AppState;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Return whether the entry has an active editor focus that a reload would clobber.
pub fn entry_is_being_edited(entry: &EditorEntry) -> bool {
    if entry.content.focused.get_untracked() || entry.title.focused.get_untracked() {
        return true;
    }
    entry
        .front_matter
        .get_untracked()
        .is_some_and(|fm| fm.focused.get_untracked())
}

/// Bump the notes file-tree epoch and reload unfocused open notes.
pub fn on_file_change_notify(state: &AppState) {
    state.file_tree_epoch.update(|n| *n = n.wrapping_add(1));

    let Some(session_id) = state.session_id.get_untracked() else {
        return;
    };

    let to_reload: Vec<EditorEntry> = state.entries.with_untracked(|list| {
        list.iter()
            .copied()
            .filter(|entry| {
                entry.backend == FileTreeBackend::Notes && !entry_is_being_edited(entry)
            })
            .collect()
    });

    if to_reload.is_empty() {
        return;
    }

    let root_owner = state.root_owner.clone();
    spawn_local(async move {
        for entry in to_reload {
            let path = normalize_note_path(&entry.title.path.get_untracked());
            match FileTreeBackend::Notes.get_file(&session_id, &path).await {
                Ok(note) => apply_fetched_note(entry, &note, &root_owner),
                Err(e) => {
                    web_sys::console::warn_1(&format!("Could not refresh {path}: {e}").into());
                }
            }
        }
    });
}

/// Copy fetched note body and front matter onto `entry` when they differ.
pub fn apply_fetched_note(entry: EditorEntry, note: &NoteFile, root_owner: &Owner) {
    let incoming = root_owner
        .with(|| entry_from_note(entry.title.path.get_untracked(), &note.text, &note.metadata));
    let new_body = incoming.content.text.get_untracked();
    if entry.content.text.get_untracked() != new_body {
        entry.content.text.set(new_body);
        entry.content.rerender();
    }
    let new_fm = incoming
        .front_matter
        .get_untracked()
        .map(|block| block.raw.get_untracked());
    let old_fm = entry
        .front_matter
        .get_untracked()
        .map(|block| block.raw.get_untracked());
    if new_fm == old_fm {
        return;
    }
    match incoming.front_matter.get_untracked() {
        Some(block) => match entry.front_matter.get_untracked() {
            Some(existing) => existing.raw.set(block.raw.get_untracked()),
            None => entry.front_matter.set(Some(block)),
        },
        None => entry.front_matter.set(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::{GetUntracked, Owner};
    use std::collections::HashMap;

    #[test]
    /// Assert a ping bumps the notes tree epoch.
    fn on_file_change_notify_bumps_the_notes_tree_epoch() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            on_file_change_notify(&state);
            assert_eq!(state.file_tree_epoch.get_untracked(), 1);
            assert_eq!(state.template_tree_epoch.get_untracked(), 0);
        });
    }

    #[test]
    /// Assert fetched text replaces the open body and skips an identical body.
    fn apply_fetched_note_updates_body_when_it_changed() {
        let owner = Owner::new();
        owner.with(|| {
            let entry = EditorEntry::new("./a.md", "old");
            let note = NoteFile {
                filename: "a.md".into(),
                text: "new".into(),
                metadata: HashMap::new(),
            };
            apply_fetched_note(entry, &note, &Owner::current().expect("owner"));
            assert_eq!(entry.content.text.get_untracked(), "new");
            assert!(entry.content.html.get_untracked().contains("new"));

            apply_fetched_note(entry, &note, &Owner::current().expect("owner"));
            assert_eq!(entry.content.text.get_untracked(), "new");
        });
    }
}
