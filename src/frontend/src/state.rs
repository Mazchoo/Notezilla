use crate::models::{block::MarkdownBlock, note::SearchResult};
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
    pub blocks: RwSignal<Vec<MarkdownBlock>>,
    pub current_path: RwSignal<Option<String>>,
    pub search_query: RwSignal<String>,
    pub search_results: RwSignal<Vec<SearchResult>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            session_id: create_rw_signal(None),
            active_panel: create_rw_signal(Some(ActivePanel::Files)),
            blocks: create_rw_signal(vec![
                MarkdownBlock::new("# Welcome to Notezilla"),
                MarkdownBlock::new(
                    "Click any block to edit. Press **Enter** for a new block, \
                     **Shift+Enter** for a newline within a block.",
                ),
                MarkdownBlock::new(""),
            ]),
            current_path: create_rw_signal(None),
            search_query: create_rw_signal(String::new()),
            search_results: create_rw_signal(vec![]),
        }
    }
}
