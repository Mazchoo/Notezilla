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

/// Raw shape returned by get_dir_contents.
#[derive(Clone, Debug, Deserialize)]
pub struct DirectoryContents {
    pub folders: Vec<String>,
    pub files: Vec<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Raw shape returned by every MCP search tool.
#[derive(Debug, Deserialize)]
pub struct McpToolResult {
    pub documents: Vec<String>,
    pub metadatas: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub error: Option<String>,
}
