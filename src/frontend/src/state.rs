mod active_panel;

pub use active_panel::ActivePanel;

use crate::constants::DEFAULT_MARKDOWN_PATH;
use crate::default_settings::{
    DEFAULT_EXPORT_HOTKEY_KEY, DEFAULT_IMPORT_HOTKEY_KEY, DEFAULT_NEW_FILE_HOTKEY_KEY,
    DEFAULT_NUMBER_RESULTS_PER_PAGE, DEFAULT_OLLAMA_PORT, DEFAULT_PROMPT_OUTPUT_PATH,
    DEFAULT_SAVE_HOTKEY_KEY, DEFAULT_SIDEBAR_WIDTH, DEFAULT_TOGGLE_MARKDOWN_EDITING_HOTKEY_KEY,
};
use crate::models::{block::EditorEntry, note::NoteFile};
use leptos::prelude::*;

#[derive(Clone)]
pub struct AppState {
    /// Owner of the App component scope. Used to attach signals created later
    /// (e.g. front matter added at runtime) to a long-lived scope so they
    /// aren't disposed when transient handler scopes go away.
    pub root_owner: Owner,
    pub session_id: RwSignal<Option<String>>,
    pub active_panel: RwSignal<Option<ActivePanel>>,
    pub entries: RwSignal<Vec<EditorEntry>>,
    pub current_path: RwSignal<Option<String>>,
    pub search_query: RwSignal<String>,
    pub search_results: RwSignal<Vec<NoteFile>>,
    /// When false, clicking the main markdown block does not enter edit mode.
    pub markdown_editing_enabled: RwSignal<bool>,
    /// Transient user-facing message (e.g. save summary). Cleared automatically.
    pub toast: RwSignal<Option<String>>,
    /// Transient MCP warning messages. Cleared automatically.
    pub warning_toast: RwSignal<Option<String>>,
    /// Transient error message (e.g. save failures). Cleared automatically.
    pub error_toast: RwSignal<Option<String>>,
    /// Bumped after successful note upserts so open file-tree folders re-fetch.
    pub file_tree_epoch: RwSignal<u64>,
    /// Bumped after successful template tree mutations so open folders re-fetch.
    pub template_tree_epoch: RwSignal<u64>,
    /// Sidebar panel width in CSS pixels (preserved while collapsed).
    pub sidebar_width: RwSignal<f64>,
    /// Max search results requested from the backend per query.
    pub number_results_per_page: RwSignal<usize>,
    /// Key letter for save (Ctrl/Meta + this key). Stored lowercase.
    pub save_hotkey_key: RwSignal<char>,
    /// Key letter for new file (Ctrl/Meta + this key). Stored lowercase.
    pub new_file_hotkey_key: RwSignal<char>,
    /// Key letter for import (Ctrl/Meta + this key). Stored lowercase.
    pub import_hotkey_key: RwSignal<char>,
    /// Key letter for export as HTML (Ctrl/Meta + this key). Stored lowercase.
    pub export_hotkey_key: RwSignal<char>,
    /// Key letter for toggling markdown editing (Ctrl/Meta + this key). Stored lowercase.
    pub toggle_markdown_editing_hotkey_key: RwSignal<char>,
    /// Hidden file input used by the Import button and import hotkey.
    pub import_file_input: NodeRef<leptos::html::Input>,
    /// Filename currently being generated during export. `None` when idle.
    pub export_progress: RwSignal<Option<String>>,
    /// User-authored prompt in the Send prompt panel.
    pub prompt_text: RwSignal<String>,
    /// Destination path for the prompt response file.
    pub prompt_output_path: RwSignal<String>,
    /// TCP port of the local Ollama HTTP API.
    pub ollama_port: RwSignal<u16>,
}

