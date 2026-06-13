mod graphviz;

use graphviz::render_dot;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

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

