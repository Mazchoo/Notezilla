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
                "protocolVersion": "2025-06-18",
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
    parsed_response = _parse_sse(resp.text)
    assert parsed_response["result"]["isError"] is False, parsed_response

    return parsed_response["result"]


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
    print(call_tool(session, "get_note", {"path": "hello.md"}))

    print("\n=== get_dir_contents ===")
    print(call_tool(session, "get_dir_contents", {"path": "."}))

    print("\n=== search_notes ===")
    print(
        call_tool(
            session,
            "search_notes",
            {
                "text": "python async",
                "n_results": 5,
                "frontmatter": "tags: [paragraph]",
                "path_filter": "2025, 2024"
            },
        )
    )

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

    print("\n=== new_dir ===")
    print(call_tool(session, "new_dir", {"path": "some-random-folder"}))

    print("\n=== move_dir ===")
    print(
        call_tool(
            session, "move_dir", {"src": "my-note.md", "dst": "some-random-folder"}
        )
    )

    print("\n=== rename_dir ===")
    print(
        call_tool(
            session,
            "rename_dir",
            {"path": "./some-random-folder/my-note.md", "new_name": "some-note"},
        )
    )

    print("\n=== delete_note ===")
    print(
        call_tool(session, "delete_note", {"path": "./some-random-folder/some-note.md"})
    )

    print("\n=== delete_folder ===")
    print(call_tool(session, "delete_folder", {"path": "some-random-folder"}))

    print("\n=== get_template ===")
    print(call_tool(session, "get_template", {"path": "animal_facts.md"}))

    print("\n=== get_template_dir_contents ===")
    print(call_tool(session, "get_template_dir_contents", {"path": ""}))

    print("\n=== upsert_template ===")
    print(
        call_tool(
            session,
            "upsert_template",
            {
                "path": "my-template.md",
                "contents": "Make a report",
                "fields": {"tags": ["graph"]},
            },
        )
    )

    print("\n=== new_template_dir ===")
    print(call_tool(session, "new_template_dir", {"path": "some-template-folder"}))

    print("\n=== move_template_dir ===")
    print(
        call_tool(
            session,
            "move_template_dir",
            {"src": "my-template.md", "dst": "some-template-folder"},
        )
    )

    print("\n=== rename_template_dir ===")
    print(
        call_tool(
            session,
            "rename_template_dir",
            {
                "path": "./some-template-folder/my-template.md",
                "new_name": "some-template",
            },
        )
    )

    print("\n=== delete_template ===")
    print(
        call_tool(
            session,
            "delete_template",
            {"path": "./some-template-folder/some-template.md"},
        )
    )

    print("\n=== delete_template_folder ===")
    print(call_tool(session, "delete_template_folder", {"path": "some-template-folder"}))
