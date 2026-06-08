import { Terminal, Line, Out, Comment } from "@/components/terminal"

const stats = [
  { value: "~100 MB", label: "per sandbox" },
  { value: "~1 ms", label: "to start" },
  { value: "~20", label: "agents / box" },
]

export function Hero() {
  return (
    <section className="relative overflow-hidden border-b border-border">
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.04]"
        style={{
          backgroundImage:
            "linear-gradient(to right, currentColor 1px, transparent 1px), linear-gradient(to bottom, currentColor 1px, transparent 1px)",
          backgroundSize: "48px 48px",
        }}
        aria-hidden
      />
      <div className="relative mx-auto grid max-w-6xl gap-12 px-6 py-20 lg:grid-cols-2 lg:items-center lg:py-28">
        <div className="flex flex-col items-start gap-6">
          <span className="inline-flex items-center gap-2 rounded-full border border-border bg-surface px-3 py-1 font-mono text-xs text-muted-foreground">
            <span className="size-1.5 rounded-full bg-terminal" aria-hidden />
            workspace jail for AI agents
          </span>
          <h1 className="text-balance font-mono text-4xl font-medium leading-[1.1] tracking-tight text-foreground sm:text-5xl">
            Give an agent a workspace it <span className="text-rust">can&apos;t escape.</span>
          </h1>
          <p className="max-w-md text-pretty leading-relaxed text-muted-foreground">
            cage-bro is a lightweight workspace jail — not a heavy VM sandbox. Turn off the agent
            framework&apos;s own file and shell tools, and give it cage-bro&apos;s MCP server as the only way
            to touch files or run commands.
          </p>
          <div className="flex flex-wrap items-center gap-3">
            <a
              href="#guide"
              className="rounded-md bg-rust px-5 py-2.5 font-mono text-sm font-medium text-primary-foreground transition-opacity hover:opacity-90"
            >
              Read the guide
            </a>
            <a
              href="https://github.com/aeroxy/cage-bro"
              target="_blank"
              rel="noreferrer"
              className="rounded-md border border-border bg-surface px-5 py-2.5 font-mono text-sm text-foreground transition-colors hover:border-rust/50"
            >
              View on GitHub
            </a>
          </div>
          <dl className="mt-2 flex gap-8 border-t border-border pt-6">
            {stats.map((s) => (
              <div key={s.label}>
                <dt className="font-mono text-2xl font-medium text-foreground">{s.value}</dt>
                <dd className="mt-1 font-mono text-xs text-muted-foreground">{s.label}</dd>
              </div>
            ))}
          </dl>
        </div>

        <Terminal title="alice@mac-mini: /srv/alice" className="w-full">
          <Comment>create the folder and start cage-bro there</Comment>
          <Line prompt>sudo mkdir -p /srv/alice/workspace</Line>
          <Line prompt>cd /srv/alice</Line>
          <Line prompt>cage-bro mcp --http --port 8081</Line>
          <Out color="green">{"  MCP listening on http://127.0.0.1:8081/mcp"}</Out>
          <Out color="green">{"  jail → /srv/alice/workspace"}</Out>
          <div className="h-3" />
          <Comment>agent tries to read a peer&apos;s workspace</Comment>
          <Line prompt>file_read /srv/bob/workspace/secret.csv</Line>
          <Out color="muted">{"  Error: path outside sandbox"}</Out>
        </Terminal>
      </div>
    </section>
  )
}
