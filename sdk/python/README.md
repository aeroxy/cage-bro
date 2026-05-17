# cage-bro Python SDK

[![PyPI](https://img.shields.io/pypi/v/cage-bro)](https://pypi.org/project/cage-bro/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

Python SDK for [cage-bro](https://github.com/aeroxy/cage-bro) — a sandboxed execution environment for AI agents with browser, shell, code execution, file ops, and MCP support.

## Install

```bash
pip install cage-bro
```

Requires a running cage-bro server. See the [main project](https://github.com/aeroxy/cage-bro) for installation instructions.

## Quick Start

```python
from cage_bro import CageBro

cage = CageBro("http://localhost:8080")

# Shell commands
result = cage.shell_exec("ls -la")
print(result["stdout"])

# Code execution
result = cage.python("print(2 + 2)")
print(result["stdout"])

# File operations
cage.file_write("hello.txt", "world")
content = cage.file_read("hello.txt")

# Browser automation
cage.browser_launch()
cage.browser_navigate("https://example.com")
screenshot = cage.browser_screenshot()
```

## Context Manager

```python
with CageBro("http://localhost:8080") as cage:
    cage.shell_exec("echo hello")
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
| `shell_exec(command, timeout_ms=None)` | Execute a shell command |
| `shell_create_session(shell=None)` | Create a persistent shell session |

### Files

| Method | Description |
|---|---|
| `file_read(path)` | Read a file and return its content |
| `file_write(path, content)` | Write content to a file |
| `file_edit(path, old_text, new_text)` | Edit a file (find and replace) |
| `file_list(path=".")` | List directory contents |
| `file_search(query, path=None)` | Search files for text |
| `file_delete(path)` | Delete a file or directory |

### Code Execution

| Method | Description |
|---|---|
| `python(code, timeout_ms=None)` | Execute Python code in the sandbox |
| `node(code, timeout_ms=None)` | Execute Node.js code in the sandbox |

### Browser

| Method | Description |
|---|---|
| `browser_launch(port=None, stealth=True)` | Launch the headless browser |
| `browser_navigate(url)` | Navigate to a URL |
| `browser_screenshot()` | Take a screenshot of the current page |
| `browser_click(selector)` | Click an element by CSS selector |
| `browser_type(selector, text)` | Type text into an input element |
| `browser_evaluate(expression)` | Evaluate JavaScript in the browser |
| `browser_content()` | Get the current page HTML content |
| `browser_close()` | Close the browser |

## Links

- [cage-bro main project](https://github.com/aeroxy/cage-bro)
- [TypeScript SDK](https://www.npmjs.com/package/@cage-bro/sdk)
- [API documentation](https://github.com/aeroxy/cage-bro#api)

## License

Apache-2.0
