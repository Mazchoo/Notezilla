//! Renders markdown to HTML for the editor and for PDF export.
//!
//! [`render_markdown`] and [`render_markdown_for_pdf`] run the same pipeline
//! for the two [`RenderTarget`]s: math is substituted first, then markdown is
//! parsed, and the constructs this crate renders itself (fenced code, graphviz,
//! mermaid, quizzes, flash cards, images, lists) are intercepted and handed to a render.

mod block_kind;
mod heading_slug;
mod html_escape;
mod math_substitution;
mod pdf;
mod renders;
mod source_map;
mod svg_attr;
mod svg_path_ends;
mod svg_text_elements;

use block_kind::BlockKind;
use heading_slug::HeadingSlugs;
pub(crate) use html_escape::{escape_html, escape_html_comment, escape_svg_text};
use math_substitution::{substitute_math, substitute_math_mapped};
pub(crate) use pdf::{html_to_pdf_bytes, pdf_rgb_operator};
use pdf::{pdf_list_item_open, PdfListKind};
use pulldown_cmark::{
    html, CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use renders::{
    CodeRender, FlashcardsRender, GraphvizRender, MermaidRender, MissingImageRender, QuizRender,
    RenderPdf, RenderTarget,
};
use source_map::SourceMap;
pub(crate) use source_map::{
    resolve_source_offset, source_offset_at_textarea, textarea_text_before, SourceKind,
};
use std::collections::VecDeque;
use std::ops::Range;

/// Kind of fenced diagram that renders to a standalone SVG file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagramKind {
    Graphviz,
    Mermaid,
}

impl DiagramKind {
    /// Return the fence language token used in export filenames.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Graphviz => "graphviz",
            Self::Mermaid => "mermaid",
        }
    }

    /// Render `source` to an SVG string.
    pub fn render_svg(self, source: &str) -> Result<String, String> {
        match self {
            Self::Graphviz => GraphvizRender.render_svg(source),
            Self::Mermaid => MermaidRender.render_svg(source),
        }
    }
}

/// Render markdown to HTML for the editor.
pub fn render_markdown(src: &str) -> String {
    render_markdown_for(src, RenderTarget::Editor)
}

/// Render markdown to editor HTML tagged with markdown source offsets.
///
/// Text runs, inline code, fences, images and math carry `data-src`, so a
/// click on the rendered note can be resolved to a caret position in `src`.
pub fn render_markdown_with_source_map(src: &str) -> String {
    let (with_math, offsets) = substitute_math_mapped(src);

    let mut out = String::with_capacity(with_math.len() * 2);
    html::push_html(
        &mut out,
        InterceptedMarkdown::new(&with_math, markdown_options(), RenderTarget::Editor)
            .with_source_map(SourceMap::new(src, offsets)),
    );
    out
}

/// Render markdown to HTML for PDF export.
pub fn render_markdown_for_pdf(src: &str) -> String {
    render_markdown_for(src, RenderTarget::Pdf)
}

