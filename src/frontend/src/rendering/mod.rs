//! Renders markdown to HTML for the editor and for PDF export.
//!
//! [`render_markdown`] and [`render_markdown_for_pdf`] run the same pipeline
//! for the two [`RenderTarget`]s: math is substituted first, then markdown is
//! parsed, and the constructs this crate renders itself (fenced code, graphviz,
//! mermaid, images, lists) are intercepted and handed to a render.

mod block_kind;
mod html_escape;
mod math_substitution;
mod pdf;
mod renders;
mod svg_attr;
mod svg_path_ends;
mod svg_text_elements;

use block_kind::BlockKind;
pub(crate) use html_escape::escape_html;
use math_substitution::substitute_math;
pub(crate) use pdf::{html_to_pdf_bytes, pdf_rgb_operator};
use pdf::{pdf_list_item_open, PdfListKind};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use renders::{
    CodeRender, GraphvizRender, MermaidRender, MissingImageRender, RenderPdf, RenderTarget,
};
use std::collections::VecDeque;

/// Render markdown to HTML for the editor.
pub fn render_markdown(src: &str) -> String {
    render_markdown_for(src, RenderTarget::Editor)
}

/// Render markdown to HTML for PDF export.
pub fn render_markdown_for_pdf(src: &str) -> String {
    render_markdown_for(src, RenderTarget::Pdf)
}

/// Render markdown to HTML for the given output `target`.
fn render_markdown_for(src: &str, target: RenderTarget) -> String {
    // Math is converted before markdown parsing so `_` and `*` inside an
    // equation are not treated as emphasis, and so display math can become a
    // block-level HTML element.
    let with_math = substitute_math(src, target);

    let mut out = String::with_capacity(with_math.len() * 2);
    html::push_html(
        &mut out,
        InterceptedMarkdown::new(&with_math, markdown_options(), target),
    );
    out
}

/// Return the markdown extensions enabled for every render.
fn markdown_options() -> Options {
    Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
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
    target: RenderTarget,
    pdf_list_stack: Vec<PdfListKind>,
}

impl<'a> InterceptedMarkdown<'a> {
    /// Build a markdown event stream that intercepts code blocks and images.
    fn new(src: &'a str, opts: Options, target: RenderTarget) -> Self {
        Self {
            parser: Parser::new_ext(src, opts).into_offset_iter(),
            src,
            pending: VecDeque::new(),
            block: None,
            in_image: false,
            buf: String::new(),
            depth: 0,
            last_top_end: 0,
            target,
            pdf_list_stack: Vec::new(),
        }
    }

    /// Insert extra `<br>` events for extra blank lines between top-level blocks.
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

    /// Update nesting depth and the end offset of the last top-level event.
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

    /// Render the intercepted code, graphviz, or mermaid block to HTML.
    fn finish_code_block(&mut self) {
        let source = std::mem::take(&mut self.buf);
        let html_fragment = match self.block.take() {
            Some(BlockKind::Graphviz) => self.render(&GraphvizRender, &source),
            Some(BlockKind::Mermaid) => self.render(&MermaidRender, &source),
            Some(BlockKind::Code(lang)) => self.render(&CodeRender::new(lang), &source),
            None => return,
        };
        self.pending.push_back(Event::Html(html_fragment.into()));
    }

    /// Replace the intercepted image with the missing-image placeholder.
    fn finish_image(&mut self) {
        self.in_image = false;
        let alt = std::mem::take(&mut self.buf);
        let html_fragment = self.render(&MissingImageRender, &alt);
        self.pending.push_back(Event::Html(html_fragment.into()));
    }

    /// Render `source` with `render` for this stream's output target.
    fn render<R: RenderPdf>(&self, render: &R, source: &str) -> String {
        render.render_for(self.target, source)
    }

