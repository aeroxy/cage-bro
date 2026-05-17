# API Reference

All endpoints are relative to `http://localhost:8080` (default). All request/response bodies are JSON.

---

## Health & Info

### `GET /health`

Returns server health status.

**Response**
```json
{ "status": "ok", "version": "0.1.0" }
```

### `GET /v1/sandbox/info`

Returns sandbox metadata.

**Response**
```json
{ "name": "cage-bro", "version": "0.1.0", "runtime": "process", "status": "running" }
```

---

## Shell

### `POST /v1/shell/exec`

Execute a command and return output.

**Request**
```json
{ "command": "ls -la", "timeout_ms": 5000 }
```

| Field | Type | Required | Default |
|---|---|---|---|
| `command` | `string` | yes | -- |
| `timeout_ms` | `u64` | no | none |

**Response**
```json
{ "exit_code": 0, "stdout": "total 0\n...", "stderr": "", "duration_ms": 42 }
```

**Example**
```bash
curl -X POST http://localhost:8080/v1/shell/exec \
  -H 'Content-Type: application/json' \
  -d '{"command": "echo hello"}'
```

### `POST /v1/shell/session`

Create a persistent PTY session.

**Request**
```json
{ "shell": "/bin/bash", "cols": 120, "rows": 40 }
```

| Field | Type | Required | Default |
|---|---|---|---|
| `shell` | `string` | no | system default (zsh on macOS, bash on Linux) |
| `cols` | `u16` | no | -- |
| `rows` | `u16` | no | -- |

**Response**
```json
{ "session_id": "abc-123", "status": "running", "ws_url": "ws://localhost:8080/v1/shell/session/abc-123/ws" }
```

### `GET /v1/shell/session/list`

List active sessions.

**Response**
```json
{ "sessions": ["abc-123", "def-456"] }
```

### `GET /v1/shell/session/{id}/ws`

WebSocket upgrade for interactive terminal. Bidirectional binary frames: client sends input bytes, server sends PTY output bytes.

### `POST /v1/shell/session/{id}/close`

Close a session.

**Response**
```json
{ "status": "closed" }
```

---

## Files

### `POST /v1/file/read`

Read a file. Sandbox-scoped to workspace directory.

**Request**
```json
{ "path": "src/main.rs" }
```

| Field | Type | Required |
|---|---|---|
| `path` | `string` | yes |

**Response**
```json
{ "path": "src/main.rs", "content": "fn main() { ... }", "encoding": "utf-8" }
```

### `POST /v1/file/write`

Write content to a file. Creates parent directories automatically.

**Request**
```json
{ "path": "output.txt", "content": "hello world" }
```

| Field | Type | Required |
|---|---|---|
| `path` | `string` | yes |
| `content` | `string` | yes |

**Response**
```json
{ "status": "ok" }
```

### `POST /v1/file/edit`

Replace the first occurrence of `old_text` with `new_text` in a file.

**Request**
```json
{ "path": "config.json", "old_text": "\"debug\": false", "new_text": "\"debug\": true" }
```

| Field | Type | Required |
|---|---|---|
| `path` | `string` | yes |
| `old_text` | `string` | yes |
| `new_text` | `string` | yes |

**Response**
```json
{ "status": "ok" }
```

### `POST /v1/file/list`

List directory contents.

**Request**
```json
{ "path": "src" }
```

| Field | Type | Required |
|---|---|---|
| `path` | `string` | yes |

**Response**
```json
{
  "entries": [
    { "path": "src/main.rs", "name": "main.rs", "is_dir": false, "size": 1234, "modified": "2026-05-17T..." }
  ]
}
```

### `POST /v1/file/search`

Search file contents with case-insensitive text matching.

**Request**
```json
{ "query": "TODO", "path": "src", "file_pattern": "*.rs", "max_results": 50 }
```

| Field | Type | Required | Default |
|---|---|---|---|
| `query` | `string` | yes | -- |
| `path` | `string` | no | workspace root |
| `file_pattern` | `string` | no | all files |
| `max_results` | `usize` | no | unlimited |

**Response**
```json
{
  "results": [
    { "path": "src/main.rs", "line_number": 42, "line_content": "// TODO: fix this" }
  ]
}
```

### `POST /v1/file/delete`

Delete a file or directory (recursive for directories).

**Request**
```json
{ "path": "temp/" }
```

| Field | Type | Required |
|---|---|---|
| `path` | `string` | yes |

**Response**
```json
{ "status": "ok" }
```

---

## Code Execution

### `POST /v1/code/python`

Execute Python code statelessly.

**Request**
```json
{ "code": "print(2 + 2)", "timeout_ms": 10000 }
```

| Field | Type | Required | Default |
|---|---|---|---|
| `code` | `string` | yes | -- |
| `timeout_ms` | `u64` | no | none |

**Response**
```json
{ "stdout": "4\n", "stderr": "", "exit_code": 0, "duration_ms": 150 }
```

