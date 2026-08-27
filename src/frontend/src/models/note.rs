use serde::Deserialize;
use std::collections::HashMap;

/// Minimal note payload returned by get_note and search tools.
#[derive(Clone, Debug, Deserialize)]
pub struct NoteFile {
    pub filename: String,
    pub text: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl NoteFile {
    /// Return the basename of the note path.
    pub fn file_name(&self) -> String {
        self.filename
            .rsplit(['/', '\\'])
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown.md")
            .to_string()
    }

    /// Return the first `n` characters of note body text for list previews.
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

/// Directory listing returned by get_dir_contents.
#[derive(Clone, Debug, Deserialize)]
pub struct DirectoryContents {
    pub folders: Vec<String>,
    pub files: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(filename: &str) -> NoteFile {
        NoteFile {
            filename: filename.into(),
            text: String::new(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    /// Assert file_name returns the last path segment, or `unknown.md` when empty.
    fn file_name_returns_basename_or_unknown() {
        assert_eq!(note("folder/hello.md").file_name(), "hello.md");
        assert_eq!(note("folder\\hello.md").file_name(), "hello.md");
        assert_eq!(note("hello.md").file_name(), "hello.md");
        assert_eq!(note("folder/").file_name(), "unknown.md");
        assert_eq!(note("").file_name(), "unknown.md");
    }

    #[test]
    /// Assert snippet_text truncates by Unicode scalar and appends an ellipsis.
    fn snippet_text_truncates_with_ellipsis() {
        assert_eq!(NoteFile::snippet_text("hello", 10), "hello");
        assert_eq!(NoteFile::snippet_text("hello", 3), "hel\u{2026}");
        assert_eq!(NoteFile::snippet_text("café", 3), "caf\u{2026}");
        assert_eq!(NoteFile::snippet_text("", 5), "");
    }
}
