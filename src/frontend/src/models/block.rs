use crate::components::sidebar::file_tree_backend::FileTreeBackend;
use crate::rendering::render_markdown;
use leptos::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

static BLOCK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Allocate the next unique block identifier.
fn next_id() -> u64 {
    BLOCK_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Split `text` into optional YAML front matter and the remaining markdown body.
pub fn split_front_matter(text: &str) -> (Option<String>, String) {
    let norm = text.replace("\r\n", "\n");
    if !norm.starts_with("---\n") {
        return (None, text.to_string());
    }
    let body = &norm[4..]; // after opening ---\n
    if let Some(close_pos) = body.find("\n---\n") {
        let fm = body[..close_pos].to_string();
        let content = body[close_pos + 5..].to_string();
        (Some(fm), content)
    } else if body.ends_with("\n---") {
        // closing --- at end of file with no trailing newline
        let fm = body[..body.len() - 4].to_string();
        (Some(fm), String::new())
    } else {
        (None, text.to_string())
    }
}

/// Holds the raw YAML front matter (without `---` delimiters) for a note.
/// Displayed as a key-value table in view mode; editable as raw YAML in edit mode.
#[derive(Clone, Copy, Debug)]
pub struct FrontMatterBlock {
    pub raw: RwSignal<String>,
    pub focused: RwSignal<bool>,
}

impl FrontMatterBlock {
    /// Create a front-matter block from raw YAML.
    pub fn new(raw: impl Into<String>) -> Self {
        Self {
            raw: RwSignal::new(raw.into()),
            focused: RwSignal::new(false),
        }
    }

    /// Parse raw YAML into `(key, value)` pairs for the view-mode table.
    pub fn parse_fields(raw: &str) -> Vec<(String, String)> {
        raw.lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ':');
                let key = parts.next()?.trim().to_string();
                if key.is_empty() || key.starts_with('-') || key.starts_with(' ') {
                    return None;
                }
                let value = parts.next().unwrap_or("").trim().to_string();
                Some((key, value))
            })
            .collect()
    }
}

/// A title block that displays the file path of the associated markdown content.
/// Rendered as a styled label (distinct from markdown `#` titles).
/// One line; click to edit the path inline.
/// `collapsed` hides the entry's front matter and markdown when true.
#[derive(Clone, Copy, Debug)]
pub struct TitleBlock {
    pub id: u64,
    pub path: RwSignal<String>,
    pub focused: RwSignal<bool>,
    pub collapsed: RwSignal<bool>,
}

impl TitleBlock {
    /// Create a title block for a note path.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            path: RwSignal::new(path.into()),
            focused: RwSignal::new(false),
            collapsed: RwSignal::new(false),
        }
    }
}

/// A single editing unit in the document. The document is a Vec<MarkdownBlock>.
///
/// All fields are reactive signals so only the changed block re-renders.
/// MarkdownBlock is Copy (all fields are Copy signal handles).
#[derive(Clone, Copy, Debug)]
pub struct MarkdownBlock {
    pub text: RwSignal<String>,  // raw markdown source
    pub html: RwSignal<String>,  // cached rendered HTML
    pub focused: RwSignal<bool>, // true while the textarea is active
}

impl MarkdownBlock {
    /// Create a markdown block; HTML starts empty until the editor renders it.
    pub fn new(raw: impl Into<String>) -> Self {
        Self {
            text: RwSignal::new(raw.into()),
            html: RwSignal::new(String::new()),
            focused: RwSignal::new(false),
        }
    }

    /// Re-render markdown to HTML and update the cache.
    pub fn rerender(self) {
        let raw = self.text.get_untracked();
        self.html.set(render_markdown(&raw));
    }
}

/// A unified editor entry: a title block, optional front matter, and markdown content.
/// Adding a new entry always produces a divider + title + (front matter?) + markdown in the UI.
/// `front_matter` is a reactive `RwSignal<Option<FrontMatterBlock>>` so the delete button can
/// remove it without rebuilding the whole entry.
#[derive(Clone, Copy, Debug)]
pub struct EditorEntry {
    pub title: TitleBlock,
    pub front_matter: RwSignal<Option<FrontMatterBlock>>,
    pub content: MarkdownBlock,
    pub backend: FileTreeBackend,
}

