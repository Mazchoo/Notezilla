# Notezilla

## MCP Server

The server runs at `http://127.0.0.1:8020` and is registered in `.mcp.json` as `notezilla`. Call its tools through the MCP integration (`mcp__notezilla__<tool>`), not raw curl — direct requests to `/mcp` are denied in `.claude/settings.json` so the per-tool permissions can't be bypassed.

Use `GET /tools` to list available tools and their schemas:

```bash
curl http://127.0.0.1:8020/tools
```

Tool permissions (see `.claude/settings.json`):
- **Allowed:** read-only tools (`search_notes`, `get_note`, `get_dir_contents`, `get_template`, `get_template_dir_contents`, `get_database_sync_difference`)
- **Ask first:** tools that create, move, rename, upsert or restore
- **Denied:** all `delete_*` tools
