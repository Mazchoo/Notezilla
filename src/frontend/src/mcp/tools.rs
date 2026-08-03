use super::client::call_tool;
use crate::models::note::{DirectoryContents, NoteFile};
use serde_json::{json, Value};

pub struct UpsertNoteResult {
    pub new_file_created: bool,
}

fn notes_from_structured(val: &Value) -> Result<Vec<NoteFile>, String> {
    let notes = val
        .get("notes")
        .cloned()
        .ok_or_else(|| "Missing notes in MCP response".to_string())?;

    serde_json::from_value(notes).map_err(|e| format!("Parse error: {e}"))
}

pub async fn search_by_text(
    session_id: &str,
    text: &str,
    frontmatter: &str,
    n_results: usize,
    offset: usize,
) -> Result<Vec<NoteFile>, String> {
    let val = call_tool(
        session_id,
        "search_notes_by_text",
        json!({
            "text": text,
            "frontmatter": frontmatter,
            "n_results": n_results,
            "offset": offset
        }),
    )
    .await?;

    notes_from_structured(&val)
}

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

pub async fn delete_note(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "delete_note", json!({ "path": path }))
        .await
        .map(|_| ())
}

pub async fn delete_folder(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "delete_folder", json!({ "path": path }))
        .await
        .map(|_| ())
}

pub async fn new_dir(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "new_dir", json!({ "path": path }))
        .await
        .map(|_| ())
}

pub async fn move_dir(session_id: &str, src: &str, dst: &str) -> Result<(), String> {
    call_tool(session_id, "move_dir", json!({ "src": src, "dst": dst }))
        .await
        .map(|_| ())
}

pub async fn rename_dir(session_id: &str, path: &str, new_name: &str) -> Result<(), String> {
    call_tool(
        session_id,
        "rename_dir",
        json!({ "path": path, "new_name": new_name }),
    )
    .await
    .map(|_| ())
}

pub async fn get_dir_contents(session_id: &str, path: &str) -> Result<DirectoryContents, String> {
    let val = call_tool(session_id, "get_dir_contents", json!({ "path": path })).await?;

    serde_json::from_value(val).map_err(|e| format!("Parse error: {e}"))
}

pub async fn get_note(session_id: &str, path: &str) -> Result<NoteFile, String> {
    let val = call_tool(session_id, "get_note", json!({ "path": path })).await?;

    let notes = val
        .get("notes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing notes in MCP response".to_string())?;

    let item = notes
        .first()
        .ok_or_else(|| format!("Note not found at '{path}'"))?;

    serde_json::from_value(item.clone()).map_err(|e| format!("Parse error: {e}"))
}
