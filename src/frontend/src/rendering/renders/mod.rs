//! Renders for the markdown constructs the editor handles itself.
//!
//! Every construct that this crate renders (fenced code, graphviz, mermaid,
//! math, missing images) is expressed as a unit struct or small value type
//! implementing [`Render`], so callers select a construct and render it the
//! same way. [`RenderPdf`] carries the second output form: HTML that ironpress
//! converts to PDF, which most constructs share with the editor.

mod code;
mod graphviz;
mod graphviz_label;
mod math;
mod mermaid;
mod mermaid_arrow;
mod mermaid_label;
mod missing_image;
mod render_error;
mod render_target;

pub(super) use code::CodeRender;
pub(super) use graphviz::GraphvizRender;
pub(super) use math::MathRender;
pub(super) use mermaid::MermaidRender;
pub(super) use missing_image::MissingImageRender;
pub(super) use render_target::RenderTarget;

/// Renders one markdown construct to an HTML fragment for the editor.
pub trait Render {
    /// Return the editor HTML for `source`.
    ///
    /// Rendering never fails: a render that cannot process `source` returns
    /// fallback HTML showing the source text instead.
    fn render(&self, source: &str) -> String;
}

/// Renders one markdown construct to an HTML fragment for PDF export.
pub trait RenderPdf: Render {
    /// Return the PDF-export HTML for `source`.
    ///
    /// Defaults to [`Render::render`]. A construct overrides this only when
    /// ironpress needs different markup than the browser, as math does.
    fn render_pdf(&self, source: &str) -> String {
        self.render(source)
    }

    /// Return the HTML for `source` for the given output `target`.
    fn render_for(&self, target: RenderTarget, source: &str) -> String {
        match target {
            RenderTarget::Editor => self.render(source),
            RenderTarget::Pdf => self.render_pdf(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Render, RenderPdf, RenderTarget};

    struct SharedRender;

    impl Render for SharedRender {
        fn render(&self, source: &str) -> String {
            format!("editor:{source}")
        }
    }

    impl RenderPdf for SharedRender {}

    struct SplitRender;

    impl Render for SplitRender {
        fn render(&self, source: &str) -> String {
            format!("editor:{source}")
        }
    }

    impl RenderPdf for SplitRender {
        fn render_pdf(&self, source: &str) -> String {
            format!("pdf:{source}")
        }
    }

    #[test]
    /// Assert a render without a PDF override emits its editor HTML for both targets.
    fn pdf_defaults_to_editor_html() {
        assert_eq!(SharedRender.render_pdf("a"), "editor:a");
        assert_eq!(SharedRender.render_for(RenderTarget::Pdf, "a"), "editor:a");
        assert_eq!(
            SharedRender.render_for(RenderTarget::Editor, "a"),
            "editor:a"
        );
    }

    #[test]
    /// Assert `render_for` selects the overridden PDF HTML for the PDF target.
    fn render_for_selects_target_html() {
        assert_eq!(
            SplitRender.render_for(RenderTarget::Editor, "a"),
            "editor:a"
        );
        assert_eq!(SplitRender.render_for(RenderTarget::Pdf, "a"), "pdf:a");
    }
}
