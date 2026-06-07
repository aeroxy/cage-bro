# cage-bro Guide — Clean, Isolated Workspaces for Agents

cage-bro gives an AI agent a workspace it can't escape. Two common ways to use it:

- **A fleet on one box** — run ~20 agents on a Mac mini, each jailed to its own folder.
- **A remote sandbox in a VM** — run your agent (e.g. Claude) on your Mac, but give it a clean, disposable workspace inside a VM.

---

## Philosophy

cage-bro is **not** a heavy VM sandbox. It's a lightweight *workspace jail* for an
agent. Each sandbox is a confined process — about 100 MB and ~1 ms to start — so a
small box can hold dozens, where one container-per-agent would need tens of GB.

The idea is simple: **turn off the agent framework's own file and shell tools, and
give it cage-bro's MCP server as the only way to touch files or run commands.**
cage-bro then keeps every one of those operations inside the agent's workspace.

```text
Mac mini
│
├─ alice/   zeroclaw (its own file/shell tools OFF)
│             └─ MCP → cage-bro :8081 → jail: /srv/alice/workspace
│
├─ bob/     zeroclaw (its own file/shell tools OFF)
│             └─ MCP → cage-bro :8082 → jail: /srv/bob/workspace
│
└─ … up to ~20 agents, each on its own port and folder
```

---

## What "jailed" means

When an agent uses a cage-bro instance whose workspace is `W`:

