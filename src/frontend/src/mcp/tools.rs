use super::client::call_tool;
use crate::models::note::{DirectoryContents, NoteFile, SearchResult};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct UpsertNoteResult {
    pub new_file_created: bool,
}

fn notes_from_structured(val: &Value) -> Result<Vec<SearchResult>, String> {
    let notes = val
        .get("notes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Missing notes in MCP response".to_string())?;

    notes
        .iter()
        .map(|item| {
            let text = item.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let mut metadata: HashMap<String, Value> = item
                .get("metadata")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            if metadata.is_empty() {
                if let Some(filename) = item.get("filename").and_then(|v| v.as_str()) {
                    metadata.insert("filename".to_string(), json!(filename));
                }
            }
            Ok(SearchResult {
                document: text.to_string(),
                metadata,
            })
        })
        .collect()
}

pub async fn search_by_text(
    session_id: &str,
    text: &str,
    n_results: usize,
) -> Result<Vec<SearchResult>, String> {
    let val = call_tool(
        session_id,
        "search_notes_by_text",
        json!({ "text": text, "n_results": n_results }),
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
