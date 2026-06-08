const tree = [
  { depth: 0, label: "Mac mini", color: "text-foreground", connector: "" },
  { depth: 1, label: "alice/   zeroclaw  (file/shell tools OFF)", color: "text-foreground/80", connector: "├─" },
  { depth: 2, label: "MCP → cage-bro :8081 → jail: /srv/alice/workspace", color: "text-terminal", connector: "│   └─" },
  { depth: 1, label: "bob/     zeroclaw  (file/shell tools OFF)", color: "text-foreground/80", connector: "├─" },
  { depth: 2, label: "MCP → cage-bro :8082 → jail: /srv/bob/workspace", color: "text-terminal", connector: "│   └─" },
  { depth: 1, label: "… up to ~20 agents, each on its own port and folder", color: "text-muted-foreground", connector: "└─" },
]

export function PhilosophySection() {
  return (
    <section className="border-b border-border">
      <div className="mx-auto grid max-w-6xl gap-12 px-6 py-20 lg:grid-cols-2 lg:items-center">
        <div className="flex flex-col gap-4">
          <span className="font-mono text-xs uppercase tracking-wider text-rust">Philosophy</span>
          <h2 className="text-balance font-mono text-3xl font-medium tracking-tight text-foreground">
            Not a heavy VM. A workspace jail.
          </h2>
          <p className="max-w-md text-pretty leading-relaxed text-muted-foreground">
            Each sandbox is a confined process — about 100 MB and ~1 ms to start — so a small box can hold
            dozens, where one container-per-agent would need tens of GB.
          </p>
          <p className="max-w-md text-pretty leading-relaxed text-muted-foreground">
            The idea is simple: turn off the agent framework&apos;s own file and shell tools, and give it
            cage-bro&apos;s MCP server as the only way to touch files or run commands. cage-bro then keeps every
            one of those operations inside the agent&apos;s workspace.
          </p>
        </div>

        <div className="overflow-x-auto rounded-lg border border-border bg-[oklch(0.13_0_0)] p-5 font-mono text-xs leading-6 sm:text-sm">
          {tree.map((t, i) => (
            <div key={i} className="whitespace-pre">
              <span className="text-muted-foreground/50">{t.connector} </span>
              <span className={t.color}>{t.label}</span>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