    /// Consume `event` if it opens an intercepted construct; otherwise return it.
    fn try_start_intercepted(&mut self, event: Event<'a>) -> Option<Event<'a>> {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                self.block = Some(BlockKind::from_fence_language(lang.as_ref()));
                self.buf.clear();
                None
            }
            Event::Start(Tag::Image { .. }) => {
                self.in_image = true;
                self.buf.clear();
                None
            }
            other => self.rewrite_pdf_list(other),
        }
    }

    /// Replace `<ul>/<ol>/<li>` so `$…$` can typeset on the same row as the marker.
    fn rewrite_pdf_list(&mut self, event: Event<'a>) -> Option<Event<'a>> {
        if !self.target.rewrites_lists() {
            return Some(event);
        }
        match event {
            Event::Start(Tag::List(start)) => {
                self.pdf_list_stack.push(PdfListKind::new(start));
                Some(Event::Html(r#"<div class="pdf-list">"#.into()))
            }
            Event::Start(Tag::Item) => {
                let marker = match self.pdf_list_stack.last_mut() {
                    Some(kind) => kind.next_marker(),
                    None => PdfListKind::Ul.next_marker(),
                };
                Some(Event::Html(pdf_list_item_open(&marker).into()))
            }
            Event::End(TagEnd::Item) => Some(Event::Html("</div></div></div>".into())),
            Event::End(TagEnd::List(_)) => {
                self.pdf_list_stack.pop();
                Some(Event::Html("</div>".into()))
            }
            other => Some(other),
        }
    }
}

impl<'a> Iterator for InterceptedMarkdown<'a> {
    type Item = Event<'a>;

    /// Yield the next rewritten markdown event.
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

#[cfg(test)]
mod tests {
    use super::{render_markdown, render_markdown_for_pdf};
    use crate::constants::LIST_BULLET;

    #[test]
    /// Assert plain markdown becomes HTML.
    fn renders_plain_markdown() {
        let html = render_markdown("# Title\n\ntext\n");
        assert!(html.contains("<h1>Title</h1>"), "{html}");
        assert!(html.contains("<p>text</p>"), "{html}");
    }

    #[test]
    /// Assert a fenced code block is intercepted by the code render.
    fn fenced_code_uses_the_code_render() {
        let html = render_markdown("```rust\nfn main() {}\n```\n");
        assert!(html.contains(r#"<pre class="code-block">"#), "{html}");
        assert!(html.contains("<span style=\"color:"), "{html}");
    }

    #[test]
    /// Assert a graphviz fence is intercepted by the graphviz render.
    fn graphviz_fence_uses_the_graphviz_render() {
        let html = render_markdown("```graphviz\ndigraph { A -> B }\n```\n");
        assert!(html.contains("<svg"), "{html}");
        assert!(!html.contains("<pre"), "{html}");
    }

    #[test]
    /// Assert a mermaid fence is intercepted by the mermaid render.
    fn mermaid_fence_uses_the_mermaid_render() {
        let html = render_markdown("```mermaid\ngraph LR\n    A --> B\n```\n");
        assert!(html.contains("<svg"), "{html}");
    }

    #[test]
    /// Assert an image becomes the missing-image placeholder with its alt text.
    fn image_uses_the_missing_image_render() {
        let html = render_markdown("![a diagram](diagram.png)\n");
        assert!(html.contains(r#"class="md-image-missing""#), "{html}");
        assert!(html.contains(r#"aria-label="a diagram""#), "{html}");
        assert!(!html.contains("<img"), "{html}");
    }

    #[test]
    /// Assert extra blank lines between blocks become `<br>` elements.
    fn extra_blank_lines_become_line_breaks() {
        let two = render_markdown("one\n\n\n\ntwo\n");
        assert!(two.contains("<br>"), "{two}");
        let three = render_markdown("one\n\n\n\n\ntwo\n");
        assert!(
            three.matches("<br>").count() > two.matches("<br>").count(),
            "each extra blank line adds a break: {three}"
        );
    }

    #[test]
    /// Assert a single blank line between blocks adds no line break.
    fn single_blank_line_adds_no_line_break() {
        let html = render_markdown("one\n\ntwo\n");
        assert!(!html.contains("<br>"), "{html}");
    }

    #[test]
    /// Assert the editor target keeps markdown lists as `<ul>`/`<li>`.
    fn editor_keeps_markdown_lists() {
        let html = render_markdown("- one\n- two\n");
        assert!(html.contains("<ul>") && html.contains("<li>"), "{html}");
        assert!(!html.contains("pdf-li"), "{html}");
    }

    #[test]
    /// Assert the PDF target rewrites unordered lists as flex rows with markers.
    fn pdf_rewrites_unordered_lists_as_flex_rows() {
        let html = render_markdown_for_pdf("- one\n- two\n");
        assert!(!html.contains("<li>") && !html.contains("<ul>"), "{html}");
        assert!(html.contains(r#"class="pdf-list""#), "{html}");
        assert_eq!(html.matches(r#"class="pdf-li""#).count(), 2, "{html}");
        assert_eq!(html.matches(LIST_BULLET).count(), 2, "{html}");
    }

    #[test]
    /// Assert the PDF target numbers ordered list markers from the list start.
    fn pdf_numbers_ordered_lists() {
        let html = render_markdown_for_pdf("3. three\n4. four\n");
        assert!(html.contains(">3. </div>"), "{html}");
        assert!(html.contains(">4. </div>"), "{html}");
    }

    #[test]
    /// Assert nested PDF lists restart numbering per list.
    fn pdf_nested_lists_track_their_own_markers() {
        let html = render_markdown_for_pdf("1. one\n    1. inner\n2. two\n");
        assert!(html.contains(">1. </div>"), "{html}");
        assert!(html.contains(">2. </div>"), "{html}");
        assert_eq!(html.matches(r#"class="pdf-list""#).count(), 2, "{html}");
    }

    #[test]
    /// Assert both targets render the same fenced code HTML.
    fn code_html_is_shared_between_targets() {
        let src = "```rust\nfn main() {}\n```\n";
        assert_eq!(render_markdown(src), render_markdown_for_pdf(src));
    }

    #[test]
    /// Assert only the PDF target emits `data-math` for inline math.
    fn math_html_differs_between_targets() {
        assert!(render_markdown("$x_i$\n").contains("<math"));
        assert!(render_markdown_for_pdf("$x_i$\n").contains("data-math="));
    }
}
