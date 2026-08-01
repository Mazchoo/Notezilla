use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct SearchResult {
    pub document: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SearchResult {
    pub fn path(&self) -> String {
        self.metadata
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown.md")
            .to_string()
    }

    /// Basename of the note path (file name only, no folders).
    pub fn file_name(&self) -> String {
        let path = self.path();
        path.rsplit(['/', '\\'])
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown.md")
            .to_string()
    }

    pub fn title(&self) -> String {
        self.metadata
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.path())
    }

    /// First `n` characters of note body text for list previews.
    pub fn snippet_text(text: &str, n: usize) -> String {
        let mut iter = text.chars();
        let preview: String = iter.by_ref().take(n).collect();
        if iter.next().is_some() {
            format!("{preview}\u{2026}")
        } else {
            preview
        }
    }
}

/// Minimal note payload returned by get_note and search tools.
#[derive(Clone, Debug, Deserialize)]
pub struct NoteFile {
    pub filename: String,
    pub text: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Directory listing returned by get_dir_contents.
#[derive(Clone, Debug, Deserialize)]
pub struct DirectoryContents {
    pub folders: Vec<String>,
    pub files: Vec<String>,
}
