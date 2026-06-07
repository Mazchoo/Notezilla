use serde_json::json;
use crate::models::note::{McpToolResult, SearchResult};
use super::client::call_tool;

fn zip_results(raw: McpToolResult) -> Vec<SearchResult> {
    raw.documents
        .into_iter()
        .zip(raw.metadatas.into_iter())
        .map(|(document, metadata)| SearchResult { document, metadata })
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

    let raw: McpToolResult = serde_json::from_value(val)
        .map_err(|e| format!("Parse error: {e}"))?;

    if let Some(err) = raw.error {
        return Err(format!("Backend error: {err}"));
    }

    Ok(zip_results(raw))
}

pub async fn list_by_path(
    session_id: &str,
    path_parts: &[&str],
    n_results: usize,
) -> Result<Vec<SearchResult>, String> {
    let val = call_tool(
        session_id,
        "search_notes_by_path",
        json!({ "path_parts": path_parts, "n_results": n_results }),
    )
    .await?;

    let raw: McpToolResult = serde_json::from_value(val)
        .map_err(|e| format!("Parse error: {e}"))?;

    Ok(zip_results(raw))
}

pub async fn upsert_note(
    session_id: &str,
    path: &str,
    contents: &str,
    fields: serde_json::Value,
) -> Result<(), String> {
    call_tool(
        session_id,
        "upsert_note",
        json!({ "path": path, "contents": contents, "fields": fields }),
    )
    .await
    .map(|_| ())
}

pub async fn delete_note(session_id: &str, path: &str) -> Result<(), String> {
    call_tool(session_id, "delete_note", json!({ "path": path }))
        .await
        .map(|_| ())
}
