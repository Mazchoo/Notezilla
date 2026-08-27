use crate::components::toast::show_mcp_warnings;
use crate::constants::MCP_URL;
use gloo_net::http::Request;
use leptos::task::spawn_local;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static CALL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize, Debug)]
struct JsonRpcResponse {
    result: Option<Value>,
    error: Option<Value>,
}

/// Parse the first `data:` JSON-RPC payload from an SSE body.
fn parse_sse(body: &str) -> Option<JsonRpcResponse> {
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            let trimmed = rest.trim();
            if trimmed.is_empty() || trimmed == "[DONE]" {
                continue;
            }
            return serde_json::from_str(trimmed).ok();
        }
    }
    None
}

/// Perform the MCP initialize handshake and return the session ID.
pub async fn initialize_session() -> Result<String, String> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "notezilla-leptos", "version": "0.1.0" }
        }
    });

    let response = Request::post(MCP_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .map_err(|e| format!("Serialize error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let session_id = response
        .headers()
        .entries()
        .find(|(name, _)| name.eq_ignore_ascii_case("mcp-session-id"))
        .map(|(_, value)| value)
        .ok_or_else(|| "No mcp-session-id header in initialize response".to_string())?;

    // Fire-and-forget notifications/initialized
    let notify = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    if let Ok(req) = Request::post(MCP_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("mcp-session-id", &session_id)
        .json(&notify)
    {
        spawn_local(async move {
            let _ = req.send().await;
        });
    }

    Ok(session_id)
}

/// Call an MCP tool and return its structured payload.
pub async fn call_tool(
    session_id: &str,
    tool_name: &str,
    arguments: Value,
) -> Result<Value, String> {
    let id = CALL_ID.fetch_add(1, Ordering::Relaxed);

    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": { "name": tool_name, "arguments": arguments }
    });

    let response = Request::post(MCP_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("mcp-session-id", session_id)
        .json(&body)
        .map_err(|e| format!("Serialize error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("Body read error: {e}"))?;

    let rpc = parse_sse(&text).ok_or_else(|| format!("Could not parse SSE body: {text:?}"))?;

    if let Some(err) = rpc.error {
        return Err(format!("RPC error: {err}"));
    }

    let result = rpc.result.ok_or("Empty RPC result")?;

    let message = result
        .pointer("/content/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Unexpected result shape: {result}"))?;

    let structured = result
        .get("structuredContent")
        .cloned()
        .unwrap_or_else(|| json!({}));
    show_mcp_warnings(&warnings_from_structured(&structured));

    if let Some(error_message) = message.strip_prefix("Error: ") {
        return Err(error_message.to_string());
    }

    if message != "Success" {
        return Err(format!("Unexpected tool message: {message}"));
    }

    Ok(structured)
}

/// Collect non-empty `warnings` strings from a tool payload.
fn warnings_from_structured(structured: &Value) -> Vec<String> {
    let Some(arr) = structured.get("warnings").and_then(Value::as_array) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    /// Assert the first non-empty `data:` JSON-RPC payload is parsed.
    fn parse_sse_reads_the_first_data_payload() {
        let body = "event: message\ndata: {\"result\":{\"ok\":true}}\n\n";
        let rpc = parse_sse(body).expect("payload");
        assert_eq!(rpc.result, Some(json!({"ok": true})));
        assert_eq!(rpc.error, None);
    }

    #[test]
    /// Assert empty and `[DONE]` data lines are skipped.
    fn parse_sse_skips_empty_and_done_lines() {
        let body = "data:\ndata: [DONE]\ndata: {\"error\":{\"code\":1}}\n";
        let rpc = parse_sse(body).expect("payload");
        assert_eq!(rpc.error, Some(json!({"code": 1})));
    }

    #[test]
    /// Assert missing or invalid payloads yield None.
    fn parse_sse_returns_none_without_valid_json() {
        assert!(parse_sse("no data here").is_none());
        assert!(parse_sse("data: not-json\n").is_none());
    }

    #[test]
    /// Assert non-empty warning strings are collected and empties dropped.
    fn warnings_from_structured_collects_non_empty_strings() {
        assert_eq!(
            warnings_from_structured(&json!({"warnings": ["a", "", "b", 1]})),
            vec!["a".to_string(), "b".to_string()]
        );
        assert!(warnings_from_structured(&json!({})).is_empty());
        assert!(warnings_from_structured(&json!({"warnings": "x"})).is_empty());
    }
}
