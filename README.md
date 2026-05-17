# cage-bro

A sandboxed execution environment for AI agents. Single Rust binary with browser, shell, code execution, file ops, and MCP server.

## Why cage-bro

| | Claude Desktop sandbox | cage-bro |
|---|---|---|
| **Works with** | Claude only | Any agent (LangChain, CrewAI, OpenAI, custom) |
| **Browser** | None | Obscura (stealth, CDP) |
| **Code exec** | None | Python, Node, Jupyter |
| **Terminal** | None | Full PTY |
| **Self-hosted** | No (Anthropic controls it) | Yes, your infra |
| **API** | Claude Desktop only | REST + MCP |
| **Isolation** | macOS sandbox | Process/microVM |

Claude's sandbox is a security layer. cage-bro is an execution environment.

**Why use cage-bro:**
1. **Framework-agnostic** — one sandbox for all your agents, not per-vendor
2. **Browser + code + shell + files** — full agent runtime, not just tool isolation
3. **Self-hosted** — your data, your infra, no vendor lock-in
4. **MCP + REST** — works with Claude Desktop AND Cursor AND custom agents
5. **Obscura stealth** — anti-detection browser for scraping at scale

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/cage-bro serve --port 8080

# Dashboard
open http://localhost:8080
```

## Features

| Feature | Description |
|---|---|
| **Shell** | PTY-based terminal sessions via WebSocket |
| **Browser** | Obscura headless browser (stealth mode, CDP) |
| **Code** | Python, Node.js (stateless) + Jupyter (stateful) |
| **Files** | Read, write, edit, list, search with sandbox scope |
| **MCP** | Built-in MCP server for Claude Desktop, Cursor, etc. |
| **Dashboard** | Web UI with terminal, code editor, file browser |

## Installation

```bash
# Install obscura browser
cage-bro setup

# Start server
cage-bro serve --port 8080
```

## API

### Shell
```bash
curl -X POST http://localhost:8080/v1/shell/exec -d '{"command": "ls -la"}'
```

### Code
```bash
curl -X POST http://localhost:8080/v1/code/python -d '{"code": "print(2+2)"}'
```

### Browser
```bash
curl -X POST http://localhost:8080/v1/browser/launch -d '{"stealth": true}'
curl -X POST http://localhost:8080/v1/browser/navigate -d '{"url": "https://example.com"}'
```

### Files
```bash
curl -X POST http://localhost:8080/v1/file/read -d '{"path": "test.txt"}'
curl -X POST http://localhost:8080/v1/file/write -d '{"path": "test.txt", "content": "hello"}'
```

## MCP Server

### Claude Desktop
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

### HTTP mode
```bash
cage-bro mcp --http --port 8081
```

## SDKs

### Python
```bash
pip install cage-bro
```

```python
from cage_bro import CageBro

cage = CageBro("http://localhost:8080")
result = cage.shell_exec("ls -la")
print(result["stdout"])
```

### TypeScript
```bash
npm install @cage-bro/sdk
```

```typescript
import { CageBro } from "@cage-bro/sdk";

const cage = new CageBro({ baseUrl: "http://localhost:8080" });
const result = await cage.shellExec("ls -la");
console.log(result.stdout);
```

## Dashboard

The web dashboard is embedded in the binary and available at `http://localhost:8080`.

| Route | Page |
|---|---|
| `/#/` | Dashboard home |
| `/#/terminal` | xterm.js terminal |
| `/#/code` | Code execution |
| `/#/files` | File browser/editor |
| `/#/browser` | Browser view |

## Architecture

```
cage-bro (single Rust binary)
├── Axum HTTP server (REST API + dashboard)
├── ProcessRuntime (PTY shell, landlock, seccomp)
├── BrowserManager (Obscura sidecar via CDP)
├── JupyterKernelManager (ipykernel via jupyter_client)
├── MCP Server (stdio + HTTP/SSE)
└── Dashboard (React + shadcn/ui, embedded via rust-embed)
```

## License

Apache-2.0
