# Related

What cage-bro works with, what it builds on, and what you might use alongside it.

---

## Direct Integrations

### Claude Desktop

cage-bro ships as an MCP server that plugs directly into Claude Desktop. Claude gets access to all 15 tools (shell, files, code, browser) without any wrapper code.

**Config**: `~/Library/Application Support/Claude/claude_desktop_config.json`
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

See [examples/claude-desktop/](../examples/claude-desktop/) for a ready-to-use config.

### Cursor

Same MCP integration as Claude Desktop. Add the cage-bro MCP server to your Cursor settings.

### OpenAI Function Calling

Example at [examples/openai-functions/cage_bro_example.py](../examples/openai-functions/cage_bro_example.py). Wraps cage-bro REST API calls as OpenAI function definitions for GPT-4 function calling.

### LangChain

Example at [examples/langchain/cage_bro_tools.py](../examples/langchain/cage_bro_tools.py). Wraps cage-bro endpoints as LangChain `Tool` objects for use with agents.

---

## Runtime Dependencies

### Obscura Browser

A stealth headless browser with CDP (Chrome DevTools Protocol) support. cage-bro launches it as a sidecar process and communicates over WebSocket CDP.

- Downloaded via `cage-bro setup`
- GitHub: [h4ckf0r0day/obscura](https://github.com/h4ckf0r0day/obscura/)
- Default CDP port: 9333
- Stealth mode: avoids common bot detection

### ipykernel

Python Jupyter kernel for stateful code execution. Installed as a Python dependency. cage-bro manages kernel lifecycles, generates connection files, and communicates via `jupyter_client`.

### Python 3 / Node.js

Required for code execution endpoints. cage-bro shells out to `python3` and `node` on PATH.

---

## Rust Crate Dependencies

| Crate | Used for |
|---|---|
| `axum` | HTTP server and routing |
| `tokio` | Async runtime |
| `tower-http` | CORS middleware |
| `portable-pty` | PTY-based process execution |
| `tokio-tungstenite` | WebSocket client (CDP) |
| `rust-embed` | Embed dashboard static files in binary |
| `serde` / `serde_json` | JSON serialization |
| `uuid` | Session and sandbox IDs |
| `tracing` / `tracing-subscriber` | Structured logging |
| `glob` | File pattern matching for search |
| `reqwest` | HTTP client (browser manager) |

---

## Frontend Stack (Dashboard)

| Library | Version | Purpose |
|---|---|---|
| React | 19 | UI framework |
| Vite | -- | Build tool |
| xterm.js | -- | Terminal emulator in browser |
| radix-ui | -- | Accessible component primitives |
| tailwindcss | -- | Utility-first CSS |
| shadcn/ui | -- | Component library |

---

## Installation Channels

| Channel | Command |
|---|---|
| Cargo | `cargo install cage-bro` |
| PyPI (CLI wrapper) | `pip install cage-bro-cli` |
| npm (CLI wrapper) | `npm install @cage-bro/cli` |
| Homebrew | `brew install cage-bro` |
| From source | `cargo build --release` |

---

## SDKs

| Language | Package | Install |
|---|---|---|
| Python | `cage-bro` | `pip install cage-bro` |
| TypeScript | `@cage-bro/sdk` | `npm install @cage-bro/sdk` |

---

## Examples

| Example | Location | Description |
|---|---|---|
| Claude Desktop config | [examples/claude-desktop/](../examples/claude-desktop/) | Ready-to-use MCP config |
| OpenAI function calling | [examples/openai-functions/](../examples/openai-functions/) | GPT-4 integration |
| LangChain tools | [examples/langchain/](../examples/langchain/) | LangChain agent tools |
