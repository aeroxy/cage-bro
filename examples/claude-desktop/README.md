# Claude Desktop MCP Configuration

Add this to your Claude Desktop config to use cage-bro as an MCP server.

## Config Location

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

## Configuration

```json
{
  "mcpServers": {
    "cage-bro": {
      "command": "cage-bro",
      "args": ["mcp"]
    }
  }
}
```

## Available Tools

Once configured, Claude Desktop will have access to these tools:

- `shell_exec` - Execute shell commands
- `file_read` / `file_write` / `file_edit` / `file_list` / `file_search` - File operations
- `python_exec` / `node_exec` - Code execution
- `browser_navigate` / `browser_screenshot` / `browser_click` / `browser_type` / `browser_evaluate` / `browser_snapshot` - Browser automation
- `sandbox_info` - Get sandbox information
