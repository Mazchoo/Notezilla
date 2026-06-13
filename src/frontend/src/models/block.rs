use leptos::*;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::sync::atomic::{AtomicU64, Ordering};

static BLOCK_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u64 {
    BLOCK_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Splits `text` into an optional YAML front matter string and the remaining markdown body.
/// Expects front matter wrapped in `---` delimiters at the very start of the file.
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
    pub id: u64,
    pub raw: RwSignal<String>,
    pub focused: RwSignal<bool>,
}

impl FrontMatterBlock {
    pub fn new(raw: impl Into<String>) -> Self {
        Self {
            id: next_id(),
            raw: create_rw_signal(raw.into()),
            focused: create_rw_signal(false),
        }
    }

    /// Parses the raw YAML into `(key, value)` pairs for display.
    /// Handles simple `key: value` lines; skips blank lines and list continuations.
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

/// A unified editor entry: a title block, optional front matter, and markdown content.
/// Adding a new entry always produces a divider + title + (front matter?) + markdown in the UI.
/// `front_matter` is a reactive `RwSignal<Option<FrontMatterBlock>>` so the delete button can
/// remove it without rebuilding the whole entry.
#[derive(Clone, Copy, Debug)]
pub struct EditorEntry {
    pub title: TitleBlock,
    pub front_matter: RwSignal<Option<FrontMatterBlock>>,
    pub content: MarkdownBlock,
}

impl EditorEntry {
    pub fn new(path: impl Into<String>, raw: impl Into<String>) -> Self {
        Self {
            title: TitleBlock::new(path),
            front_matter: create_rw_signal(Some(FrontMatterBlock::new("tags: []"))),
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
    let mut events: Vec<Event> = Vec::new();
    let mut in_graphviz = false;
    let mut dot_buf = String::new();

    for event in parser {
        if in_graphviz {
            match event {
                Event::End(TagEnd::CodeBlock) => {
                    in_graphviz = false;
                    let svg = render_dot(&dot_buf).unwrap_or_else(|_| {
                        let escaped = dot_buf
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;");
                        format!("<pre><code>{escaped}</code></pre>")
                    });
                    events.push(Event::Html(svg.into()));
                }
                Event::Text(text) => dot_buf.push_str(&text),
                _ => {}
            }
        } else {
            let is_graphviz = matches!(
                &event,
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                    if lang.as_ref() == "graphviz"
            );
            if is_graphviz {
                in_graphviz = true;
                dot_buf.clear();
            } else {
                events.push(event);
            }
        }
    }

    let mut out = String::with_capacity(src.len() * 2);
    html::push_html(&mut out, events.into_iter());
    out
}

fn render_dot(dot: &str) -> Result<String, String> {
    use layout::backends::svg::SVGWriter;
    use layout::gv::{DotParser, GraphBuilder};

    let mut parser = DotParser::new(dot);
    let tree = parser
        .process()
        .map_err(|e| format!("{e:?}"))?;
    let mut builder = GraphBuilder::new();
    builder.visit_graph(&tree);
    let mut vg = builder.get();
    let mut svg = SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);
    Ok(svg.finalize())
}
