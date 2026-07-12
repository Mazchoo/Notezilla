mod code;
mod graphviz;
mod image;
mod math;
mod mermaid;
mod utils;

use code::highlight_code;
use graphviz::render_dot;
use image::missing_image_html;
use math::substitute_math;
use mermaid::render_mermaid;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::collections::VecDeque;
pub(crate) use utils::escape_html;

enum BlockKind {
    Graphviz,
    Mermaid,
    Code(String), // language token
}

/// Streams parser events through code-block / image interception without
/// buffering the full document in a `Vec<Event>`.
struct InterceptedMarkdown<'a> {
    parser: pulldown_cmark::OffsetIter<'a>,
    src: &'a str,
    pending: VecDeque<Event<'a>>,
    block: Option<BlockKind>,
    in_image: bool,
    buf: String,
    depth: i32,
    last_top_end: usize,
}

impl<'a> InterceptedMarkdown<'a> {
    fn new(src: &'a str, opts: Options) -> Self {
        Self {
            parser: Parser::new_ext(src, opts).into_offset_iter(),
            src,
            pending: VecDeque::new(),
            block: None,
            in_image: false,
            buf: String::new(),
            depth: 0,
            last_top_end: 0,
        }
    }

    fn inject_blank_line_breaks(&mut self, range_start: usize) {
        if self.depth == 0
            && self.block.is_none()
            && !self.in_image
            && self.last_top_end > 0
            && range_start > self.last_top_end
        {
            let gap = &self.src[self.last_top_end..range_start];
            let newlines = gap.matches('\n').count();
            if newlines > 2 {
                for _ in 2..newlines {
                    self.pending.push_back(Event::Html("<br>\n".into()));
                }
            }
        }
    }

    fn track_depth(&mut self, event: &Event<'_>, range_end: usize) {
        match event {
            Event::Start(_) => self.depth += 1,
            Event::End(_) => {
                self.depth -= 1;
                if self.depth == 0 {
                    self.last_top_end = range_end;
                }
            }
            _ if self.depth == 0 => {
                self.last_top_end = range_end;
            }
            _ => {}
        }
    }

    fn finish_code_block(&mut self) {
        let html_fragment = match self.block.take().unwrap() {
            BlockKind::Graphviz => render_dot(&self.buf).unwrap_or_else(|_| {
                let escaped = escape_html(&self.buf);
                format!("<pre><code>{escaped}</code></pre>")
            }),
            BlockKind::Mermaid => render_mermaid(&self.buf),
            BlockKind::Code(ref lang) => highlight_code(lang, &self.buf),
        };
        self.pending.push_back(Event::Html(html_fragment.into()));
        self.buf.clear();
    }

    fn finish_image(&mut self) {
        self.in_image = false;
        let alt = std::mem::take(&mut self.buf);
        self.pending
            .push_back(Event::Html(missing_image_html(&alt).into()));
    }

    /// Returns `None` if `event` opened an intercepted construct (and has
    /// been consumed); otherwise hands the event back unchanged.
    fn try_start_intercepted(&mut self, event: Event<'a>) -> Option<Event<'a>> {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                let lang_str = lang.as_ref();
                self.block = Some(match lang_str {
                    "graphviz" => BlockKind::Graphviz,
                    "mermaid" => BlockKind::Mermaid,
                    other => BlockKind::Code(other.to_string()),
                });
                self.buf.clear();
                None
            }
            Event::Start(Tag::Image { .. }) => {
                self.in_image = true;
                self.buf.clear();
                None
            }
            other => Some(other),
        }
    }
}

impl<'a> Iterator for InterceptedMarkdown<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Some(event);
            }

            let (event, range) = self.parser.next()?;
            self.inject_blank_line_breaks(range.start);
            self.track_depth(&event, range.end);

            if self.block.is_some() {
                match event {
                    Event::End(TagEnd::CodeBlock) => self.finish_code_block(),
                    Event::Text(text) => self.buf.push_str(&text),
                    _ => {}
                }
                continue;
            }

            if self.in_image {
                match event {
                    Event::End(TagEnd::Image) => self.finish_image(),
                    Event::Text(text) | Event::Code(text) => self.buf.push_str(&text),
                    Event::SoftBreak | Event::HardBreak => self.buf.push(' '),
                    _ => {}
                }
                continue;
            }

            match self.try_start_intercepted(event) {
                None => continue,
                Some(event) => return Some(event),
            }
        }
    }
}

pub fn render_markdown(src: &str) -> String {
    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;

    // Convert `$…$` / `$$…$$` to MathML before markdown parsing so `_` / `*`
    // inside equations are not treated as emphasis, and so display math can
    // become a block-level HTML element.
    let with_math = substitute_math(src);

    let mut out = String::with_capacity(with_math.len() * 2);
    html::push_html(&mut out, InterceptedMarkdown::new(&with_math, opts));
    out
}
