# cage-bro TypeScript SDK

[![npm](https://img.shields.io/npm/v/@cage-bro/sdk)](https://www.npmjs.com/package/@cage-bro/sdk)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

TypeScript/JavaScript SDK for [cage-bro](https://github.com/aeroxy/cage-bro) — a sandboxed execution environment for AI agents with browser, shell, code execution, file ops, and MCP support.

## Install

```bash
npm install @cage-bro/sdk
```

Requires a running cage-bro server. See the [main project](https://github.com/aeroxy/cage-bro) for installation instructions.

## Quick Start

```typescript
import { CageBro } from "@cage-bro/sdk";

const cage = new CageBro({ baseUrl: "http://localhost:8080" });

// Shell commands
const result = await cage.shellExec("ls -la");
console.log(result.stdout);

// Code execution
const output = await cage.python("print(2 + 2)");
console.log(output.stdout);

// File operations
await cage.fileWrite("hello.txt", "world");
const content = await cage.fileRead("hello.txt");

// Browser automation
await cage.browserLaunch();
await cage.browserNavigate("https://example.com");
const screenshot = await cage.browserScreenshot();
```

## Configuration

```typescript
const cage = new CageBro({
  baseUrl: "http://localhost:8080", // default
  timeout: 30000,                    // request timeout in ms
});
```

## API Reference

### Sandbox

| Method | Description |
|---|---|
| `info()` | Get sandbox info |
| `health()` | Health check |

### Shell

| Method | Description |
|---|---|
| `shellExec(command, timeoutMs?)` | Execute a shell command |
| `shellCreateSession(shell?)` | Create a persistent shell session |

### Files

| Method | Description |
|---|---|
| `fileRead(path)` | Read a file and return its content |
| `fileWrite(path, content)` | Write content to a file |
| `fileEdit(path, oldText, newText)` | Edit a file (find and replace) |
| `fileList(path?)` | List directory contents |
| `fileSearch(query, path?)` | Search files for text |
| `fileDelete(path)` | Delete a file or directory |

### Code Execution

| Method | Description |
|---|---|
| `python(code, timeoutMs?)` | Execute Python code in the sandbox |
| `node(code, timeoutMs?)` | Execute Node.js code in the sandbox |

### Browser

| Method | Description |
|---|---|
| `browserLaunch(port?, stealth?)` | Launch the headless browser |
| `browserNavigate(url)` | Navigate to a URL |
| `browserScreenshot()` | Take a screenshot of the current page |
| `browserClick(selector)` | Click an element by CSS selector |
| `browserType(selector, text)` | Type text into an input element |
| `browserEvaluate(expression)` | Evaluate JavaScript in the browser |
| `browserContent()` | Get the current page HTML content |
| `browserClose()` | Close the browser |

## Links

- [cage-bro main project](https://github.com/aeroxy/cage-bro)
- [Python SDK](https://pypi.org/project/cage-bro/)
- [API documentation](https://github.com/aeroxy/cage-bro#api)

## License

Apache-2.0
