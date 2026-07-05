use crate::models::{block::EditorEntry, note::SearchResult};
use leptos::prelude::*;

const DEFAULT_MARKDOWN_PATH: &str = "./example_folder/new_markdown.md";
const DEFAULT_MARKDOWN_TEMPLATE: &str = include_str!("../templates/new_markdown.md");

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActivePanel {
    Files,
    Search,
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
    pub search_results: RwSignal<Vec<SearchResult>>,
    /// When false, clicking the main markdown block does not enter edit mode.
    pub markdown_editing_enabled: RwSignal<bool>,
    /// Transient user-facing message (e.g. save summary). Cleared automatically.
    pub toast: RwSignal<Option<String>>,
    /// Transient error message (e.g. save failures). Cleared automatically.
    pub error_toast: RwSignal<Option<String>>,
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
            error_toast: RwSignal::new(None),
        }
    }
}