| Tool | Confinement |
|---|---|
| `file_read` / `file_write` / `file_edit` / `file_list` / `file_search` | **Only inside `W`.** Any path that resolves outside `W` (including via `..`) is rejected. |
| `shell_exec` / `python_exec` / `node_exec` | Run **with `W` as the working directory**. On Linux, [Landlock](https://docs.kernel.org/userspace-api/landlock.html) limits them to read+write `W` and `/tmp`, and read+execute a system allowlist (`/usr`, `/bin`, `/lib`, `/etc`, `/proc`, …). `rlimit`s cap memory/CPU/output and a timeout kills runaways. |

So an agent **can't read or modify another agent's workspace**. It can read system
files (needed to run interpreters) and write only its own workspace and `/tmp`.

> **The boundary, honestly:** this is *process-level* isolation, not a hardware
> one. `/tmp` is shared and the network is open unless your deployment restricts
> them, and Landlock needs a real Linux kernel (macOS gets resource limits only).
> For hostile code or hard multi-tenancy, run each agent as its **own OS user**
> and/or **inside a VM** (see the next section). The
> [Isolation model](README.md#isolation-model) in the README has the full threat
> model.

One cage-bro instance = one workspace, so **run one instance per agent** (cage-bro
allows one instance per working directory).

---

## Scenario A: a fleet of agents on one box

### Install

```bash
cargo install cage-bro            # or: brew install aeroxy/tap/cage-bro
cage-bro setup                    # (optional) fetch the Obscura browser
# zeroclaw: https://github.com/zeroclaw-labs/zeroclaw
```

### Jail one agent

We'll jail a [zeroclaw](https://github.com/zeroclaw-labs/zeroclaw) agent to
`/srv/alice`.

**1. Create the folder and start cage-bro there.** cage-bro jails to
`<cwd>/workspace`, so launch it from the agent's folder:

```bash
sudo mkdir -p /srv/alice/workspace
cd /srv/alice
cage-bro mcp --http --port 8081       # MCP at http://127.0.0.1:8081/mcp
```

**2. Point zeroclaw at it and disable zeroclaw's own file/shell tools** in the
agent's `config.toml`:

```toml
[mcp]
enabled = true
deferred_loading = true            # fetch tool schemas on demand (saves context)

[[mcp.servers]]
name = "cagebro"                   # tools appear as cagebro__shell_exec, cagebro__file_read, …
transport = "http"
url = "http://127.0.0.1:8081/mcp"

# Deny zeroclaw's built-in file/shell tools so its only route to the disk is
# cage-bro (jailed to /srv/alice/workspace). Apply to the channel(s) the agent uses.
[channels.main]
tools_deny = ["file_read", "file_write", "file_edit", "file_list", "file_search", "shell"]
```

zeroclaw prefixes external MCP tools as `<server>__<tool>`, so `cagebro__shell_exec`
and friends don't clash with — and survive the denial of — zeroclaw's own `shell`
and `file_*` tools. Once those are denied, the model has no other tool that touches
the disk. (You can layer on `[autonomy] level = "read_only"` and `workspace_only`
as extra guards.)

**3. Check the jail:**

```bash
# write inside the workspace — OK
curl -s -X POST http://127.0.0.1:8081/mcp -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"file_write","arguments":{"path":"hello.txt","content":"hi"}}}'

# read another agent's folder — rejected
curl -s -X POST http://127.0.0.1:8081/mcp -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"file_read","arguments":{"path":"/srv/bob/workspace/secret.csv"}}}'
# -> Error: path outside sandbox
```

### Scale to many agents

Give each agent its own folder, port, and ideally its own **OS user** — the OS user
is what holds the line even if an agent process is compromised.

| Agent | OS user | Workspace | cage-bro MCP |
|---|---|---|---|
| alice | `agent-alice` | `/srv/alice/workspace` | `127.0.0.1:8081` |
| bob   | `agent-bob`   | `/srv/bob/workspace`   | `127.0.0.1:8082` |
| …     | …             | …                      | …              |

```bash
#!/usr/bin/env bash
# spin up N jailed agents (same-user; prefix with `sudo -u agent-NN` to harden)
agents=(alice bob carol dave)
port=8081
for name in "${agents[@]}"; do
  dir="/srv/$name"
  mkdir -p "$dir/workspace"
  ( cd "$dir" && cage-bro mcp --http --port "$port" >"$dir/cagebro.log" 2>&1 & )
  echo "$name -> MCP 127.0.0.1:$port"
  port=$((port + 1))
done
```

Each agent's zeroclaw config points `cagebro` at its own port. That's N
mutually-isolated agents on one machine.

---

## Scenario B: a remote sandbox in a VM (clean workspaces)

Run your agent — Claude Code, Claude Desktop, or any MCP client — on your Mac, but
keep its file and code work in a **clean, throwaway workspace inside a Linux VM**.
You get two things: experiments never touch your Mac, and the **VM is the hardware
boundary** that process isolation alone isn't. (Bonus: in a real Linux VM you also
get Landlock, which macOS doesn't have, so the in-VM jail is stronger.)

**1. Spin up a Linux VM** — a cloud instance, or local via Lima / UTM / OrbStack.
No KVM needed; cage-bro is just a process. Install cage-bro inside it:

```bash
# inside the VM
cargo install cage-bro            # or brew
mkdir -p /srv/work && cd /srv/work
```

**2. Connect your Mac's agent to it over SSH.** The simplest reliable transport is
stdio-over-SSH: your MCP client launches `cage-bro mcp` on the VM and talks to it
through the SSH pipe. In your client's MCP config (e.g. Claude Desktop's
`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "sandbox": {
      "command": "ssh",
      "args": ["sandbox-vm", "cd /srv/work && cage-bro mcp"]
    }
  }
}
```

Here `sandbox-vm` is an SSH host alias. The workspace is `/srv/work/workspace`
*inside the VM*; the agent's `file_*` and `shell_exec` calls run there, jailed by
cage-bro and contained by the VM. Your Mac's filesystem is never exposed.

To reset the workspace, just wipe `/srv/work/workspace` (or the VM). To run
several clean workspaces, point separate MCP entries at separate folders/VMs.

> Prefer this over running cage-bro natively on the Mac whenever the agent might
> run untrusted code: the VM gives the kernel/hardware boundary, cage-bro keeps the
> agent tidy inside it.

---

## Optional: handing files between agents with drift

If a machine has [drift](https://github.com/aeroxy/drift) installed, agents can
hand files to each other over an encrypted connection addressed by IP + port —
useful precisely because isolated agents (separate workspaces, often separate OS
users) share no directory. An agent that has the drift skill can push or pull a
file through `shell_exec`:

```bash
drift send --target 127.0.0.1:9082 report.csv   # push into a peer's workspace
drift pull --target 127.0.0.1:9081 report.csv   # or pull from a peer
```

The file lands in the other agent's workspace, where it can read it with
`file_read` — without either agent ever getting direct access to the other's
folder. drift is optional; it's just the network path for explicit, auditable file
exchange when you want one.

---

## Security model — guaranteed vs. your responsibility

| Concern | cage-bro | Your deploy layer |
|---|---|---|
| Read/write another agent's workspace | ✅ blocked | — |
| Write outside the workspace | ✅ blocked (`W` + `/tmp` only) | — |
| Runaway CPU / memory / output | ✅ rlimits + timeout + output cap | cgroup `pids.max` for fork bombs |
| Shared `/tmp`, readable `/proc` | ⚠️ allowed | separate OS user per agent |
| Network exfiltration | ❌ not filtered | host firewall / egress allowlist |
| Kernel-exploit escape | ❌ shared kernel | run inside a VM (Scenario B) |

For regulated or financial workloads, the clean setup is **one OS user + one
cage-bro per agent, the whole box inside a VM**, with egress locked down. cage-bro
provides density and per-agent file confinement *inside* that boundary.

---

## Reference

### Commands

```bash
cage-bro serve --port 8080        # REST API + dashboard + E2B-compatible endpoints
cage-bro mcp                      # MCP over stdio (workspace = <cwd>/workspace)
cage-bro mcp --http --port 8081   # MCP over HTTP at POST /mcp
cage-bro setup                    # install the Obscura browser
```

### MCP tools

| Tool | Purpose |
|---|---|
| `shell_exec` | Run a shell command (jailed to the workspace) |
| `python_exec` / `node_exec` | Run a Python / Node snippet (jailed) |
| `file_read` / `file_write` / `file_edit` | Workspace file I/O |
| `file_list` / `file_search` | List / grep within the workspace |
| `browser_navigate` / `browser_click` / `browser_type` / `browser_screenshot` / `browser_snapshot` / `browser_evaluate` | Headless browser control |
| `sandbox_info` | Sandbox metadata |
