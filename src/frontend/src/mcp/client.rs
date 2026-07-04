use gloo_net::http::Request;
use leptos::task::spawn_local;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};

// Proxied by Trunk to http://127.0.0.1:8000 in development.
const MCP_URL: &str = "/mcp";
static CALL_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize, Debug)]
struct JsonRpcResponse {
    result: Option<Value>,
    error: Option<Value>,
}

/// Extract the first `data: ...` line from an SSE body and parse it as JSON.
/// The FastMCP server returns Content-Type: text/event-stream but delivers
/// the full body at once (not a persistent stream), so we read it as text.
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
        .get("mcp-session-id")
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

/// Call any MCP tool and return its structured payload.
///
/// FastMCP wraps every tool result as:
///   { "result": { "content": [{ "type": "text", "text": "Success" | "Error: ..." }],
///                 "structuredContent": { ... } } }
/// Errors are surfaced from the text message; successful calls return structuredContent.
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

    if let Some(error_message) = message.strip_prefix("Error: ") {
        return Err(error_message.to_string());
    }

    if message != "Success" {
        return Err(format!("Unexpected tool message: {message}"));
    }

    Ok(result
        .get("structuredContent")
        .cloned()
        .unwrap_or_else(|| json!({})))
}
