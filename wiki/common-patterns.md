# Common Patterns

The three most common use cases for cage-bro, with working code.

---

## Pattern 1: AI Agent with Shell + File Access

Use cage-bro as a sandboxed execution backend for an AI agent that needs to run commands and read/write files without touching the host system.

### REST API

```bash
# Run a command
curl -s -X POST http://localhost:8080/v1/shell/exec \
  -H 'Content-Type: application/json' \
  -d '{"command": "python3 --version"}'

# Write a script
curl -s -X POST http://localhost:8080/v1/file/write \
  -H 'Content-Type: application/json' \
  -d '{"path": "scripts/hello.py", "content": "print(\"Hello from sandbox!\")"}'

# Execute it
curl -s -X POST http://localhost:8080/v1/shell/exec \
  -H 'Content-Type: application/json' \
  -d '{"command": "python3 scripts/hello.py"}'

# Read the output or other files
curl -s -X POST http://localhost:8080/v1/file/read \
  -H 'Content-Type: application/json' \
  -d '{"path": "scripts/hello.py"}'
```

### Python SDK

```python
from cage_bro import CageBro

with CageBro("http://localhost:8080") as cage:
    # Write a file
    cage.file_write("data.csv", "name,age\nAlice,30\nBob,25")

    # Run analysis
    result = cage.shell_exec("wc -l data.csv")
    print(f"Lines: {result['stdout'].strip()}")

    # Edit the file
    cage.file_edit("data.csv", "name,age", "name,age,city")

    # Search for content
    matches = cage.file_search("Alice", path="data.csv")
    for m in matches:
        print(f"{m['path']}:{m['line_number']}: {m['line_content']}")
```

### TypeScript SDK

```typescript
import { CageBro } from "@cage-bro/sdk";

const cage = new CageBro({ baseUrl: "http://localhost:8080" });

// Write and execute
await cage.fileWrite("app.js", "console.log('hello')");
const result = await cage.shellExec("node app.js");
console.log(result.stdout); // "hello\n"

// List what we created
const files = await cage.fileList(".");
console.log(files.map(f => f.name)); // ["app.js", ...]
```

---

## Pattern 2: Browser Automation for Web Scraping / Testing

Launch a headless browser, navigate pages, interact with elements, and extract content -- all inside the sandbox.

### REST API

```bash
# Launch browser (stealth mode avoids bot detection)
curl -s -X POST http://localhost:8080/v1/browser/launch \
  -H 'Content-Type: application/json' \
  -d '{"stealth": true}'

# Navigate to a page
curl -s -X POST http://localhost:8080/v1/browser/navigate \
  -H 'Content-Type: application/json' \
  -d '{"url": "https://news.ycombinator.com"}'

# Extract page text
curl -s -X POST http://localhost:8080/v1/browser/content

# Take a screenshot
curl -s -X POST http://localhost:8080/v1/browser/screenshot

# Click a link
curl -s -X POST http://localhost:8080/v1/browser/click \
  -H 'Content-Type: application/json' \
  -d '{"selector": ".titleline > a"}'

# Evaluate custom JS
curl -s -X POST http://localhost:8080/v1/browser/evaluate \
  -H 'Content-Type: application/json' \
  -d '{"expression": "document.querySelectorAll(\".titleline\").length"}'
```

### Python SDK

```python
from cage_bro import CageBro

with CageBro("http://localhost:8080") as cage:
    cage.browser_launch(stealth=True)

    # Scrape a page
    page = cage.browser_navigate("https://example.com")
    print(page["title"])  # "Example Domain"

    # Interact with elements
    cage.browser_type("input[name=q]", "cage-bro sandbox")
    cage.browser_click("button[type=submit]")

    # Get the result
    content = cage.browser_content()
    print(content["text"][:200])

    # Screenshot for debugging
    screenshot = cage.browser_screenshot()
    import base64
    with open("page.png", "wb") as f:
        f.write(base64.b64decode(screenshot["data"]))

    cage.browser_close()
```

### TypeScript SDK

