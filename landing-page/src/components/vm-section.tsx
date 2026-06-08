import { Terminal, Line, Out, Comment } from "@/components/terminal"

export function VmSection() {
  return (
    <section id="vm" className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-20">
        <div className="flex items-center gap-3">
          <span className="flex size-7 items-center justify-center rounded border border-border bg-surface font-mono text-xs text-rust">
            B
          </span>
          <h2 className="font-mono text-3xl font-medium tracking-tight text-foreground">
            A remote sandbox in a VM
          </h2>
        </div>
        <p className="mt-4 max-w-2xl text-pretty leading-relaxed text-muted-foreground">
          Run your agent — Claude Code, Claude Desktop, or any MCP client — on your Mac, but keep its file and
          code work in a clean, throwaway workspace inside a Linux VM. Experiments never touch your Mac, and the{" "}
          <span className="text-foreground">VM is the hardware boundary</span> that process isolation alone
          isn&apos;t.
        </p>

        <div className="mt-10 grid gap-6 lg:grid-cols-2 lg:items-start">
          <Terminal title="inside the VM">
            <Comment>spin up a Linux VM — cloud, Lima, UTM, OrbStack</Comment>
            <Line prompt>cargo install cage-bro</Line>
            <Line prompt>mkdir -p /srv/work && cd /srv/work</Line>
            <div className="h-3" />
            <Comment>workspace is /srv/work/workspace inside the VM</Comment>
            <Line prompt>cage-bro mcp</Line>
            <Out color="green">{"  stdio MCP ready · jail → /srv/work/workspace"}</Out>
          </Terminal>

          <Terminal title="claude_desktop_config.json">
            <Out color="muted">{"{"}</Out>
            <Out color="muted">{'  "mcpServers": {'}</Out>
            <Out color="muted">{'    "sandbox": {'}</Out>
            <div className="whitespace-pre">
              <span className="text-muted-foreground">{'      "command": '}</span>
              <span className="text-terminal">{'"ssh"'}</span>
              <span className="text-muted-foreground">,</span>
            </div>
            <div className="whitespace-pre">
              <span className="text-muted-foreground">{'      "args": ['}</span>
              <span className="text-terminal">{'"sandbox-vm"'}</span>
              <span className="text-muted-foreground">,</span>
            </div>
            <div className="whitespace-pre">
              <span className="text-muted-foreground">{"        "}</span>
              <span className="text-terminal">{'"cd /srv/work && cage-bro mcp"'}</span>
            </div>
            <Out color="muted">{"      ]"}</Out>
            <Out color="muted">{"    }"}</Out>
            <Out color="muted">{"  }"}</Out>
            <Out color="muted">{"}"}</Out>
          </Terminal>
        </div>

        <p className="mt-6 max-w-2xl text-sm leading-relaxed text-muted-foreground">
          Stdio-over-SSH is the simplest reliable transport: your MCP client launches{" "}
          <code className="font-mono text-foreground">cage-bro mcp</code> on the VM and talks to it through the
          SSH pipe. Your Mac&apos;s filesystem is never exposed. To reset, just wipe{" "}
          <code className="font-mono text-foreground">/srv/work/workspace</code> (or the VM).
        </p>
      </div>
    </section>
  )
}
