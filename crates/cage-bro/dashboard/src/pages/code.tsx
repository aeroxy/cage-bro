import { useState } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Play, Loader2 } from "lucide-react"
import { apiPost } from "@/lib/api"

const defaultCode: Record<string, string> = {
  python: 'print("Hello from cage-bro!")',
  node: 'console.log("Hello from cage-bro!")',
}

export function CodePage() {
  const [code, setCode] = useState(defaultCode.python)
  const [output, setOutput] = useState("")
  const [running, setRunning] = useState(false)
  const [lang, setLang] = useState<"python" | "node">("python")
  const [duration, setDuration] = useState<number | null>(null)

  const switchLang = (v: string) => {
    const newLang = v as "python" | "node"
    setLang(newLang)
    // Only reset code if it was the default for the old language
    if (code === defaultCode[lang]) {
      setCode(defaultCode[newLang])
    }
  }

  const run = async () => {
    setRunning(true)
    setOutput("")
    setDuration(null)

    try {
      const data = await apiPost(`/v1/code/${lang}`, { code })

      if (data.error) {
        setOutput(`Error: ${data.error}`)
      } else {
        let out = ""
        if (data.stdout) out += data.stdout
        if (data.stderr) out += `\n${data.stderr}`
        setOutput(out.trim() || "(no output)")
        setDuration(data.duration_ms)
      }
    } catch (e: any) {
      setOutput(`Network error: ${e.message}`)
    } finally {
      setRunning(false)
    }
  }

  return (
    <div className="p-6 h-full flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">Code Execution</h1>
        <div className="flex items-center gap-2">
          {duration !== null && (
            <Badge variant="outline">{duration}ms</Badge>
          )}
          <Button size="sm" onClick={run} disabled={running}>
            {running ? (
              <Loader2 className="h-4 w-4 mr-1 animate-spin" />
            ) : (
              <Play className="h-4 w-4 mr-1" />
            )}
            Run
          </Button>
        </div>
      </div>

      <Tabs value={lang} onValueChange={switchLang}>
        <TabsList>
          <TabsTrigger value="python">Python</TabsTrigger>
          <TabsTrigger value="node">Node.js</TabsTrigger>
        </TabsList>
      </Tabs>

      <div className="grid grid-cols-2 gap-4 flex-1 min-h-0">
        <Card className="flex flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Editor</CardTitle>
          </CardHeader>
          <CardContent className="flex-1">
            <Textarea
              value={code}
              onChange={(e) => setCode(e.target.value)}
              className="h-full font-mono text-sm resize-none"
              placeholder="Enter code..."
            />
          </CardContent>
        </Card>

        <Card className="flex flex-col">
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Output</CardTitle>
          </CardHeader>
          <CardContent className="flex-1">
            <pre className="text-sm font-mono whitespace-pre-wrap h-full overflow-auto p-2 bg-muted rounded-md">
              {output || "Run code to see output..."}
            </pre>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
