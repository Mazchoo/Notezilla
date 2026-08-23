//! Tracks list style while rewriting markdown lists for PDF export.

/// Unordered list, or an ordered list with the next marker number.
pub(super) enum PdfListKind {
    Ul,
    Ol { next: u64 },
}
