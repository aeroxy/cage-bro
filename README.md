# cage-bro

[![crates.io](https://img.shields.io/crates/v/cage-bro)](https://crates.io/crates/cage-bro)
[![npm CLI](https://img.shields.io/npm/v/@cage-bro/cli)](https://www.npmjs.com/package/@cage-bro/cli)
[![PyPI CLI](https://img.shields.io/pypi/v/cage-bro-cli)](https://pypi.org/project/cage-bro-cli/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

A sandboxed execution environment for AI agents. Single Rust binary with browser, shell, code execution, file ops, and MCP server.

## Why cage-bro

| | Claude Desktop sandbox | Docker sandbox | cage-bro |
|---|---|---|---|
| **Works with** | Claude only | Any | Any agent (LangChain, CrewAI, OpenAI, custom) |
| **Browser** | None | Manual setup | Obscura (stealth, CDP) |
| **Code exec** | None | Any (manual) | Python, Node, Jupyter + any language via shell |
| **Terminal** | None | Full | Full PTY |
| **Self-hosted** | No (Anthropic controls it) | Yes | Yes, your infra |
| **API** | Claude Desktop only | None (manual) | REST + MCP |
| **Memory** | N/A | ~2GB per container (Chromium) | ~100MB per sandbox |
| **Init time** | N/A | Seconds | ~1ms |
| **Density** | N/A | 1 container per VM | 20+ sandboxes per 1c1g VM |

Claude's sandbox is a security layer. Docker is a general-purpose container. cage-bro is an agent execution environment.

**Why use cage-bro:**
1. **Framework-agnostic** — one sandbox for all your agents, not per-vendor
2. **Browser + code + shell + files** — full agent runtime, not just tool isolation
3. **Self-hosted** — your data, your infra, no vendor lock-in
4. **MCP + REST** — works with Claude Desktop AND Cursor AND custom agents
5. **Obscura stealth** — anti-detection browser for scraping at scale

## Why not Docker?

Docker sandboxes work, but they're heavy. A Chromium-based browser container eats ~2GB and takes seconds to start. If you're running 20 agents on a small VM, you need 40GB of RAM just for the sandboxes.

cage-bro is a single Rust binary. ~100MB memory, ~1ms init. A 1c1g VM can host 20+ concurrent sandboxes. For individuals, that means running a full agent stack on a $5/month VPS. For companies, that means millions in infrastructure savings at scale.

The best pattern: put cage-bro inside a Docker container. You get isolation at the container level and density at the sandbox level.

## Does it only support Python, Node, and Jupyter?

Python, Node.js, and Jupyter are the bundled runtimes — they start instantly with no setup. But `shell_exec` can run anything: Go, Rust, Java, Ruby, Bash scripts, compiled binaries. If it runs in a shell, it runs in cage-bro.

## Built for financial services

cage-bro was designed with regulated industries in mind. If you're building AI agent systems in finance, you need:

- **Self-hosted, on-prem execution** — data can't leave your network. No cloud dependencies, no third-party telemetry.
- **Sandboxed code execution** — agents running untrusted code (backtesting, data analysis, report generation) need isolation. Each sandbox is its own process with landlock/seccomp boundaries.
- **Auditability** — every action goes through the REST API. Structured inputs, structured outputs. Easy to log, easy to replay, easy to audit.
- **Human-in-the-loop integration** — pair with an approval layer before sensitive operations. The agent proposes, a human approves, cage-bro executes.
- **MCP + structured tooling** — agents interact via typed tool calls, not raw shell access. Policy agents, compliance agents, and trading agents all get the same clean interface.
- **Density for multi-agent architectures** — agentic trading platforms, IPO audit pipelines, KYC/AML workflows all need many isolated sandboxes running in parallel. One small VM can host dozens.

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

### Cargo
```bash
cargo install cage-bro
```

### Homebrew
```bash
brew install aeroxy/tap/cage-bro
```

### pip
```bash
pip install cage-bro-cli
```

### npm
```bash
npm install -g @cage-bro/cli
```

### Setup
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

### Python SDK
[![PyPI](https://img.shields.io/pypi/v/cage-bro)](https://pypi.org/project/cage-bro/)

```bash
pip install cage-bro
```

```python
from cage_bro import CageBro

cage = CageBro("http://localhost:8080")
result = cage.shell_exec("ls -la")
print(result["stdout"])
```

### TypeScript SDK
[![npm](https://img.shields.io/npm/v/@cage-bro/sdk)](https://www.npmjs.com/package/@cage-bro/sdk)

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

**Demo (mock mode):** https://aeroxy.github.io/cage-bro/

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
