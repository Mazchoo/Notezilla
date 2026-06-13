mod graphviz;
mod mermaid;

use graphviz::render_dot;
use mermaid::render_mermaid;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

enum DiagramKind {
    Graphviz,
    Mermaid,
}

pub fn render_markdown(src: &str) -> String {
    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(src, opts);
    let mut events: Vec<Event> = Vec::new();
    let mut diagram: Option<DiagramKind> = None;
    let mut diagram_buf = String::new();

    for event in parser {
        match diagram {
            Some(_) => match event {
                Event::End(TagEnd::CodeBlock) => {
                    let svg = match diagram.take().unwrap() {
                        DiagramKind::Graphviz => render_dot(&diagram_buf).unwrap_or_else(|_| {
                            let escaped = escape_html(&diagram_buf);
                            format!("<pre><code>{escaped}</code></pre>")
                        }),
                        DiagramKind::Mermaid => render_mermaid(&diagram_buf),
                    };
                    events.push(Event::Html(svg.into()));
                    diagram_buf.clear();
                }
                Event::Text(text) => diagram_buf.push_str(&text),
                _ => {}
            },
            None => {
                if let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) = event {
                    match lang.as_ref() {
                        "graphviz" => {
                            diagram = Some(DiagramKind::Graphviz);
                            diagram_buf.clear();
                            continue;
                        }
                        "mermaid" => {
                            diagram = Some(DiagramKind::Mermaid);
                            diagram_buf.clear();
                            continue;
                        }
                        _ => {}
                    }
                }
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
