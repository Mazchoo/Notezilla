"""Example HTTP requests to the running FastMCP server.

MCP streamable-http protocol over plain HTTP:
  1. POST /mcp  initialize              → receive mcp-session-id header
  2. POST /mcp  notifications/initialized  (with session id header)
  3. POST /mcp  tools/call              (with session id header)

Responses arrive as Server-Sent Events (SSE):
  data: {"jsonrpc":"2.0","id":1,"result":{...}}\n\n
"""

import json

import httpx

from src.config import MCP_PORT

MCP_URL = f"http://localhost:{MCP_PORT}/mcp"
HEADERS = {
    "Content-Type": "application/json",
    "Accept": "application/json, text/event-stream",
}


def _parse_sse(text: str) -> dict:
    """Extract the JSON payload from an SSE response body.

    SSE lines look like:
        event: message
        data: {"jsonrpc": "2.0", ...}
    """
    for line in text.splitlines():
        if line.startswith("data:"):
            return json.loads(line[len("data:") :].strip())
    return {}


def _post(session_id: str | None, body: dict) -> httpx.Response:
    headers = {**HEADERS}
    if session_id:
        headers["mcp-session-id"] = session_id
    response = httpx.post(MCP_URL, json=body, headers=headers)
    response.raise_for_status()
    return response


def open_session() -> str:
    """Send initialize + initialized handshake, return the session id."""
    # Step 1: initialize — session id is returned in the response header
    resp = _post(
        None,
        {
            "jsonrpc": "2.0",
            "id": 0,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "make_request", "version": "1.0"},
            },
        },
    )
    session_id = resp.headers.get("mcp-session-id", "")

    # Step 2: notify server that client is ready
    _post(
        session_id,
        {
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
        },
    )
    return session_id


def call_tool(session_id: str, name: str, arguments: dict, req_id: int = 1) -> dict:
    """Call an MCP tool and return the parsed JSON response."""
    resp = _post(
        session_id,
        {
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments},
        },
    )
    return _parse_sse(resp.text)


def list_tools(session_id: str) -> dict:
    """List all available MCP tools."""
    resp = _post(
        session_id,
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {},
        },
    )
    return _parse_sse(resp.text)


if __name__ == "__main__":
    session = open_session()
    print(f"Session ID: {session}")

    print("\n=== tools/list ===")
    print(list_tools(session))

    print("\n=== get_note ===")
    print(call_tool(session, "get_note", {"path": "new_file.md"}))

    print("\n=== get_dir_contents ===")
    print(call_tool(session, "get_dir_contents", {"path": "."}))

    print("\n=== search_notes_by_text ===")
    print(
        call_tool(
            session, "search_notes_by_text", {"text": "python async", "n_results": 5}
        )
    )

    print("\n=== search_notes_by_field ===")
    print(
        call_tool(
            session, "search_notes_by_field", {"field": "date", "value": "2020-02-09"}
        )
    )

    print("\n=== search_notes_by_tag ===")
    print(
        call_tool(session, "search_notes_by_tag", {"field": "tags", "value": "journal"})
    )

    print("\n=== search_notes_by_path ===")
    print(call_tool(session, "search_notes_by_path", {"path_parts": ["2024", "01"]}))

    print("\n=== upsert_note ===")
    print(
        call_tool(
            session,
            "upsert_note",
            {
                "path": "my-note.md",
                "contents": "Hello world",
                "fields": {"title": "My Note"},
            },
        )
    )

    print("\n=== delete_note ===")
    print(call_tool(session, "delete_note", {"path": "my-note.md"}))