/// Return mermaid and graphviz fence sources in `src`, in document order.
pub fn diagram_fences(src: &str) -> Vec<(DiagramKind, String)> {
    let mut out = Vec::new();
    let mut kind = None;
    let mut buf = String::new();

    for (event, _) in Parser::new_ext(src, markdown_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                kind = match BlockKind::from_fence_language(lang.as_ref()) {
                    BlockKind::Graphviz => Some(DiagramKind::Graphviz),
                    BlockKind::Mermaid => Some(DiagramKind::Mermaid),
                    BlockKind::Quiz | BlockKind::Flashcards | BlockKind::Code(_) => None,
                };
                buf.clear();
            }
            Event::Text(text) if kind.is_some() => buf.push_str(&text),
            Event::End(TagEnd::CodeBlock) => {
                if let Some(diagram) = kind.take() {
                    out.push((diagram, std::mem::take(&mut buf)));
                }
            }
            _ => {}
        }
    }
    out
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
    /// Parser offset where the fence or image being intercepted starts.
    intercept_start: usize,
    source_map: Option<SourceMap<'a>>,
    /// Events of the heading being given an anchor id, held until its end.
    heading_queue: VecDeque<Event<'a>>,
    /// Plain text of the heading being buffered, used to derive its id.
    heading_text: Option<String>,
    heading_slugs: HeadingSlugs,
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
            intercept_start: 0,
            source_map: None,
            heading_queue: VecDeque::new(),
            heading_text: None,
            heading_slugs: HeadingSlugs::default(),
        }
    }

    /// Tag the emitted HTML with offsets into the original markdown.
    fn with_source_map(mut self, source_map: SourceMap<'a>) -> Self {
        self.source_map = Some(source_map);
        self
    }

    /// Tag a rendered fence or image with the source offset it started at.
    fn tag_block(&self, fragment: String) -> String {
        match &self.source_map {
            Some(map) => map.annotate_block(fragment, self.intercept_start),
            None => fragment,
        }
    }

    /// Wrap a text or inline-code event in an element tagged with its source offset.
    fn tag_text(&self, event: Event<'a>, range: Range<usize>) -> Event<'a> {
        let Some(map) = &self.source_map else {
            return event;
        };
        let tagged = match &event {
            Event::Text(text) => map.text_element("span", text, self.src, range, false),
            Event::Code(text) => map.text_element("code", text, self.src, range, true),
            _ => None,
        };
        tagged.map_or(event, |html| Event::Html(html.into()))
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

    /// Render the intercepted code, graphviz, mermaid, quiz, or flash card block to HTML.
    fn finish_code_block(&mut self) {
        let source = std::mem::take(&mut self.buf);
        let html_fragment = match self.block.take() {
            Some(BlockKind::Graphviz) => self.render(&GraphvizRender, &source),
            Some(BlockKind::Mermaid) => self.render(&MermaidRender, &source),
            Some(BlockKind::Quiz) => self.render(&QuizRender, &source),
            Some(BlockKind::Flashcards) => self.render(&FlashcardsRender, &source),
            Some(BlockKind::Code(lang)) => self.render(&CodeRender::new(lang), &source),
            None => return,
        };
        let html_fragment = self.tag_block(html_fragment);
        self.pending.push_back(Event::Html(html_fragment.into()));
    }

    /// Replace the intercepted image with the missing-image placeholder.
    fn finish_image(&mut self) {
        self.in_image = false;
        let alt = std::mem::take(&mut self.buf);
        let html_fragment = self.tag_block(self.render(&MissingImageRender, &alt));
        self.pending.push_back(Event::Html(html_fragment.into()));
    }

    /// Buffer a heading's events and return its start tag carrying an anchor id.
    fn anchor_heading(
        &mut self,
        level: HeadingLevel,
        classes: Vec<CowStr<'a>>,
        attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)>,
    ) -> Event<'a> {
        self.heading_text = Some(String::new());
        while let Some(event) = self.next_rewritten() {
            let end = matches!(event, Event::End(TagEnd::Heading(_)));
            self.heading_queue.push_back(event);
            if end {
                break;
            }
        }
        let text = self.heading_text.take().unwrap_or_default();
        Event::Start(Tag::Heading {
            level,
            id: Some(self.heading_slugs.next(&text).into()),
            classes,
            attrs,
        })
    }

    /// Add the text of `event` to the heading being buffered, if any.
    fn collect_heading_text(&mut self, event: &Event<'_>) {
        if let (Some(heading), Event::Text(text) | Event::Code(text)) =
            (self.heading_text.as_mut(), event)
        {
            heading.push_str(text);
        }
    }

    /// Render `source` with `render` for this stream's output target.
    fn render<R: RenderPdf>(&self, render: &R, source: &str) -> String {
        render.render_for(self.target, source)
    }

    /// Consume `event` if it opens an intercepted construct; otherwise return it.
    fn try_start_intercepted(&mut self, event: Event<'a>, start: usize) -> Option<Event<'a>> {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                self.block = Some(BlockKind::from_fence_language(lang.as_ref()));
                self.intercept_start = start;
                self.buf.clear();
                None
            }
            Event::Start(Tag::Image { .. }) => {
                self.in_image = true;
                self.intercept_start = start;
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

    /// Yield the next markdown event with code blocks, images and lists rewritten.
    fn next_rewritten(&mut self) -> Option<Event<'a>> {
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

            match self.try_start_intercepted(event, range.start) {
                None => continue,
                Some(event) => {
                    self.collect_heading_text(&event);
                    return Some(self.tag_text(event, range));
                }
            }
        }
    }
}

impl<'a> Iterator for InterceptedMarkdown<'a> {
    type Item = Event<'a>;

    /// Yield the next rewritten markdown event, giving headings anchor ids.
    fn next(&mut self) -> Option<Event<'a>> {
        if let Some(event) = self.heading_queue.pop_front() {
            return Some(event);
        }
        match self.next_rewritten()? {
            Event::Start(Tag::Heading {
                level,
                id: None,
                classes,
                attrs,
            }) => Some(self.anchor_heading(level, classes, attrs)),
            event => Some(event),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        diagram_fences, render_markdown, render_markdown_for_pdf, render_markdown_with_source_map,
        DiagramKind,
    };
    use crate::constants::LIST_BULLET;

