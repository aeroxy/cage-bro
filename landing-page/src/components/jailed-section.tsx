const rows = [
  {
    tool: "file_read / file_write / file_edit / file_list / file_search",
    confinement: "Only inside the workspace W. Any path that resolves outside W (including via ..) is rejected.",
  },
  {
    tool: "shell_exec / python_exec / node_exec",
    confinement:
      "Run with W as the working directory. On Linux, Landlock limits them to read+write W and /tmp, and read+execute a system allowlist (/usr, /bin, /lib, /etc, /proc).",
  },
]

export function JailedSection() {
  return (
    <section id="guide" className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-20">
        <div className="flex flex-col gap-3">
          <span className="font-mono text-xs uppercase tracking-wider text-rust">The guide</span>
          <h2 className="text-balance font-mono text-3xl font-medium tracking-tight text-foreground">
            What &ldquo;jailed&rdquo; means
          </h2>
          <p className="max-w-2xl text-pretty leading-relaxed text-muted-foreground">
            When an agent uses a cage-bro instance whose workspace is{" "}
            <code className="rounded bg-surface px-1.5 py-0.5 font-mono text-sm text-foreground">W</code>, every
            file and shell operation is kept inside that folder. An agent{" "}
            <span className="text-foreground">can&apos;t read or modify another agent&apos;s workspace.</span>
          </p>
        </div>

        {/* Desktop: two-column table */}
        <div className="mt-10 hidden overflow-hidden rounded-lg border border-border sm:block">
          <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1.6fr)] border-b border-border bg-surface font-mono text-xs uppercase tracking-wider text-muted-foreground">
            <div className="border-r border-border px-5 py-3">Tool</div>
            <div className="px-5 py-3">Confinement</div>
          </div>
          {rows.map((r, i) => (
            <div
              key={r.tool}
              className={`grid grid-cols-[minmax(0,1fr)_minmax(0,1.6fr)] ${
                i < rows.length - 1 ? "border-b border-border" : ""
              }`}
            >
              <div className="border-r border-border px-5 py-4 font-mono text-sm text-terminal">{r.tool}</div>
              <div className="px-5 py-4 text-sm leading-relaxed text-muted-foreground">{r.confinement}</div>
            </div>
          ))}
        </div>

        {/* Mobile: stacked cards */}
        <div className="mt-10 flex flex-col gap-3 sm:hidden">
          {rows.map((r) => (
            <div key={r.tool} className="rounded-lg border border-border bg-surface-2 p-4">
              <div className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">Tool</div>
              <div className="mt-1 break-words font-mono text-sm text-terminal">{r.tool}</div>
              <div className="mt-3 font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
                Confinement
              </div>
              <p className="mt-1 text-sm leading-relaxed text-muted-foreground">{r.confinement}</p>
            </div>
          ))}
        </div>

        <div className="mt-6 rounded-lg border border-rust/30 bg-rust/[0.06] p-5">
          <p className="text-sm leading-relaxed text-muted-foreground">
            <span className="font-mono font-medium text-rust">The boundary, honestly:</span> this is{" "}
            <span className="text-foreground">process-level isolation</span>, not a hardware one.{" "}
            <code className="font-mono text-foreground">/tmp</code> is shared and the network is open unless
            your deployment restricts them, and Landlock needs a real Linux kernel. For hostile code or hard
            multi-tenancy, run each agent as its own OS user and/or inside a VM.
          </p>
        </div>
      </div>
    </section>
  )
}
