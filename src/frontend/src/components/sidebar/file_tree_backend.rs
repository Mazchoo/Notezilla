//! Selects which MCP folder a file tree reads and writes.

use crate::components::file_io::normalize_note_path;
use crate::mcp::tools;
use crate::models::block::EditorEntry;
use crate::models::note::{DirectoryContents, NoteFile};
use leptos::prelude::GetUntracked;

/// Which MCP folder a file tree operates on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileTreeBackend {
    Notes,
    Templates,
}

impl FileTreeBackend {
    /// Insert `entry` into the open list: notes before templates, templates last.
    pub fn insert_in_editor(self, list: &mut Vec<EditorEntry>, mut entry: EditorEntry) {
        entry.backend = self;
        match self {
            Self::Notes => match list.iter().position(|e| e.backend == Self::Templates) {
                Some(i) => list.insert(i, entry),
                None => list.push(entry),
            },
            Self::Templates => list.push(entry),
        }
    }

    /// Open `entry` in the editor list for this backend.
    /// Notes replace the same path; templates replace any open template and stay last.
    pub fn open_in_editor(self, list: &mut Vec<EditorEntry>, entry: EditorEntry) {
        match self {
            Self::Notes => {
                let path = normalize_note_path(&entry.title.path.get_untracked());
                list.retain(|e| {
                    e.backend != Self::Notes
                        || normalize_note_path(&e.title.path.get_untracked()) != path
                });
            }
            Self::Templates => list.retain(|e| e.backend != Self::Templates),
        }
        self.insert_in_editor(list, entry);
    }

    /// Fetch a directory listing for this tree's MCP folder.
    pub async fn get_dir_contents(
        self,
        session_id: &str,
        path: &str,
    ) -> Result<DirectoryContents, String> {
        match self {
            Self::Notes => tools::get_dir_contents(session_id, path).await,
            Self::Templates => tools::get_template_dir_contents(session_id, path).await,
        }
    }

    /// Fetch a markdown file for this tree's MCP folder.
    pub async fn get_file(self, session_id: &str, path: &str) -> Result<NoteFile, String> {
        match self {
            Self::Notes => tools::get_note(session_id, path).await,
            Self::Templates => tools::get_template(session_id, path).await,
        }
    }

    /// Delete a file in this tree's MCP folder.
    pub async fn delete_file(self, session_id: &str, path: &str) -> Result<(), String> {
        match self {
            Self::Notes => tools::delete_note(session_id, path).await,
            Self::Templates => tools::delete_template(session_id, path).await,
        }
    }

    /// Delete a folder in this tree's MCP folder.
    pub async fn delete_folder(self, session_id: &str, path: &str) -> Result<(), String> {
        match self {
            Self::Notes => tools::delete_folder(session_id, path).await,
            Self::Templates => tools::delete_template_folder(session_id, path).await,
        }
    }

    /// Create a directory in this tree's MCP folder.
    pub async fn new_dir(self, session_id: &str, path: &str) -> Result<(), String> {
        match self {
            Self::Notes => tools::new_dir(session_id, path).await,
            Self::Templates => tools::new_template_dir(session_id, path).await,
        }
    }

    /// Move a file or folder in this tree's MCP folder.
    pub async fn move_item(self, session_id: &str, src: &str, dst: &str) -> Result<(), String> {
        match self {
            Self::Notes => tools::move_dir(session_id, src, dst).await,
            Self::Templates => tools::move_template_dir(session_id, src, dst).await,
        }
    }

    /// Rename a file or folder in this tree's MCP folder.
    pub async fn rename(self, session_id: &str, path: &str, new_name: &str) -> Result<(), String> {
        match self {
            Self::Notes => tools::rename_dir(session_id, path, new_name).await,
            Self::Templates => tools::rename_template_dir(session_id, path, new_name).await,
        }
    }

    /// Return the CSS class for an editor entry frame.
    pub fn entry_class(self) -> &'static str {
        match self {
            Self::Notes => "editor-entry",
            Self::Templates => "editor-entry editor-entry-template",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FileTreeBackend;
    use crate::models::block::EditorEntry;
    use leptos::prelude::{GetUntracked, Owner};

    #[test]
    /// Assert a template replaces any open template and stays after notes.
    fn open_in_editor_keeps_a_single_template_last() {
        let owner = Owner::new();
        owner.with(|| {
            let mut list = vec![EditorEntry::new("./note.md", "note")];
            FileTreeBackend::Templates
                .open_in_editor(&mut list, EditorEntry::new("./t1.md", "one"));
            FileTreeBackend::Templates
                .open_in_editor(&mut list, EditorEntry::new("./t2.md", "two"));
            assert_eq!(list.len(), 2);
            assert_eq!(list[0].backend, FileTreeBackend::Notes);
            assert_eq!(list[0].title.path.get_untracked(), "./note.md");
            assert_eq!(list[1].backend, FileTreeBackend::Templates);
            assert_eq!(list[1].title.path.get_untracked(), "./t2.md");

            FileTreeBackend::Notes.open_in_editor(&mut list, EditorEntry::new("./b.md", "b"));
            assert_eq!(list.len(), 3);
            assert_eq!(list[1].title.path.get_untracked(), "./b.md");
            assert_eq!(list[2].backend, FileTreeBackend::Templates);
        });
    }

    #[test]
    /// Assert template editor frames use a distinct class from notes.
    fn template_editor_classes_differ_from_notes() {
        assert_eq!(FileTreeBackend::Notes.entry_class(), "editor-entry");
        assert_eq!(
            FileTreeBackend::Templates.entry_class(),
            "editor-entry editor-entry-template"
        );
    }
}
