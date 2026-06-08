const rows = [
  { concern: "Read/write another agent's workspace", cb: "blocked", deploy: "—" },
  { concern: "Write outside the workspace (W + /tmp only)", cb: "blocked", deploy: "—" },
  { concern: "Runaway CPU / memory / output", cb: "rlimits + timeout + output cap", deploy: "cgroup pids.max" },
  { concern: "Shared /tmp, readable /proc", cb: "warn", deploy: "separate OS user per agent" },
  { concern: "Network exfiltration", cb: "no", deploy: "host firewall / egress allowlist" },
  { concern: "Kernel-exploit escape", cb: "no", deploy: "run inside a VM (Scenario B)" },
]

function Badge({ kind }: { kind: string }) {
  if (kind === "blocked" || kind.startsWith("rlimits")) {
    return (
      <span className="inline-flex items-center gap-1.5 font-mono text-xs text-terminal">
        <span className="size-1.5 rounded-full bg-terminal" aria-hidden />
        {kind === "blocked" ? "blocked" : kind}
      </span>
    )
  }
  if (kind === "warn") {
    return (
      <span className="inline-flex items-center gap-1.5 font-mono text-xs text-[oklch(0.8_0.15_85)]">
        <span className="size-1.5 rounded-full bg-[oklch(0.8_0.15_85)]" aria-hidden />
        allowed
      </span>
    )
  }
  return (
    <span className="inline-flex items-center gap-1.5 font-mono text-xs text-[oklch(0.62_0.2_25)]">
      <span className="size-1.5 rounded-full bg-[oklch(0.62_0.2_25)]" aria-hidden />
      not filtered
    </span>
  )
}

export function SecuritySection() {
  return (
    <section id="security" className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-20">
        <span className="font-mono text-xs uppercase tracking-wider text-rust">Security model</span>
        <h2 className="mt-3 font-mono text-3xl font-medium tracking-tight text-foreground">
          Guaranteed vs. your responsibility
        </h2>
        <p className="mt-4 max-w-2xl text-pretty leading-relaxed text-muted-foreground">
          For regulated or financial workloads, the clean setup is one OS user + one cage-bro per agent, the
          whole box inside a VM, with egress locked down.
        </p>

        {/* Desktop: three-column table */}
        <div className="mt-10 hidden overflow-hidden rounded-lg border border-border md:block">
          <div className="grid grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)_minmax(0,1fr)] border-b border-border bg-surface font-mono text-[11px] uppercase tracking-wider text-muted-foreground">
            <div className="border-r border-border px-5 py-3">Concern</div>
            <div className="border-r border-border px-5 py-3">cage-bro</div>
            <div className="px-5 py-3">Your deploy layer</div>
          </div>
          {rows.map((r, i) => (
            <div
              key={r.concern}
              className={`grid grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)_minmax(0,1fr)] ${
                i < rows.length - 1 ? "border-b border-border" : ""
              }`}
            >
              <div className="border-r border-border px-5 py-4 text-sm text-foreground/90">{r.concern}</div>
              <div className="border-r border-border px-5 py-4">
                <Badge kind={r.cb} />
              </div>
              <div className="px-5 py-4 font-mono text-xs text-muted-foreground">{r.deploy}</div>
            </div>
          ))}
        </div>

        {/* Mobile: stacked cards */}
        <div className="mt-10 flex flex-col gap-3 md:hidden">
          {rows.map((r) => (
            <div key={r.concern} className="rounded-lg border border-border bg-surface-2 p-4">
              <div className="text-sm text-foreground/90">{r.concern}</div>
              <div className="mt-3 flex items-center justify-between gap-3">
                <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground/60">
                  cage-bro
                </span>
                <Badge kind={r.cb} />
              </div>
              {r.deploy !== "—" && (
                <div className="mt-2 flex items-center justify-between gap-3">
                  <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground/60">
                    Your deploy layer
                  </span>
                  <span className="text-right font-mono text-xs text-muted-foreground">{r.deploy}</span>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
