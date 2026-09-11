use crate::models::block::EditorEntry;

/// Convert one editor entry to a downloadable file. Return toast text on failure.
pub trait ExportOne: Fn(EditorEntry, &str) -> Option<String> + 'static {}

impl<F> ExportOne for F where F: Fn(EditorEntry, &str) -> Option<String> + 'static {}
