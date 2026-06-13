use leptos::*;
use pulldown_cmark::{html, Options, Parser};
use std::sync::atomic::{AtomicU64, Ordering};

static BLOCK_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u64 {
    BLOCK_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// A title block that displays the file path of the associated markdown content.
/// Rendered as a styled label (distinct from markdown `#` titles).
/// One line; click to edit the path inline.
#[derive(Clone, Copy, Debug)]
pub struct TitleBlock {
    pub id: u64,
    pub path: RwSignal<String>,
    pub focused: RwSignal<bool>,
}

impl TitleBlock {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            path: create_rw_signal(path.into()),
            focused: create_rw_signal(false),
        }
    }
}

/// A single editing unit in the document. The document is a Vec<MarkdownBlock>.
///
/// All fields are reactive signals so only the changed block re-renders.
/// MarkdownBlock is Copy (all fields are Copy signal handles).
#[derive(Clone, Copy, Debug)]
pub struct MarkdownBlock {
    pub id: u64,
    pub text: RwSignal<String>,  // raw markdown source
    pub html: RwSignal<String>,  // cached rendered HTML
    pub focused: RwSignal<bool>, // true while the textarea is active
}

impl MarkdownBlock {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw_str = raw.into();
        let rendered = render_markdown(&raw_str);
        Self {
            id: next_id(),
            text: create_rw_signal(raw_str),
            html: create_rw_signal(rendered),
            focused: create_rw_signal(false),
        }
    }

    pub fn empty() -> Self {
        Self::new("")
    }

    /// Re-render markdown → HTML and update the cache.
    /// Call before setting focused=false so the div shows fresh HTML on switch.
    pub fn rerender(self) {
        let raw = self.text.get_untracked();
        self.html.set(render_markdown(&raw));
    }
}

/// A unified editor entry: a title block paired with its markdown content block.
/// Adding a new entry always produces a divider + title + markdown triple in the UI.
#[derive(Clone, Copy, Debug)]
pub struct EditorEntry {
    pub title: TitleBlock,
    pub content: MarkdownBlock,
}

impl EditorEntry {
    pub fn new(path: impl Into<String>, raw: impl Into<String>) -> Self {
        Self {
            title: TitleBlock::new(path),
            content: MarkdownBlock::new(raw),
        }
    }

    pub fn empty(path: impl Into<String>) -> Self {
        Self::new(path, "")
    }
}

pub fn render_markdown(src: &str) -> String {
    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(src, opts);
    let mut out = String::with_capacity(src.len() * 2);
    html::push_html(&mut out, parser);
    out
}