impl EditorEntry {
    /// Create an editor entry from a path and markdown body.
    pub fn new(path: impl Into<String>, raw: impl Into<String>) -> Self {
        Self {
            title: TitleBlock::new(path),
            front_matter: RwSignal::new(None),
            content: MarkdownBlock::new(raw),
            backend: FileTreeBackend::Notes,
        }
    }

    /// Create an empty editor entry for a path.
    pub fn empty(path: impl Into<String>) -> Self {
        Self::new(path, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::{GetUntracked, Owner};

    #[test]
    /// Assert YAML between `---` delimiters is split from the markdown body.
    fn split_front_matter_extracts_yaml_and_body() {
        let (fm, body) = split_front_matter("---\ntitle: x\n---\nhello\n");
        assert_eq!(fm.as_deref(), Some("title: x"));
        assert_eq!(body, "hello\n");
    }

    #[test]
    /// Assert a closing `---` at EOF with no trailing newline still yields front matter.
    fn split_front_matter_accepts_eof_closing_delimiter() {
        let (fm, body) = split_front_matter("---\ntitle: x\n---");
        assert_eq!(fm.as_deref(), Some("title: x"));
        assert_eq!(body, "");
    }

    #[test]
    /// Assert missing delimiters leave the original text as the body.
    fn split_front_matter_without_delimiters_is_body() {
        let src = "hello\n---\nnot front matter";
        let (fm, body) = split_front_matter(src);
        assert_eq!(fm, None);
        assert_eq!(body, src);

        let unclosed = "---\ntitle: x\n";
        let (fm, body) = split_front_matter(unclosed);
        assert_eq!(fm, None);
        assert_eq!(body, unclosed);
    }

    #[test]
    /// Assert CRLF files are split using normalised newlines.
    fn split_front_matter_normalises_crlf() {
        let (fm, body) = split_front_matter("---\r\ntitle: x\r\n---\r\nbody");
        assert_eq!(fm.as_deref(), Some("title: x"));
        assert_eq!(body, "body");
    }

    #[test]
    /// Assert parse_fields keeps `key: value` rows and skips list items and blanks.
    fn parse_fields_reads_key_value_lines() {
        let fields =
            FrontMatterBlock::parse_fields("title: Hello\n\n- skip\n: empty\ntags: [a, b]");
        assert_eq!(
            fields,
            vec![
                ("title".into(), "Hello".into()),
                ("tags".into(), "[a, b]".into()),
            ]
        );
        assert_eq!(
            FrontMatterBlock::parse_fields("title:"),
            vec![("title".into(), "".into())]
        );
    }

    #[test]
    /// Assert empty() stores the path and an empty markdown body.
    fn empty_entry_has_path_and_blank_body() {
        let owner = Owner::new();
        owner.with(|| {
            let entry = EditorEntry::empty("./a.md");
            assert_eq!(entry.title.path.get_untracked(), "./a.md");
            assert_eq!(entry.content.text.get_untracked(), "");
            assert!(entry.front_matter.get_untracked().is_none());
        });
    }

    #[test]
    /// Assert successive title blocks allocate distinct ids.
    fn title_blocks_receive_distinct_ids() {
        let owner = Owner::new();
        owner.with(|| {
            let a = TitleBlock::new("a.md");
            let b = TitleBlock::new("b.md");
            assert_ne!(a.id, b.id);
        });
    }

    #[test]
    /// Assert rerender writes HTML derived from the markdown source.
    fn rerender_writes_html_from_markdown() {
        let owner = Owner::new();
        owner.with(|| {
            let block = MarkdownBlock::new("# Hi");
            assert_eq!(block.html.get_untracked(), "");
            block.rerender();
            assert!(
                block.html.get_untracked().contains("Hi"),
                "{}",
                block.html.get_untracked()
            );
        });
    }
}