```typescript
import { CageBro } from "@cage-bro/sdk";
import { writeFileSync } from "fs";

const cage = new CageBro();

await cage.browserLaunch(undefined, true);
await cage.browserNavigate("https://example.com");

// Evaluate JS to extract structured data
const links = await cage.browser_evaluate(
  "JSON.stringify([...document.querySelectorAll('a')].map(a => ({text: a.textContent, href: a.href})))"
);
console.log(JSON.parse(links as string));

const screenshot = await cage.browserScreenshot();
writeFileSync("page.png", Buffer.from(screenshot.data, "base64"));

await cage.browserClose();
```

---

## Pattern 3: Stateful Code Execution with Jupyter

Run multi-step code with persistent state between executions. Variables, imports, and data survive across calls.

### REST API

```bash
# Start a kernel
curl -s -X POST http://localhost:8080/v1/code/jupyter/start \
  -H 'Content-Type: application/json' \
  -d '{"language": "python"}'
# Response: { "kernel_id": "k-abc123", ... }

# First cell: define variables
curl -s -X POST http://localhost:8080/v1/code/jupyter/execute \
  -H 'Content-Type: application/json' \
  -d '{"kernel_id": "k-abc123", "code": "data = [1, 2, 3, 4, 5]\nprint(f\"Loaded {len(data)} items\")"}'

# Second cell: use those variables (state persists!)
curl -s -X POST http://localhost:8080/v1/code/jupyter/execute \
  -H 'Content-Type: application/json' \
  -d '{"kernel_id": "k-abc123", "code": "import statistics\nprint(f\"Mean: {statistics.mean(data)}\")"}'

# Third cell: still has access
curl -s -X POST http://localhost:8080/v1/code/jupyter/execute \
  -H 'Content-Type: application/json' \
  -d '{"kernel_id": "k-abc123", "code": "print(f\"Sum: {sum(data)}, Max: {max(data)}\")"}'

# List active kernels
curl -s -X POST http://localhost:8080/v1/code/jupyter/list

# Clean up
curl -s -X POST http://localhost:8080/v1/code/jupyter/shutdown \
  -H 'Content-Type: application/json' \
  -d '{"kernel_id": "k-abc123"}'
```

### Python SDK (with stateful Jupyter via REST)

The Python SDK exposes stateless `python()` and `node()` methods directly. For Jupyter, hit the REST API:

```python
import httpx

base = "http://localhost:8080"
r = httpx.post(f"{base}/v1/code/jupyter/start", json={"language": "python"})
kernel_id = r.json()["kernel_id"]

# Build up state across cells
cells = [
    "import json",
    "config = {'env': 'prod', 'retries': 3}",
    "print(json.dumps(config, indent=2))",
]

for code in cells:
    r = httpx.post(f"{base}/v1/code/jupyter/execute", json={"kernel_id": kernel_id, "code": code})
    result = r.json()
    if result.get("stdout"):
        print(result["stdout"])
    if result.get("stderr"):
        print(f"STDERR: {result['stderr']}")

# Clean up
httpx.post(f"{base}/v1/code/jupyter/shutdown", json={"kernel_id": kernel_id})
```

### When to use Jupyter vs stateless

| Use case | Method |
|---|---|
| One-off script execution | `POST /v1/code/python` |
| Multi-step data analysis | `POST /v1/code/jupyter/start` + `execute` |
| Importing heavy libraries once, reusing | Jupyter |
| Quick snippet testing | Stateless `python`/`node` |
| Long-running computation | Jupyter (can interrupt) |

---

## MCP Integration Pattern

Connect cage-bro to Claude Desktop or any MCP-compatible client.

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

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

Claude Desktop gets access to all 15 tools: shell_exec, file_read, file_write, file_edit, file_list, file_search, python_exec, node_exec, browser_navigate, browser_screenshot, browser_click, browser_type, browser_evaluate, browser_snapshot, sandbox_info.

### HTTP mode for remote access

```bash
cage-bro mcp --http --port 8081
```

Clients connect via SSE at `http://localhost:8081/mcp`.
