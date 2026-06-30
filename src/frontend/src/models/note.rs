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

    pub fn title(&self) -> String {
        self.metadata
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.path())
    }
}

/// Minimal note payload returned by get_note and search tools.
#[derive(Clone, Debug, Deserialize)]
pub struct NoteFile {
    pub filename: String,
    pub text: String,
}

/// Directory listing returned by get_dir_contents.
#[derive(Clone, Debug, Deserialize)]
pub struct DirectoryContents {
    pub folders: Vec<String>,
    pub files: Vec<String>,
}
