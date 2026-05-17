# cage-bro

Python SDK for [cage-bro](https://github.com/aeroxy/cage-bro) — a sandboxed execution environment for AI agents.

## Install

```bash
pip install cage-bro
```

## Quick Start

```python
from cage_bro import CageBro

cage = CageBro("http://localhost:8080")

# Run shell commands
result = cage.shell_exec("ls -la")
print(result["stdout"])

# Execute code
result = cage.python("print(2 + 2)")
print(result["stdout"])

# Read/write files
cage.file_write("hello.txt", "world")
content = cage.file_read("hello.txt")
```

## API

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
| `file_read(path)` | Read a file |
| `file_write(path, content)` | Write to a file |
| `file_edit(path, old_text, new_text)` | Edit a file (find & replace) |
| `file_list(path=".")` | List directory contents |
| `file_search(query, path=None)` | Search files for text |
| `file_delete(path)` | Delete a file or directory |

### Code Execution

| Method | Description |
|---|---|
| `python(code, timeout_ms=None)` | Execute Python code |
| `node(code, timeout_ms=None)` | Execute Node.js code |

### Browser

| Method | Description |
|---|---|
| `browser_launch(port=None, stealth=True)` | Launch the browser |
| `browser_navigate(url)` | Navigate to a URL |
| `browser_screenshot()` | Take a screenshot |
| `browser_click(selector)` | Click an element |
| `browser_type(selector, text)` | Type text into an element |
| `browser_evaluate(expression)` | Evaluate JavaScript |
| `browser_content()` | Get current page content |
| `browser_close()` | Close the browser |

## Context Manager

```python
with CageBro("http://localhost:8080") as cage:
    cage.shell_exec("echo hello")
```

## License

Apache-2.0
