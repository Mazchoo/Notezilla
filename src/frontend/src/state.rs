use crate::models::{block::EditorEntry, note::NoteFile};
use crate::settings::{
    DEFAULT_EXPORT_HOTKEY_KEY, DEFAULT_IMPORT_HOTKEY_KEY, DEFAULT_NEW_FILE_HOTKEY_KEY,
    DEFAULT_NUMBER_RESULTS_PER_PAGE, DEFAULT_SAVE_HOTKEY_KEY,
    DEFAULT_TOGGLE_MARKDOWN_EDITING_HOTKEY_KEY,
};
use leptos::prelude::*;

const DEFAULT_MARKDOWN_PATH: &str = "./example_folder/new_markdown.md";
const DEFAULT_MARKDOWN_TEMPLATE: &str = include_str!("../templates/new_markdown.md");

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActivePanel {
    Files,
    Search,
    Settings,
}

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
    /// Sidebar panel width in CSS pixels (preserved while collapsed).
    pub sidebar_width: RwSignal<f64>,
    /// Max search results requested from the backend per query.
    pub number_results_per_page: RwSignal<usize>,
    /// Key letter for save (Ctrl/Meta + this key). Stored lowercase.
    pub save_hotkey_key: RwSignal<String>,
    /// Key letter for new file (Ctrl/Meta + this key). Stored lowercase.
    pub new_file_hotkey_key: RwSignal<String>,
    /// Key letter for import (Ctrl/Meta + this key). Stored lowercase.
    pub import_hotkey_key: RwSignal<String>,
    /// Key letter for export as HTML (Ctrl/Meta + this key). Stored lowercase.
    pub export_hotkey_key: RwSignal<String>,
    /// Key letter for toggling markdown editing (Ctrl/Meta + this key). Stored lowercase.
    pub toggle_markdown_editing_hotkey_key: RwSignal<String>,
    /// Hidden file input used by the Import button and import hotkey.
    pub import_file_input: NodeRef<leptos::html::Input>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            root_owner: Owner::current()
                .expect("AppState::new must be called within a reactive Owner scope"),
            session_id: RwSignal::new(None),
            active_panel: RwSignal::new(Some(ActivePanel::Files)),
            entries: RwSignal::new(vec![EditorEntry::new(
                DEFAULT_MARKDOWN_PATH,
                DEFAULT_MARKDOWN_TEMPLATE,
            )]),
            current_path: RwSignal::new(None),
            search_query: RwSignal::new(String::new()),
            search_results: RwSignal::new(vec![]),
            markdown_editing_enabled: RwSignal::new(true),
            toast: RwSignal::new(None),
            warning_toast: RwSignal::new(None),
            error_toast: RwSignal::new(None),
            file_tree_epoch: RwSignal::new(0),
            sidebar_width: RwSignal::new(250.0),
            number_results_per_page: RwSignal::new(DEFAULT_NUMBER_RESULTS_PER_PAGE),
            save_hotkey_key: RwSignal::new(DEFAULT_SAVE_HOTKEY_KEY.to_string()),
            new_file_hotkey_key: RwSignal::new(DEFAULT_NEW_FILE_HOTKEY_KEY.to_string()),
            import_hotkey_key: RwSignal::new(DEFAULT_IMPORT_HOTKEY_KEY.to_string()),
            export_hotkey_key: RwSignal::new(DEFAULT_EXPORT_HOTKEY_KEY.to_string()),
            toggle_markdown_editing_hotkey_key: RwSignal::new(
                DEFAULT_TOGGLE_MARKDOWN_EDITING_HOTKEY_KEY.to_string(),
            ),
            import_file_input: NodeRef::new(),
        }
    }

    /// Class for activity/top-bar icon buttons; appends `active` when selected.
    pub fn activity_btn_class(active: bool) -> &'static str {
        if active {
            "activity-btn active"
        } else {
            "activity-btn"
        }
    }
}