    /// Return the `data-src` offset tagged on the element whose text is `text`.
    fn tagged_offset(html: &str, text: &str) -> usize {
        let close = html
            .find(&format!(">{text}</"))
            .unwrap_or_else(|| panic!("no element holding {text:?}: {html}"));
        let open = html[..close].rfind('<').expect("opening tag");
        let attr = html[open..close]
            .split("data-src=\"")
            .nth(1)
            .unwrap_or_else(|| panic!("element holding {text:?} is untagged: {html}"));
        attr[..attr.find('"').expect("closing quote")]
            .parse()
            .expect("numeric offset")
    }

    #[test]
    /// Assert every text run is tagged with where it starts in the source.
    fn source_map_tags_text_runs_with_their_offsets() {
        let src = "# Title\n\nsome **bold** and `code` here\n\n- item one\n";
        let html = render_markdown_with_source_map(src);
        for text in [
            "Title", "some ", "bold", " and ", "code", " here", "item one",
        ] {
            assert_eq!(
                tagged_offset(&html, text),
                src.find(text).unwrap(),
                "{text}"
            );
        }
    }

    #[test]
    /// Assert text after math maps to the original source, not the substituted one.
    fn source_map_survives_math_substitution() {
        let src = "before $x_i$ after\n\n$$\ny^2\n$$\n\nlast line\n";
        let html = render_markdown_with_source_map(src);
        assert_eq!(tagged_offset(&html, " after"), src.find(" after").unwrap());
        assert_eq!(tagged_offset(&html, "last line"), src.find("last").unwrap());
        assert!(
            html.contains(r#"<math data-src="7" data-src-kind="atom""#),
            "{html}"
        );
        let block = format!(
            r#"<div data-src="{}" data-src-kind="atom" class="math-block">"#,
            src.find("$$").unwrap()
        );
        assert!(html.contains(&block), "{html}");
        assert!(
            !html.contains("<mi><span"),
            "math text stays untagged: {html}"
        );
    }

    #[test]
    /// Assert a code fence maps by line from the line after its opening fence.
    fn source_map_tags_fences_and_diagrams() {
        let src = "a\n\n```rust\nfn main() {}\n```\n\n```mermaid\ngraph LR\n    A --> B\n```\n";
        let html = render_markdown_with_source_map(src);
        let body = src.find("fn main").unwrap();
        assert!(
            html.contains(&format!(
                r#"<pre data-src="{body}" data-src-kind="lines" class="code-block">"#
            )),
            "{html}"
        );
        let fence = src.find("```mermaid").unwrap();
        assert!(
            html.contains(&format!(r#"data-src="{fence}" data-src-kind="atom""#)),
            "{html}"
        );
    }

    #[test]
    /// Assert text that differs from its source (an entity) maps as a whole.
    fn source_map_marks_non_verbatim_text_as_atom() {
        let html = render_markdown_with_source_map("a &amp; b\n");
        assert!(
            html.contains(r#"<span data-src="2" data-src-kind="atom">&amp;</span>"#),
            "{html}"
        );
    }

    #[test]
    /// Assert the plain editor render carries no source tags.
    fn plain_render_has_no_source_tags() {
        let html = render_markdown("# Title\n\n`code` $x$\n");
        assert!(!html.contains("data-src"), "{html}");
    }

    #[test]
    /// Assert markdown on the line right after display math is still parsed.
    fn line_after_display_math_is_markdown() {
        for src in [
            "$$x^2$$\n* **Huber Loss**\n",
            "$$\nx^2\n$$\r\n* **Huber Loss**\r\n",
        ] {
            let html = render_markdown(src);
            assert!(html.contains("<strong>Huber Loss</strong>"), "{html}");
            assert!(!html.contains("**"), "{html}");
        }
    }

    #[test]
    /// Assert a `cases` environment renders as MathML, not an error.
    fn cases_environment_renders() {
        let html = render_markdown(
            r"$$\rho(r) = \begin{cases} \frac{1}{2} r^2 & |r| \le k \\ k|r| - \frac{1}{2}k^2 & |r| > k \end{cases}$$",
        );
        assert!(!html.contains("math-error"), "{html}");
        assert!(html.contains("<mtable"), "{html}");
        assert_eq!(html.matches("<mtr>").count(), 2, "{html}");
        assert!(html.contains("≤"), "{html}");
    }

    #[test]
    /// Assert headings get anchor ids from their text, with or without source tags.
    fn headings_carry_anchor_ids() {
        let src = "## 3. Flowcharts (`flowchart` / `graph`)

## Notes

## Notes
";
        for html in [render_markdown(src), render_markdown_with_source_map(src)] {
            assert!(
                html.contains(r#"<h2 id="3-flowcharts-flowchart--graph">"#),
                "{html}"
            );
            assert!(html.contains(r#"<h2 id="notes">"#), "{html}");
            assert!(html.contains(r#"<h2 id="notes-1">"#), "{html}");
        }
    }

    #[test]
    /// Assert plain markdown becomes HTML.
    fn renders_plain_markdown() {
        let html = render_markdown("# Title\n\ntext\n");
        assert!(html.contains(r#"<h1 id="title">Title</h1>"#), "{html}");
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
    /// Assert a same-line `~~~…~~~` wrap is inline code, not an empty fence.
    fn tilde_span_is_inline_code_not_an_empty_fence() {
        let html = render_markdown("~~~$not_math$~~~\n");
        assert!(html.contains("$not_math$"), "{html}");
        assert!(html.contains("<code>"), "{html}");
        assert!(!html.contains("<pre"), "{html}");
    }

    #[test]
    /// Assert an unclosed tilde opener is not a fenced code block.
    fn unclosed_tilde_block_is_not_a_fenced_block() {
        let html = render_markdown("~~~\na \n");
        assert!(!html.contains("<pre"), "{html}");
    }

    #[test]
    /// Assert an unclosed same-line tilde wrap fences the rest of the line.
    fn unclosed_same_line_tilde_wraps_the_rest() {
        let html = render_markdown("~~~ a");
        assert!(html.contains("<code>"), "{html}");
        assert!(html.contains("a"), "{html}");
        assert!(!html.contains("<pre"), "{html}");
    }

    #[test]
    /// Assert a graphviz fence is intercepted by the graphviz render.
    fn graphviz_fence_uses_the_graphviz_render() {
        let html = render_markdown("```graphviz\ndigraph { A -> B }\n```\n");
        assert!(html.contains("<svg"), "{html}");
        assert!(!html.contains("<pre"), "{html}");
    }

    #[test]
    /// Assert ` ```graphviz extra ` still uses the graphviz render.
    fn graphviz_info_string_suffix_is_ignored() {
        let html = render_markdown("```graphviz extra_info\ndigraph { A -> B }\n```\n");
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
    /// Assert a quiz fence is intercepted by the quiz render.
    fn quiz_fence_uses_the_quiz_render() {
        let html = render_markdown("```quiz\nQuestion\nAnswer\n  + yes\n```\n");
        assert!(html.contains(r#"<div class="quiz">"#), "{html}");
        assert!(!html.contains("<pre"), "{html}");
    }

    #[test]
    /// Assert a flashcards fence is intercepted by the flash card render.
    fn flashcards_fence_uses_the_flashcards_render() {
        let html = render_markdown("```flashcards\nQuestion\nAnswer\n```\n");
        assert!(html.contains(r#"<form class="flashcards""#), "{html}");
        assert!(!html.contains("<pre"), "{html}");
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

    #[test]
    /// Assert mermaid and graphviz fences are collected in document order.
    fn diagram_fences_keeps_document_order_and_skips_code() {
        let src = "```rust\nfn main() {}\n```\n\
```mermaid\ngraph LR\n    A --> B\n```\n\
```graphviz\ndigraph { A -> B }\n```\n\
```mermaid\nnot a diagram\n```\n";
        let fences = diagram_fences(src);
        assert_eq!(fences.len(), 3);
        assert_eq!(fences[0].0, DiagramKind::Mermaid);
        assert_eq!(fences[1].0, DiagramKind::Graphviz);
        assert_eq!(fences[2].0, DiagramKind::Mermaid);
        assert!(fences[0].1.contains("graph LR"), "{}", fences[0].1);
        assert!(fences[1].1.contains("digraph"), "{}", fences[1].1);
        assert_eq!(fences[0].0.as_str(), "mermaid");
        assert_eq!(fences[1].0.as_str(), "graphviz");
        assert!(fences[0].0.render_svg(&fences[0].1).is_ok());
        assert!(fences[1].0.render_svg(&fences[1].1).is_ok());
        assert!(fences[2].0.render_svg(&fences[2].1).is_err());
    }
}
