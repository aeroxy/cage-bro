import { useEffect, useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Terminal, Code, Globe, FolderOpen, Cpu } from "lucide-react"
import { apiFetch } from "@/lib/api"

interface SandboxInfo {
  name: string
  version: string
  runtime: string
  status: string
}

export function HomePage() {
  const [info, setInfo] = useState<SandboxInfo | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    apiFetch("/v1/sandbox/info")
      .then((r) => r.json())
      .then(setInfo)
      .catch((e) => setError(e.message))
  }, [])

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">cage-bro</h1>
          <p className="text-muted-foreground text-sm">
            Sandboxed execution environment for AI agents
          </p>
        </div>
        <Badge variant={info?.status === "running" ? "default" : "destructive"}>
          {info?.status || "disconnected"}
        </Badge>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <ServiceCard
          icon={<Terminal className="h-5 w-5" />}
          title="Shell"
          description="PTY terminal sessions"
          status="ready"
          href="/terminal"
        />
        <ServiceCard
          icon={<Code className="h-5 w-5" />}
          title="Code"
          description="Python, Node, Jupyter"
          status="ready"
          href="/code"
        />
        <ServiceCard
          icon={<FolderOpen className="h-5 w-5" />}
          title="Files"
          description="Read, write, search"
          status="ready"
          href="/files"
        />
        <ServiceCard
          icon={<Globe className="h-5 w-5" />}
          title="Browser"
          description="Obscura headless browser"
          status="ready"
          href="/browser"
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm flex items-center gap-2">
            <Cpu className="h-4 w-4" />
            Sandbox Info
          </CardTitle>
        </CardHeader>
        <CardContent>
          {error ? (
            <p className="text-sm text-destructive">{error}</p>
          ) : info ? (
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div className="text-muted-foreground">Name</div>
              <div>{info.name}</div>
              <div className="text-muted-foreground">Version</div>
              <div>{info.version}</div>
              <div className="text-muted-foreground">Runtime</div>
              <div>{info.runtime}</div>
              <div className="text-muted-foreground">Status</div>
              <div>{info.status}</div>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">Loading...</p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function ServiceCard({
  icon,
  title,
  description,
  status,
  href,
}: {
  icon: React.ReactNode
  title: string
  description: string
  status: string
  href: string
}) {
  return (
    <a href={`#${href}`}>
      <Card className="hover:bg-accent/50 transition-colors cursor-pointer">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              {icon}
              <CardTitle className="text-sm">{title}</CardTitle>
            </div>
            <Badge variant="outline" className="text-xs">
              {status}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <p className="text-xs text-muted-foreground">{description}</p>
        </CardContent>
      </Card>
    </a>
  )
}
