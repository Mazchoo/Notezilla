use crate::models::{block::EditorEntry, note::SearchResult};
use leptos::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActivePanel {
    Files,
    Search,
}

#[derive(Clone)]
pub struct AppState {
    pub session_id: RwSignal<Option<String>>,
    pub active_panel: RwSignal<Option<ActivePanel>>,
    pub entries: RwSignal<Vec<EditorEntry>>,
    pub current_path: RwSignal<Option<String>>,
    pub search_query: RwSignal<String>,
    pub search_results: RwSignal<Vec<SearchResult>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session_id: create_rw_signal(None),
            active_panel: create_rw_signal(Some(ActivePanel::Files)),
            entries: create_rw_signal(vec![
                EditorEntry::new(
                    "./example_folder/new_markdown.md",
                    "## Example title\nExample text with list\n- list item 1\n- list item 2",
                ),
            ]),
            current_path: create_rw_signal(None),
            search_query: create_rw_signal(String::new()),
            search_results: create_rw_signal(vec![]),
        }
    }
}
