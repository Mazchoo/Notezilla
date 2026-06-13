mod code;
mod graphviz;
mod mermaid;

use code::highlight_code;
use graphviz::render_dot;
use mermaid::render_mermaid;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

enum BlockKind {
    Graphviz,
    Mermaid,
    Code(String), // language token
}

pub fn render_markdown(src: &str) -> String {
    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(src, opts);
    let mut events: Vec<Event> = Vec::new();
    let mut block: Option<BlockKind> = None;
    let mut buf = String::new();

    for event in parser {
        match block {
            Some(_) => match event {
                Event::End(TagEnd::CodeBlock) => {
                    let html_fragment = match block.take().unwrap() {
                        BlockKind::Graphviz => render_dot(&buf).unwrap_or_else(|_| {
                            let escaped = escape_html(&buf);
                            format!("<pre><code>{escaped}</code></pre>")
                        }),
                        BlockKind::Mermaid => render_mermaid(&buf),
                        BlockKind::Code(ref lang) => highlight_code(lang, &buf),
                    };
                    events.push(Event::Html(html_fragment.into()));
                    buf.clear();
                }
                Event::Text(text) => buf.push_str(&text),
                _ => {}
            },
            None => {
                if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) = event {
                    let lang_str = lang.as_ref();
                    match lang_str {
                        "graphviz" => {
                            block = Some(BlockKind::Graphviz);
                            buf.clear();
                            continue;
                        }
                        "mermaid" => {
                            block = Some(BlockKind::Mermaid);
                            buf.clear();
                            continue;
                        }
                        other => {
                            block = Some(BlockKind::Code(other.to_string()));
                            buf.clear();
                            continue;
                        }
                    }
                }
                // Unfenced code blocks — let pulldown-cmark handle them normally.
                events.push(event);
            }
        }
    }

    let mut out = String::with_capacity(src.len() * 2);
    html::push_html(&mut out, events.into_iter());
    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
