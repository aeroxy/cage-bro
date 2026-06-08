const tools = [
  { name: "shell_exec", desc: "Run a shell command (jailed to the workspace)", tag: "shell" },
  { name: "python_exec", desc: "Run a Python snippet (jailed)", tag: "shell" },
  { name: "node_exec", desc: "Run a Node snippet (jailed)", tag: "shell" },
  { name: "file_read", desc: "Read a file within the workspace", tag: "file" },
  { name: "file_write", desc: "Write a file within the workspace", tag: "file" },
  { name: "file_edit", desc: "Edit a file within the workspace", tag: "file" },
  { name: "file_list", desc: "List files within the workspace", tag: "file" },
  { name: "file_search", desc: "Grep within the workspace", tag: "file" },
  { name: "browser_navigate", desc: "Drive the headless Obscura browser", tag: "browser" },
  { name: "browser_screenshot", desc: "Capture the browser viewport", tag: "browser" },
  { name: "browser_snapshot", desc: "Snapshot the accessibility tree", tag: "browser" },
  { name: "sandbox_info", desc: "Read sandbox metadata", tag: "meta" },
]

const tagColor: Record<string, string> = {
  shell: "text-terminal",
  file: "text-foreground/70",
  browser: "text-browser",
  meta: "text-rust",
}

const commands = [
  "cage-bro serve --port 8080",
  "cage-bro mcp",
  "cage-bro mcp --http --port 8081",
  "cage-bro setup",
]

export function ReferenceSection() {
  return (
    <section className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-20">
        <span className="font-mono text-xs uppercase tracking-wider text-rust">Reference</span>
        <h2 className="mt-3 font-mono text-3xl font-medium tracking-tight text-foreground">
          Commands &amp; MCP tools
        </h2>

        <div className="mt-10 grid gap-10 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)]">
          <div>
            <h3 className="font-mono text-sm text-muted-foreground">Commands</h3>
            <div className="mt-4 flex flex-col gap-2">
              {commands.map((c) => (
                <div
                  key={c}
                  className="rounded-md border border-border bg-surface px-4 py-3 font-mono text-sm text-foreground/90"
                >
                  <span className="select-none text-rust">$ </span>
                  {c}
                </div>
              ))}
            </div>
          </div>

          <div>
            <h3 className="font-mono text-sm text-muted-foreground">MCP tools</h3>
            <div className="mt-4 grid gap-px overflow-hidden rounded-md border border-border bg-border sm:grid-cols-2">
              {tools.map((t) => (
                <div key={t.name} className="bg-surface px-4 py-3">
                  <div className="flex items-center justify-between">
                    <code className={`font-mono text-sm ${tagColor[t.tag]}`}>{t.name}</code>
                    <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground/60">
                      {t.tag}
                    </span>
                  </div>
                  <p className="mt-1 text-xs leading-relaxed text-muted-foreground">{t.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
