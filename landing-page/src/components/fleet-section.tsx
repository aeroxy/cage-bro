import { Terminal, Line, Out, Comment } from "@/components/terminal"

const fleetRows = [
  { agent: "alice", user: "agent-alice", ws: "/srv/alice/workspace", mcp: "127.0.0.1:8081" },
  { agent: "bob", user: "agent-bob", ws: "/srv/bob/workspace", mcp: "127.0.0.1:8082" },
  { agent: "carol", user: "agent-carol", ws: "/srv/carol/workspace", mcp: "127.0.0.1:8083" },
]

export function FleetSection() {
  return (
    <section id="fleet" className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-20">
        <div className="flex items-center gap-3">
          <span className="flex size-7 items-center justify-center rounded border border-border bg-surface font-mono text-xs text-rust">
            A
          </span>
          <h2 className="font-mono text-3xl font-medium tracking-tight text-foreground">
            A fleet of agents on one box
          </h2>
        </div>
        <p className="mt-4 max-w-2xl text-pretty leading-relaxed text-muted-foreground">
          Give each agent its own folder, port, and ideally its own OS user — the OS user is what holds the line
          even if an agent process is compromised. That&apos;s N mutually-isolated agents on one machine.
        </p>

        <div className="mt-10 grid gap-6 lg:grid-cols-2">
          <Terminal title="install + jail one agent">
            <Comment>install</Comment>
            <Line prompt>cargo install cage-bro</Line>
            <Out color="muted">{"  # or: brew install aeroxy/tap/cage-bro"}</Out>
            <div className="h-3" />
            <Comment>spin up N jailed agents</Comment>
            <Line prompt>{'for name in alice bob carol; do'}</Line>
            <Line>{'  mkdir -p "/srv/$name/workspace"'}</Line>
            <Line>{'  ( cd "/srv/$name" \\'}</Line>
            <Line>{'    && cage-bro mcp --http --port "$port" & )'}</Line>
            <Line>{"done"}</Line>
            <Out color="green">{"  alice -> MCP 127.0.0.1:8081"}</Out>
            <Out color="green">{"  bob   -> MCP 127.0.0.1:8082"}</Out>
            <Out color="green">{"  carol -> MCP 127.0.0.1:8083"}</Out>
          </Terminal>

          {/* Desktop table view */}
          <div className="hidden overflow-hidden rounded-lg border border-border bg-surface-2 lg:block">
            <div className="grid border-b border-border/50 bg-surface px-1 py-1 font-mono text-[10px] uppercase tracking-widest text-muted-foreground" style={{ gridTemplateColumns: '100px 130px 1fr 120px' }}>
              <div className="border-r border-border/30 px-4 py-2.5">Agent</div>
              <div className="border-r border-border/30 px-4 py-2.5">OS user</div>
              <div className="border-r border-border/30 px-4 py-2.5">Workspace</div>
              <div className="px-4 py-2.5">MCP</div>
            </div>
            {fleetRows.map((r) => (
              <div key={r.agent} className="grid border-b border-border/20 font-mono text-xs last:border-b-0 hover:bg-surface/50" style={{ gridTemplateColumns: '100px 130px 1fr 120px' }}>
                <div className="border-r border-border/20 px-4 py-3.5 text-rust font-medium">{r.agent}</div>
                <div className="border-r border-border/20 px-4 py-3.5 text-muted-foreground">{r.user}</div>
                <div className="border-r border-border/20 px-4 py-3.5 text-foreground/70">{r.ws}</div>
                <div className="px-4 py-3.5 text-browser">{r.mcp}</div>
              </div>
            ))}
          </div>

          {/* Mobile card view */}
          <div className="flex flex-col gap-3 lg:hidden">
            {fleetRows.map((r) => (
              <div key={r.agent} className="rounded-lg border border-border bg-surface-2 p-4">
                <div className="font-mono text-sm font-medium text-rust mb-3">{r.agent}</div>
                <div className="space-y-2 font-mono text-xs text-muted-foreground">
                  <div><span className="text-foreground/50">user:</span> {r.user}</div>
                  <div><span className="text-foreground/50">workspace:</span> <span className="text-foreground/70">{r.ws}</span></div>
                  <div><span className="text-foreground/50">mcp:</span> <span className="text-browser">{r.mcp}</span></div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <p className="mt-6 font-mono text-xs text-muted-foreground">
          One cage-bro instance = one workspace, so run one instance per agent.
        </p>
      </div>
    </section>
  )
}
