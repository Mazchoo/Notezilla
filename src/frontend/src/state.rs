use crate::models::{block::EditorEntry, note::SearchResult};
use leptos::prelude::*;

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
}

impl AppState {
    pub fn new() -> Self {
        Self {
            root_owner: Owner::current()
                .expect("AppState::new must be called within a reactive Owner scope"),
            session_id: RwSignal::new(None),
            active_panel: RwSignal::new(Some(ActivePanel::Files)),
            entries: RwSignal::new(vec![
                EditorEntry::new(
                    "./example_folder/new_markdown.md",
                    "## Example title\nExample text with list\n- list item 1\n- list item 2\n\n\n```graphviz\ndigraph {\n    A -> B\n}\n```\n\n\n```mermaid\ngraph LR\n    A[Square Rect] -- Link text --> B((Circle))\n    A --> C(Round Rect)\n    B --> D{Rhombus}\n    C --> D\n```\n\n\n```mermaid\npie title What Voldemort doesn't have?\n\"FRIENDS\" : 2\n\"FAMILY\" : 3\n\"NOSE\" : 4\n```\n\n\n```python\nprint('Hello dude')\n```",
                ),
            ]),
            current_path: RwSignal::new(None),
            search_query: RwSignal::new(String::new()),
            search_results: RwSignal::new(vec![]),
            markdown_editing_enabled: RwSignal::new(true),
        }
    }
}
