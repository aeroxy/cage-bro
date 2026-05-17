import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Globe, Play, Camera, Loader2 } from "lucide-react"
import { apiPost } from "@/lib/api"

export function BrowserPage() {
  const [url, setUrl] = useState("https://github.com/aeroxy/cage-bro")
  const [running, setRunning] = useState(false)
  const [loading, setLoading] = useState(false)
  const [pageContent, setPageContent] = useState<any>(null)
  const [screenshot, setScreenshot] = useState<string | null>(null)
  const [status, setStatus] = useState<string>("idle")

  const launch = async () => {
    setLoading(true)
    try {
      const data = await apiPost("/v1/browser/launch", { stealth: true })
      if (data.error) {
        setStatus(`Error: ${data.error}`)
      } else {
        setRunning(true)
        setStatus("running")
      }
    } finally {
      setLoading(false)
    }
  }

  const navigate = async () => {
    setLoading(true)
    try {
      const data = await apiPost("/v1/browser/navigate", { url })
      if (data.error) {
        setStatus(`Error: ${data.error}`)
      } else {
        setPageContent(data)
        setStatus("navigated")
      }
    } finally {
      setLoading(false)
    }
  }

  const takeScreenshot = async () => {
    setLoading(true)
    try {
      const data = await apiPost("/v1/browser/screenshot", {})
      if (data.data) {
        setScreenshot(`data:image/png;base64,${data.data}`)
      }
    } finally {
      setLoading(false)
    }
  }

  const close = async () => {
    await apiPost("/v1/browser/close")
    setRunning(false)
    setPageContent(null)
    setScreenshot(null)
    setStatus("closed")
  }

  return (
    <div className="p-6 h-full flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">Browser</h1>
        <div className="flex items-center gap-2">
          <Badge variant={running ? "default" : "outline"}>{status}</Badge>
          {!running ? (
            <Button size="sm" onClick={launch} disabled={loading}>
              {loading ? (
                <Loader2 className="h-4 w-4 mr-1 animate-spin" />
              ) : (
                <Globe className="h-4 w-4 mr-1" />
              )}
              Launch
            </Button>
          ) : (
            <Button size="sm" variant="destructive" onClick={close}>
              Close
            </Button>
          )}
        </div>
      </div>

      <div className="flex gap-2">
        <Input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && navigate()}
          placeholder="https://..."
          className="flex-1"
        />
        <Button size="sm" onClick={navigate} disabled={!running || loading}>
          <Play className="h-4 w-4 mr-1" />
          Go
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={takeScreenshot}
          disabled={!running || loading}
        >
          <Camera className="h-4 w-4 mr-1" />
          Screenshot
        </Button>
      </div>

      <div className="grid grid-cols-2 gap-4 flex-1 min-h-0">
        <Card className="flex flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Page Content</CardTitle>
          </CardHeader>
          <CardContent className="flex-1 overflow-auto">
            {pageContent ? (
              <div className="space-y-2">
                <div className="text-sm">
                  <span className="text-muted-foreground">URL:</span>{" "}
                  {pageContent.url}
                </div>
                <div className="text-sm">
                  <span className="text-muted-foreground">Title:</span>{" "}
                  {pageContent.title}
                </div>
                <pre className="text-xs font-mono whitespace-pre-wrap p-2 bg-muted rounded-md max-h-96 overflow-auto">
                  {pageContent.text}
                </pre>
              </div>
            ) : (
              <div className="h-full flex items-center justify-center text-muted-foreground text-sm">
                Navigate to a page
              </div>
            )}
          </CardContent>
        </Card>

        <Card className="flex flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Screenshot</CardTitle>
          </CardHeader>
          <CardContent className="flex-1 flex items-center justify-center">
            {screenshot ? (
              <img
                src={screenshot}
                alt="Screenshot"
                className="max-w-full max-h-full rounded-md border"
              />
            ) : (
              <div className="text-muted-foreground text-sm">
                Take a screenshot
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
