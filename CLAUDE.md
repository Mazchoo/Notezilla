# Notezilla

## MCP Server

The server runs at `http://127.0.0.1:8000`. Use `GET /tools` to list available tools and their schemas without a session handshake.

```bash
curl http://127.0.0.1:8000/tools
```

To call a tool, initialize first (capturing the session ID), then call in one chained command:

```bash
SESSION=$(curl -s -X POST http://127.0.0.1:8000/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -D - \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"curl","version":"1.0"}}}' \
  | grep -i "mcp-session-id" | tr -d '\r' | awk '{print $2}') && \
curl -s -X POST http://127.0.0.1:8000/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "mcp-session-id: $SESSION" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"TOOL_NAME","arguments":{}}}'
```

Required headers on every request: `Content-Type: application/json` and `Accept: application/json, text/event-stream`. The session ID comes from the `mcp-session-id` response header of the initialize call and must be passed on all subsequent requests.
