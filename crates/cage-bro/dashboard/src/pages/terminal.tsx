import { useEffect, useRef, useState } from "react"
import { Terminal } from "@xterm/xterm"
import { FitAddon } from "@xterm/addon-fit"
import { WebLinksAddon } from "@xterm/addon-web-links"
import "@xterm/xterm/css/xterm.css"
import { Card } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Play, Square } from "lucide-react"
import { apiPost, getWsUrl } from "@/lib/api"

export function TerminalPage() {
  const termRef = useRef<HTMLDivElement>(null)
  const terminal = useRef<Terminal | null>(null)
  const wsRef = useRef<WebSocket | null>(null)
  const [connected, setConnected] = useState(false)
  const [sessionId, setSessionId] = useState<string | null>(null)

  useEffect(() => {
    if (!termRef.current) return

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: "Menlo, Monaco, 'Courier New', monospace",
      theme: {
        background: "#0a0a0a",
        foreground: "#e4e4e7",
        cursor: "#e4e4e7",
        selectionBackground: "#27272a",
      },
    })

    const fitAddon = new FitAddon()
    term.loadAddon(fitAddon)
    term.loadAddon(new WebLinksAddon())
    term.open(termRef.current)
    fitAddon.fit()

    terminal.current = term

    const handleResize = () => fitAddon.fit()
    window.addEventListener("resize", handleResize)

    return () => {
      window.removeEventListener("resize", handleResize)
      term.dispose()
    }
  }, [])

  const connect = async () => {
    if (!terminal.current) return

    // Create session
    const data = await apiPost("/v1/shell/session", {});
    if (data.error) {
      terminal.current.writeln(`\x1b[31mError: ${data.error}\x1b[0m`)
      return
    }

    const sid = data.session_id
    setSessionId(sid)
    terminal.current.writeln(`\x1b[32mConnected to session ${sid.slice(0, 8)}...\x1b[0m\r\n`)

    // Connect WebSocket
    const ws = new WebSocket(getWsUrl(`/v1/shell/session/${sid}/ws`))

    ws.onopen = () => {
      setConnected(true)
    }

    ws.onmessage = (e) => {
      if (e.data instanceof Blob) {
        e.data.arrayBuffer().then((buf) => {
          terminal.current?.write(new Uint8Array(buf))
        })
      } else {
        terminal.current?.write(e.data)
      }
    }

    ws.onclose = () => {
      setConnected(false)
      terminal.current?.writeln("\r\n\x1b[31mDisconnected\x1b[0m")
    }

    wsRef.current = ws

    // Forward terminal input to WebSocket
    terminal.current.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(data)
      }
    })
  }

  const disconnect = async () => {
    wsRef.current?.close()
    if (sessionId) {
      await apiPost(`/v1/shell/session/${sessionId}/close`)
    }
    setConnected(false)
    setSessionId(null)
  }

  return (
    <div className="p-6 h-full flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">Terminal</h1>
        <div className="flex gap-2">
          {!connected ? (
            <Button size="sm" onClick={connect}>
              <Play className="h-4 w-4 mr-1" />
              Connect
            </Button>
          ) : (
            <Button size="sm" variant="destructive" onClick={disconnect}>
              <Square className="h-4 w-4 mr-1" />
              Disconnect
            </Button>
          )}
        </div>
      </div>
      <Card className="flex-1 p-2 bg-[#0a0a0a]">
        <div ref={termRef} className="h-full w-full" />
      </Card>
    </div>
  )
}
