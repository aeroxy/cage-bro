const dashboardHref = `${import.meta.env.BASE_URL}dashboard/`

function Wordmark() {
  return (
    <a href="#top" className="flex items-center gap-2 font-mono text-sm font-medium tracking-tight">
      <span className="flex size-6 items-center justify-center rounded border border-rust/40 bg-rust/10 text-rust">
        {'['}
      </span>
      <span className="text-foreground">
        cage<span className="text-rust">-</span>bro
      </span>
    </a>
  )
}

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-50 border-b border-border bg-background/80 backdrop-blur-md">
      <div className="mx-auto flex h-14 max-w-6xl items-center justify-between gap-3 px-4 sm:px-6">
        <div className="flex items-center gap-2 sm:gap-3">
          <Wordmark />
          <span className="rounded border border-rust/40 bg-rust/10 px-1.5 py-0.5 font-mono text-[10px] font-medium uppercase tracking-wider text-rust">
            v0.2.0
          </span>
        </div>
        <nav className="flex items-center gap-4 font-mono text-xs text-muted-foreground sm:gap-6">
          <a href="#guide" className="hidden transition-colors hover:text-foreground md:inline">
            Guide
          </a>
          <a href="#fleet" className="hidden transition-colors hover:text-foreground md:inline">
            Fleet
          </a>
          <a href="#security" className="hidden transition-colors hover:text-foreground md:inline">
            Security
          </a>
          <a
            href={dashboardHref}
            className="rounded-md border border-border bg-surface px-3 py-1.5 text-foreground transition-colors hover:border-rust/50 hover:text-rust"
          >
            Dashboard
          </a>
          <a
            href="https://github.com/aeroxy/cage-bro"
            target="_blank"
            rel="noreferrer"
            className="hidden rounded-md border border-border bg-surface px-3 py-1.5 text-foreground transition-colors hover:border-rust/50 hover:text-rust sm:inline"
          >
            GitHub
          </a>
        </nav>
      </div>
    </header>
  )
}
