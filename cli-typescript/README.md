# cage-bro CLI (TypeScript)

[![npm](https://img.shields.io/npm/v/@cage-bro/cli)](https://www.npmjs.com/package/@cage-bro/cli)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

CLI installer for [cage-bro](https://github.com/aeroxy/cage-bro) — downloads the pre-built Rust binary on install.

## Install

```bash
npm install -g @cage-bro/cli
```

## Usage

```bash
# Start the sandbox server
cage-bro serve --port 8080

# Run MCP server (for Claude Desktop, Cursor, etc.)
cage-bro mcp

# Install dependencies (Obscura browser)
cage-bro setup
```

On first run, the CLI downloads the pre-built binary for your platform from [GitHub releases](https://github.com/aeroxy/cage-bro/releases) and caches it in `~/.cache/cage-bro/`.

## Supported Platforms

| Platform | Status |
|---|---|
| macOS ARM64 | Pre-built binary available |
| Other platforms | Build from source (see below) |

For unsupported platforms, build from source:

```bash
cargo install --git https://github.com/aeroxy/cage-bro
```

## What is cage-bro?

cage-bro is a sandboxed execution environment for AI agents. Single Rust binary with browser, shell, code execution, file ops, and MCP server. Works with any agent framework — LangChain, CrewAI, OpenAI, or custom.

## Links

- [cage-bro main project](https://github.com/aeroxy/cage-bro)
- [TypeScript SDK](https://www.npmjs.com/package/@cage-bro/sdk) (programmatic API)
- [Python CLI](https://pypi.org/project/cage-bro-cli/)

## License

Apache-2.0
