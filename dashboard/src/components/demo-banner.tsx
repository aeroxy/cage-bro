import { useEffect, useState } from "react"
import { isMockMode } from "@/lib/api"
import { AlertTriangle, ExternalLink } from "lucide-react"

export function DemoBanner() {
  const [mock, setMock] = useState(false)

  useEffect(() => {
    isMockMode().then(setMock)
  }, [])

  if (!mock) return null

  return (
    <div className="bg-yellow-500/10 border-b border-yellow-500/30 px-4 py-2 flex items-center gap-2 text-sm">
      <AlertTriangle className="h-4 w-4 text-yellow-500 shrink-0" />
      <span>
        <strong>Demo mode</strong> — This is a preview with mock data.{" "}
        <a
          href="https://github.com/aeroxy/cage-bro#installation"
          target="_blank"
          rel="noopener noreferrer"
          className="underline inline-flex items-center gap-1"
        >
          Install cage-bro
          <ExternalLink className="h-3 w-3" />
        </a>{" "}
        to use the real sandbox.
      </span>
    </div>
  )
}
