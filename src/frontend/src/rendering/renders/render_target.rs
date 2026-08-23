//! Selects which HTML form a render produces.

/// Output target of a render.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RenderTarget {
    /// HTML for the editor preview, styled by `index.html`.
    Editor,
    /// HTML for ironpress PDF conversion, styled by `templates/export-pdf.html`.
    Pdf,
}

impl RenderTarget {
    /// Return whether markdown lists must be rewritten as flex rows.
    ///
    /// Ironpress `<li>` skips `data-math` spans, so PDF export lays lists out
    /// as flex rows whose body is a block that typesets inline math.
    pub fn rewrites_lists(self) -> bool {
        matches!(self, RenderTarget::Pdf)
    }
}

#[cfg(test)]
mod tests {
    use super::RenderTarget;

    #[test]
    /// Assert only the PDF target rewrites markdown lists.
    fn only_pdf_rewrites_lists() {
        assert!(RenderTarget::Pdf.rewrites_lists());
        assert!(!RenderTarget::Editor.rewrites_lists());
    }
}
