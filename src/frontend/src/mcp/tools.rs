use super::client::call_tool;
use crate::models::note::{DirectoryContents, NoteFile};
use serde_json::{json, Value};

pub struct UpsertNoteResult {
    pub new_file_created: bool,
}

/// Parse `notes` from an MCP structured payload.
fn notes_from_structured(val: &Value) -> Result<Vec<NoteFile>, String> {
    let notes = val
        .get("notes")
        .cloned()
        .ok_or_else(|| "Missing notes in MCP response".to_string())?;

    serde_json::from_value(notes).map_err(|e| format!("Parse error: {e}"))
}

/// Search notes via the MCP `search_notes` tool.
pub async fn search_by_text(
    session_id: &str,
    text: &str,
    frontmatter: &str,
    path_filter: &str,
    n_results: usize,
    offset: usize,
) -> Result<Vec<NoteFile>, String> {
    let val = call_tool(
        session_id,
        "search_notes",
        json!({
            "text": text,
            "frontmatter": frontmatter,
            "path_filter": path_filter,
            "n_results": n_results,
            "offset": offset
        }),
    )
    .await?;

    notes_from_structured(&val)
}

/// Create or update a note via the MCP `upsert_note` tool.
pub async fn upsert_note(
    session_id: &str,
    path: &str,
    contents: &str,
    fields: serde_json::Value,
) -> Result<UpsertNoteResult, String> {
    let val = call_tool(
        session_id,
        "upsert_note",
        json!({ "path": path, "contents": contents, "fields": fields }),
    )
    .await?;

    let new_file_created = val
        .get("newFileCreated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(UpsertNoteResult { new_file_created })
}

/// Delete a note via the MCP `delete_note` tool.
pub async fn delete_note(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "delete_note", json!({ "path": path }))
        .await
        .map(|_| ())
}

/// Delete a folder via the MCP `delete_folder` tool.
pub async fn delete_folder(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "delete_folder", json!({ "path": path }))
        .await
        .map(|_| ())
}

/// Create a directory via the MCP `new_dir` tool.
pub async fn new_dir(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "new_dir", json!({ "path": path }))
        .await
        .map(|_| ())
}

/// Move a file or folder via the MCP `move_dir` tool.
pub async fn move_dir(session_id: &str, src: &str, dst: &str) -> Result<(), String> {
    call_tool(session_id, "move_dir", json!({ "src": src, "dst": dst }))
        .await
        .map(|_| ())
}

/// Rename a file or folder via the MCP `rename_dir` tool.
pub async fn rename_dir(session_id: &str, path: &str, new_name: &str) -> Result<(), String> {
    call_tool(
        session_id,
        "rename_dir",
        json!({ "path": path, "new_name": new_name }),
    )
    .await
    .map(|_| ())
}

/// Fetch a directory listing via the MCP `get_dir_contents` tool.
pub async fn get_dir_contents(session_id: &str, path: &str) -> Result<DirectoryContents, String> {
    let val = call_tool(session_id, "get_dir_contents", json!({ "path": path })).await?;

    serde_json::from_value(val).map_err(|e| format!("Parse error: {e}"))
}

/// Fetch a note via the MCP `get_note` tool.
pub async fn get_note(session_id: &str, path: &str) -> Result<NoteFile, String> {
    get_markdown_file(session_id, "get_note", path).await
}

/// Fetch a template directory listing via the MCP backend.
pub async fn get_template_dir_contents(
    session_id: &str,
    path: &str,
) -> Result<DirectoryContents, String> {
    let val = call_tool(
        session_id,
        "get_template_dir_contents",
        json!({ "path": path }),
    )
    .await?;

    serde_json::from_value(val).map_err(|e| format!("Parse error: {e}"))
}

/// Fetch a template file via the MCP `get_template` tool.
pub async fn get_template(session_id: &str, path: &str) -> Result<NoteFile, String> {
    get_markdown_file(session_id, "get_template", path).await
}

/// Delete a template file via the MCP `delete_template` tool.
pub async fn delete_template(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "delete_template", json!({ "path": path }))
        .await
        .map(|_| ())
}

/// Create a template directory via the MCP `new_template_dir` tool.
pub async fn new_template_dir(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "new_template_dir", json!({ "path": path }))
        .await
        .map(|_| ())
}

/// Delete a template folder via the MCP `delete_template_folder` tool.
pub async fn delete_template_folder(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(
        session_id,
        "delete_template_folder",
        json!({ "path": path }),
    )
    .await
    .map(|_| ())
}

/// Move a template file or folder via the MCP `move_template_dir` tool.
pub async fn move_template_dir(session_id: &str, src: &str, dst: &str) -> Result<(), String> {
    call_tool(
        session_id,
        "move_template_dir",
        json!({ "src": src, "dst": dst }),
    )
    .await
    .map(|_| ())
}

/// Rename a template file or folder via the MCP `rename_template_dir` tool.
pub async fn rename_template_dir(
    session_id: &str,
    path: &str,
    new_name: &str,
) -> Result<(), String> {
    call_tool(
        session_id,
        "rename_template_dir",
        json!({ "path": path, "new_name": new_name }),
    )
    .await
    .map(|_| ())
}

/// Fetch a markdown file from an MCP get tool and parse the first note.
async fn get_markdown_file(session_id: &str, tool: &str, path: &str) -> Result<NoteFile, String> {
    let val = call_tool(session_id, tool, json!({ "path": path })).await?;

    let notes = val
        .get("notes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing notes in MCP response".to_string())?;

    let item = notes
        .first()
        .ok_or_else(|| format!("File not found at '{path}'"))?;

    serde_json::from_value(item.clone()).map_err(|e| format!("Parse error: {e}"))
}
