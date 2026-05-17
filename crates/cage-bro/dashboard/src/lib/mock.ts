const MOCK = {
  health: { status: "ok", version: "0.1.0" },
  sandbox_info: {
    name: "cage-bro",
    version: "0.1.0",
    runtime: "process",
    status: "running",
  },
  shell_exec: {
    exit_code: 0,
    stdout: "total 24\ndrwxr-xr-x  6 user  staff  192 May 17 12:00 .\ndrwxr-xr-x  3 user  staff   96 May 17 11:00 ..\n-rw-r--r--  1 user  staff  128 May 17 12:00 README.md\ndrwxr-xr-x  2 user  staff   64 May 17 12:00 src\n",
    stderr: "",
    duration_ms: 12,
  },
  file_list: {
    entries: [
      { path: "README.md", name: "README.md", is_dir: false, size: 128 },
      { path: "src", name: "src", is_dir: true, size: 0 },
      { path: "src/main.rs", name: "main.rs", is_dir: false, size: 2048 },
      { path: "src/lib.rs", name: "lib.rs", is_dir: false, size: 512 },
      { path: "Cargo.toml", name: "Cargo.toml", is_dir: false, size: 1024 },
    ],
  },
  file_read: {
    path: "src/main.rs",
    content: `use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    cli::run().await
}`,
    encoding: "utf-8",
  },
  file_write: { status: "ok" },
  file_edit: { status: "ok" },
  file_search: {
    results: [
      { path: "src/main.rs", line_number: 1, line_content: "use anyhow::Result;" },
      { path: "src/lib.rs", line_number: 1, line_content: "pub mod server;" },
      { path: "Cargo.toml", line_number: 1, line_content: "[package]" },
    ],
  },
  python: {
    stdout: "Hello from cage-bro!\nPython 3.11.14\n",
    stderr: "",
    exit_code: 0,
    duration_ms: 45,
  },
  node: {
    stdout: "Hello from cage-bro!\nv25.2.1\n",
    stderr: "",
    exit_code: 0,
    duration_ms: 120,
  },
  browser_launch: { status: "ok", message: "Browser running on port 9333" },
  browser_navigate: {
    url: "https://github.com/aeroxy/cage-bro",
    title: "aeroxy/cage-bro",
    text: "cage-bro\n\nA sandboxed execution environment for AI agents. Single Rust binary with browser, shell, code execution, file ops, and MCP server.\n\nStars 0\nLicense Apache-2.0\n\nFeatures:\n- Shell: PTY-based terminal sessions via WebSocket\n- Browser: Obscura headless browser (stealth mode, CDP)\n- Code: Python, Node.js (stateless) + Jupyter (stateful)\n- Files: Read, write, edit, list, search with sandbox scope\n- MCP: Built-in MCP server for Claude Desktop, Cursor\n- Dashboard: Web UI with terminal, code editor, file browser",
  },
  browser_screenshot: {
    data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
    format: "png",
    width: 1280,
    height: 720,
  },
  browser_content: {
    url: "https://github.com/aeroxy/cage-bro",
    title: "aeroxy/cage-bro",
    text: "cage-bro — Sandboxed execution environment for AI agents",
  },
  shell_session: {
    session_id: "demo-session-0000",
    status: "created",
    ws_url: "/v1/shell/session/demo-session-0000/ws",
  },
  shell_session_list: { sessions: ["demo-session-0000"] },
}

export type MockKey = keyof typeof MOCK

export function getMockResponse(path: string): any {
  // Match path to mock key
  if (path.includes("/health")) return MOCK.health
  if (path.includes("/sandbox/info")) return MOCK.sandbox_info
  if (path.includes("/shell/exec")) return MOCK.shell_exec
  if (path.includes("/shell/session/list")) return MOCK.shell_session_list
  if (path.includes("/shell/session") && path.includes("/close")) return { status: "closed" }
  if (path.includes("/shell/session")) return MOCK.shell_session
  if (path.includes("/file/list")) return MOCK.file_list
  if (path.includes("/file/read")) return MOCK.file_read
  if (path.includes("/file/write")) return MOCK.file_write
  if (path.includes("/file/edit")) return MOCK.file_edit
  if (path.includes("/file/search")) return MOCK.file_search
  if (path.includes("/file/delete")) return { status: "ok" }
  if (path.includes("/code/python")) return MOCK.python
  if (path.includes("/code/node")) return MOCK.node
  if (path.includes("/browser/launch")) return MOCK.browser_launch
  if (path.includes("/browser/navigate")) return MOCK.browser_navigate
  if (path.includes("/browser/screenshot")) return MOCK.browser_screenshot
  if (path.includes("/browser/content")) return MOCK.browser_content
  if (path.includes("/browser/close")) return { status: "closed" }
  if (path.includes("/browser/click")) return { status: "ok" }
  if (path.includes("/browser/type")) return { status: "ok" }
  if (path.includes("/browser/evaluate")) return { result: "42" }
  return { error: "Unknown endpoint" }
}