### `POST /v1/code/node`

Execute Node.js code statelessly. Same request/response shape as Python.

### `POST /v1/code/jupyter/start`

Start a stateful Jupyter kernel.

**Request**
```json
{ "language": "python" }
```

| Field | Type | Required | Default |
|---|---|---|---|
| `language` | `string` | no | `"python"` |

**Response**
```json
{ "kernel_id": "k-abc123", "language": "python", "status": "ready" }
```

### `POST /v1/code/jupyter/execute`

Execute code on a running kernel. State persists between calls.

**Request**
```json
{ "kernel_id": "k-abc123", "code": "x = 42\nprint(x)" }
```

| Field | Type | Required |
|---|---|---|
| `kernel_id` | `string` | yes |
| `code` | `string` | yes |

**Response**
```json
{ "stdout": "42\n", "stderr": "", "exit_code": 0, "duration_ms": 89 }
```

### `POST /v1/code/jupyter/interrupt`

Interrupt a running kernel.

**Request**
```json
{ "kernel_id": "k-abc123" }
```

**Response**
```json
{ "status": "interrupted" }
```

### `POST /v1/code/jupyter/shutdown`

Shutdown a kernel and clean up resources.

**Request**
```json
{ "kernel_id": "k-abc123" }
```

**Response**
```json
{ "status": "shutdown" }
```

### `POST /v1/code/jupyter/list`

List active Jupyter kernels.

**Response**
```json
{
  "kernels": [
    { "kernel_id": "k-abc123", "language": "python", "status": "ready" }
  ]
}
```

---

## Browser

Requires `cage-bro setup` to install the Obscura headless browser.

### `POST /v1/browser/launch`

Launch the headless browser.

**Request**
```json
{ "port": 9333, "stealth": true }
```

| Field | Type | Required | Default |
|---|---|---|---|
| `port` | `u16` | no | 9333 |
| `stealth` | `bool` | no | false |

**Response**
```json
{ "status": "launched", "message": "Browser launched successfully" }
```

### `POST /v1/browser/navigate`

Navigate to a URL and extract page content.

**Request**
```json
{ "url": "https://example.com" }
```

| Field | Type | Required |
|---|---|---|
| `url` | `string` | yes |

**Response**
```json
{ "url": "https://example.com", "title": "Example Domain", "text": "Example Domain\n..." }
```

### `POST /v1/browser/screenshot`

Capture a screenshot of the current page.

**Request**
```json
{ "quality": 80 }
```

| Field | Type | Required | Default |
|---|---|---|---|
| `quality` | `u32` | no | 80 |

**Response**
```json
{ "data": "base64...", "format": "png", "width": 1280, "height": 720 }
```

### `POST /v1/browser/click`

Click an element by CSS selector.

**Request**
```json
{ "selector": "#submit-btn" }
```

| Field | Type | Required |
|---|---|---|
| `selector` | `string` | yes |

**Response**
```json
{ "status": "ok" }
```

### `POST /v1/browser/type`

Type text into an element. Focuses the element, sets its value, and dispatches input/change events.

**Request**
```json
{ "selector": "input[name=email]", "text": "user@example.com" }
```

| Field | Type | Required |
|---|---|---|
| `selector` | `string` | yes |
| `text` | `string` | yes |

**Response**
```json
{ "status": "ok" }
```

### `POST /v1/browser/evaluate`

Evaluate JavaScript in the browser context.

**Request**
```json
{ "expression": "document.title" }
```

| Field | Type | Required |
|---|---|---|
| `expression` | `string` | yes |

**Response**
```json
{ "result": "Example Domain" }
```

### `POST /v1/browser/content`

Get current page content (URL, title, text).

**Response**
```json
{ "url": "https://example.com", "title": "Example Domain", "text": "..." }
```

### `POST /v1/browser/close`

Close the browser.

**Response**
```json
{ "status": "closed" }
```

---

## MCP Server

### Starting

```bash
# stdio mode (for Claude Desktop, Cursor)
cage-bro mcp

# HTTP/SSE mode
cage-bro mcp --http --port 8081
```

### Protocol

JSON-RPC 2.0 over stdio (line-delimited) or HTTP/SSE at `/mcp`.

**Methods**: `initialize`, `tools/list`, `tools/call`, `ping`

### Tools (15 total)

| Tool | Parameters | Description |
|---|---|---|
| `shell_exec` | `command: string` | Execute a shell command |
| `file_read` | `path: string` | Read a file |
| `file_write` | `path: string, content: string` | Write a file |
| `file_edit` | `path: string, old_text: string, new_text: string` | Edit a file with string replacement |
| `file_list` | `path: string` | List directory contents |
| `file_search` | `query: string, path?: string` | Search files for text |
| `python_exec` | `code: string` | Execute Python (30s timeout) |
| `node_exec` | `code: string` | Execute Node.js (30s timeout) |
| `browser_navigate` | `url: string` | Navigate browser to URL |
| `browser_screenshot` | -- | Take a screenshot |
| `browser_click` | `selector: string` | Click element by CSS selector |
| `browser_type` | `selector: string, text: string` | Type into an element |
| `browser_evaluate` | `expression: string` | Evaluate JavaScript |
| `browser_snapshot` | -- | Get current page content |
| `sandbox_info` | -- | Get sandbox info |

