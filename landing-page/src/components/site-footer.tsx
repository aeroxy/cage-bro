export function SiteFooter() {
  return (
    <footer className="border-b border-border">
      <div className="mx-auto flex max-w-6xl flex-col gap-8 px-6 py-14">
        <div className="flex flex-col items-start justify-between gap-6 sm:flex-row sm:items-center">
          <div>
            <div className="font-mono text-sm font-medium text-foreground">
              cage<span className="text-rust">-</span>bro
            </div>
            <p className="mt-2 max-w-sm text-sm leading-relaxed text-muted-foreground">
              A lightweight workspace jail for AI agents. Process-level isolation, ~1ms to start, one instance
              per agent.
            </p>
          </div>
          <div className="flex flex-wrap gap-3 font-mono text-xs">
            <a
              href={`${import.meta.env.BASE_URL}dashboard/`}
              className="rounded-md border border-rust/40 bg-rust/10 px-4 py-2 text-rust transition-colors hover:border-rust/60"
            >
              Dashboard
            </a>
            <a
              href="https://github.com/aeroxy/cage-bro"
              target="_blank"
              rel="noreferrer"
              className="rounded-md border border-border bg-surface px-4 py-2 text-foreground transition-colors hover:border-rust/50 hover:text-rust"
            >
              GitHub
            </a>
            <a
              href="https://github.com/zeroclaw-labs/zeroclaw"
              target="_blank"
              rel="noreferrer"
              className="rounded-md border border-border bg-surface px-4 py-2 text-muted-foreground transition-colors hover:text-foreground"
            >
              zeroclaw
            </a>
            <a
              href="https://github.com/aeroxy/drift"
              target="_blank"
              rel="noreferrer"
              className="rounded-md border border-border bg-surface px-4 py-2 text-muted-foreground transition-colors hover:text-foreground"
            >
              drift
            </a>
          </div>
        </div>
        <div className="flex flex-col gap-2 border-t border-border pt-6 font-mono text-xs text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
          <span>cage-bro v0.2.0 — Landlock + rlimits + E2B-compatible API</span>
          <span>Built for jailed agents.</span>
        </div>
      </div>
    </footer>
  )
}
