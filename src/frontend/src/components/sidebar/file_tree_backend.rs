use crate::mcp::tools;
use crate::models::note::{DirectoryContents, NoteFile};

/// Which MCP folder a file tree operates on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileTreeBackend {
    Notes,
    Templates,
}

impl FileTreeBackend {
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
}