### Claude Desktop Config

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

---

## SDK: Python

```bash
pip install cage-bro
```

### `CageBro(base_url, timeout)`

```python
from cage_bro import CageBro

cage = CageBro("http://localhost:8080", timeout=30.0)
```

Supports context manager: `with CageBro() as cage: ...`

### Shell

| Method | Signature | Returns |
|---|---|---|
| `shell_exec` | `(command: str, timeout_ms: int = None)` | `dict` with `exit_code`, `stdout`, `stderr`, `duration_ms` |
| `shell_create_session` | `(shell: str = None)` | `dict` with `session_id`, `ws_url` |

### Files

| Method | Signature | Returns |
|---|---|---|
| `file_read` | `(path: str)` | `str` (file content) |
| `file_write` | `(path: str, content: str)` | `None` |
| `file_edit` | `(path: str, old_text: str, new_text: str)` | `None` |
| `file_list` | `(path: str = ".")` | `list[dict]` with `path`, `name`, `is_dir`, `size` |
| `file_search` | `(query: str, path: str = None)` | `list[dict]` with `path`, `line_number`, `line_content` |
| `file_delete` | `(path: str)` | `None` |

### Code

| Method | Signature | Returns |
|---|---|---|
| `python` | `(code: str, timeout_ms: int = None)` | `dict` with `stdout`, `stderr`, `exit_code`, `duration_ms` |
| `node` | `(code: str, timeout_ms: int = None)` | `dict` with `stdout`, `stderr`, `exit_code`, `duration_ms` |

### Browser

| Method | Signature | Returns |
|---|---|---|
| `browser_launch` | `(port: int = None, stealth: bool = True)` | `dict` |
| `browser_navigate` | `(url: str)` | `dict` with `url`, `title`, `text` |
| `browser_screenshot` | `()` | `dict` with `data`, `format`, `width`, `height` |
| `browser_click` | `(selector: str)` | `dict` |
| `browser_type` | `(selector: str, text: str)` | `dict` |
| `browser_evaluate` | `(expression: str)` | `Any` |
| `browser_content` | `()` | `dict` with `url`, `title`, `text` |
| `browser_close` | `()` | `None` |

---

## SDK: TypeScript

```bash
npm install @cage-bro/sdk
```

### `CageBro(options?)`

```typescript
import { CageBro } from "@cage-bro/sdk";

const cage = new CageBro({ baseUrl: "http://localhost:8080", timeout: 30000 });
```

### Shell

| Method | Signature | Returns |
|---|---|---|
| `shellExec` | `(command: string, timeoutMs?: number)` | `Promise<ShellResult>` |
| `shellCreateSession` | `(shell?: string)` | `Promise<{ session_id, ws_url }>` |

### Files

| Method | Signature | Returns |
|---|---|---|
| `fileRead` | `(path: string)` | `Promise<string>` |
| `fileWrite` | `(path: string, content: string)` | `Promise<void>` |
| `fileEdit` | `(path: string, oldText: string, newText: string)` | `Promise<void>` |
| `fileList` | `(path?: string)` | `Promise<FileEntry[]>` |
| `fileSearch` | `(query: string, path?: string)` | `Promise<SearchResult[]>` |
| `fileDelete` | `(path: string)` | `Promise<void>` |

### Code

| Method | Signature | Returns |
|---|---|---|
| `python` | `(code: string, timeoutMs?: number)` | `Promise<ShellResult>` |
| `node` | `(code: string, timeoutMs?: number)` | `Promise<ShellResult>` |

### Browser

| Method | Signature | Returns |
|---|---|---|
| `browserLaunch` | `(port?: number, stealth?: boolean)` | `Promise<Record>` |
| `browserNavigate` | `(url: string)` | `Promise<PageContent>` |
| `browserScreenshot` | `()` | `Promise<ScreenshotData>` |
| `browserClick` | `(selector: string)` | `Promise<void>` |
| `browserType` | `(selector: string, text: string)` | `Promise<void>` |
| `browserEvaluate` | `(expression: string)` | `Promise<unknown>` |
| `browserContent` | `()` | `Promise<PageContent>` |
| `browserClose` | `()` | `Promise<void>` |

### Interfaces

```typescript
interface ShellResult {
  exit_code: number;
  stdout: string;
  stderr: string;
  duration_ms: number;
}

interface FileEntry {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
}

interface SearchResult {
  path: string;
  line_number: number;
  line_content: string;
}

interface PageContent {
  url: string;
  title: string;
  text: string;
}

interface ScreenshotData {
  data: string;     // base64
  format: string;
  width: number;
  height: number;
}
```