impl AppState {
    /// Construct default application state inside a reactive owner.
    pub fn new() -> Self {
        Self {
            root_owner: Owner::current()
                .expect("AppState::new must be called within a reactive Owner scope"),
            session_id: RwSignal::new(None),
            active_panel: RwSignal::new(Some(ActivePanel::Files)),
            entries: RwSignal::new(vec![EditorEntry::new(
                DEFAULT_MARKDOWN_PATH,
                include_str!("../templates/new_markdown.md"),
            )]),
            current_path: RwSignal::new(None),
            search_query: RwSignal::new(String::new()),
            search_results: RwSignal::new(vec![]),
            markdown_editing_enabled: RwSignal::new(true),
            toast: RwSignal::new(None),
            warning_toast: RwSignal::new(None),
            error_toast: RwSignal::new(None),
            file_tree_epoch: RwSignal::new(0),
            template_tree_epoch: RwSignal::new(0),
            sidebar_width: RwSignal::new(DEFAULT_SIDEBAR_WIDTH),
            number_results_per_page: RwSignal::new(DEFAULT_NUMBER_RESULTS_PER_PAGE),
            save_hotkey_key: RwSignal::new(DEFAULT_SAVE_HOTKEY_KEY),
            new_file_hotkey_key: RwSignal::new(DEFAULT_NEW_FILE_HOTKEY_KEY),
            import_hotkey_key: RwSignal::new(DEFAULT_IMPORT_HOTKEY_KEY),
            export_hotkey_key: RwSignal::new(DEFAULT_EXPORT_HOTKEY_KEY),
            toggle_markdown_editing_hotkey_key: RwSignal::new(
                DEFAULT_TOGGLE_MARKDOWN_EDITING_HOTKEY_KEY,
            ),
            import_file_input: NodeRef::new(),
            export_progress: RwSignal::new(None),
            prompt_text: RwSignal::new(String::new()),
            prompt_output_path: RwSignal::new(DEFAULT_PROMPT_OUTPUT_PATH.to_string()),
            ollama_port: RwSignal::new(DEFAULT_OLLAMA_PORT),
        }
    }

    /// Return the CSS class for activity and top-bar icon buttons.
    pub fn activity_btn_class(active: bool) -> &'static str {
        if active {
            "activity-btn active"
        } else {
            "activity-btn"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_MARKDOWN_PATH;
    use crate::default_settings::{
        DEFAULT_EXPORT_HOTKEY_KEY, DEFAULT_IMPORT_HOTKEY_KEY, DEFAULT_NEW_FILE_HOTKEY_KEY,
        DEFAULT_NUMBER_RESULTS_PER_PAGE, DEFAULT_OLLAMA_PORT, DEFAULT_PROMPT_OUTPUT_PATH,
        DEFAULT_SAVE_HOTKEY_KEY, DEFAULT_SIDEBAR_WIDTH, DEFAULT_TOGGLE_MARKDOWN_EDITING_HOTKEY_KEY,
    };
    use leptos::prelude::{GetUntracked, Owner};

    #[test]
    /// Assert the activity button class includes `active` only when selected.
    fn activity_btn_class_marks_the_active_button() {
        assert_eq!(AppState::activity_btn_class(true), "activity-btn active");
        assert_eq!(AppState::activity_btn_class(false), "activity-btn");
    }

    #[test]
    /// Assert AppState::new seeds the default note, panel, and settings.
    fn new_seeds_default_entry_and_settings() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            let entries = state.entries.get_untracked();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].title.path.get_untracked(), DEFAULT_MARKDOWN_PATH);
            assert_eq!(
                entries[0].content.text.get_untracked(),
                include_str!("../templates/new_markdown.md")
            );
            assert_eq!(state.active_panel.get_untracked(), Some(ActivePanel::Files));
            assert!(state.markdown_editing_enabled.get_untracked());
            assert_eq!(
                state.number_results_per_page.get_untracked(),
                DEFAULT_NUMBER_RESULTS_PER_PAGE
            );
            assert_eq!(
                state.save_hotkey_key.get_untracked(),
                DEFAULT_SAVE_HOTKEY_KEY
            );
            assert_eq!(
                state.new_file_hotkey_key.get_untracked(),
                DEFAULT_NEW_FILE_HOTKEY_KEY
            );
            assert_eq!(
                state.import_hotkey_key.get_untracked(),
                DEFAULT_IMPORT_HOTKEY_KEY
            );
            assert_eq!(
                state.export_hotkey_key.get_untracked(),
                DEFAULT_EXPORT_HOTKEY_KEY
            );
            assert_eq!(
                state.toggle_markdown_editing_hotkey_key.get_untracked(),
                DEFAULT_TOGGLE_MARKDOWN_EDITING_HOTKEY_KEY
            );
            assert_eq!(state.sidebar_width.get_untracked(), DEFAULT_SIDEBAR_WIDTH);
            assert_eq!(state.prompt_text.get_untracked(), "");
            assert_eq!(
                state.prompt_output_path.get_untracked(),
                DEFAULT_PROMPT_OUTPUT_PATH
            );
            assert_eq!(state.ollama_port.get_untracked(), DEFAULT_OLLAMA_PORT);
        });
    }
}
