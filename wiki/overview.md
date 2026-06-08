# cage-bro Wiki

A sandboxed execution environment for AI agents. Single Rust binary with browser, shell, code execution, file ops, and MCP server.

## What is cage-bro?

cage-bro gives AI agents a sandboxed playground: a PTY shell, headless browser, code runtimes (Python, Node.js, Jupyter), and a scoped filesystem -- all behind a single REST API and MCP server. It's designed to be embedded into agent workflows (Claude Desktop, Cursor, LangChain, OpenAI function calling) or used standalone via the web dashboard.

## Architecture

```
cage-bro (single Rust binary)
├── Axum HTTP server (REST API + E2B lifecycle + embedded dashboard)
├── ProcessRuntime (exec isolation: Landlock [Linux] + rlimits, snapshots)
├── BrowserManager (Obscura headless browser via CDP)
├── JupyterKernelManager (ipykernel via jupyter_client)
├── MCP Server (stdio + HTTP/SSE transport)
└── Dashboard (React + shadcn/ui, embedded via rust-embed)
```

## Crate Structure

| Crate | Purpose |
|---|---|
| `cage-bro` | Main binary: HTTP server, API routes, MCP server, CLI, browser manager, dashboard |
| `cage-bro-runtime` | Core runtime: process execution, PTY sessions, filesystem operations |
| `cage-bro-code` | Code execution: stateless (Python/Node) and stateful (Jupyter kernels) |

## Quick Start

```bash
# Build
cargo build --release

# Setup browser
./target/release/cage-bro setup

# Start server
./target/release/cage-bro serve --port 8080

# Open dashboard
open http://localhost:8080
```

## SDKs

- **Python**: `pip install cage-bro`
- **TypeScript**: `npm install @cage-bro/sdk`

## Wiki Contents

- [API Reference](apis.md) -- Every endpoint, MCP tool, and SDK method
- [Common Patterns](common-patterns.md) -- The 3 most common use cases
- [Gotchas](gotchas.md) -- Edge cases, limitations, and things that will bite you
- [Related](related.md) -- Integrations and ecosystem
- [Branding & Assets](branding.md) -- Logo, palette, and where the brand assets live
