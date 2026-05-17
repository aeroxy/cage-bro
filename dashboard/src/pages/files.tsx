import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { FolderOpen, File, ArrowLeft, Save, Search } from "lucide-react"
import { apiPost } from "@/lib/api"

interface FileEntry {
  path: string
  name: string
  is_dir: boolean
  size: number
}

export function FilesPage() {
  const [cwd, setCwd] = useState(".")
  const [entries, setEntries] = useState<FileEntry[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [content, setContent] = useState("")
  const [searchQuery, setSearchQuery] = useState("")
  const [searchResults, setSearchResults] = useState<any[]>([])

  const loadDir = async (path: string) => {
    const data = await apiPost("/v1/file/list", { path })
    setEntries(data.entries || [])
    setCwd(path)
    setSelected(null)
    setContent("")
  }

  const loadFile = async (path: string) => {
    const data = await apiPost("/v1/file/read", { path })
    if (data.error) {
      setContent(`Error: ${data.error}`)
    } else {
      setContent(data.content)
    }
    setSelected(path)
  }

  const saveFile = async () => {
    if (!selected) return
    await apiPost("/v1/file/write", { path: selected, content })
  }

  const search = async () => {
    if (!searchQuery) return
    const data = await apiPost("/v1/file/search", { query: searchQuery, path: cwd })
    setSearchResults(data.results || [])
  }

  useEffect(() => {
    loadDir(".")
  }, [])

  return (
    <div className="p-6 h-full flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">Files</h1>
        <div className="flex items-center gap-2">
          <Input
            placeholder="Search..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && search()}
            className="w-48"
          />
          <Button size="sm" variant="outline" onClick={search}>
            <Search className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4 flex-1 min-h-0">
        <Card className="flex flex-col">
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm flex items-center gap-2">
                <FolderOpen className="h-4 w-4" />
                {cwd}
              </CardTitle>
              {cwd !== "." && (
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => {
                    const parent = cwd.split("/").slice(0, -1).join("/") || "."
                    loadDir(parent)
                  }}
                >
                  <ArrowLeft className="h-4 w-4" />
                </Button>
              )}
            </div>
          </CardHeader>
          <CardContent className="flex-1 overflow-auto">
            <div className="space-y-1">
              {entries.map((e) => (
                <button
                  key={e.path}
                  onClick={() => (e.is_dir ? loadDir(e.path) : loadFile(e.path))}
                  className={`w-full text-left px-2 py-1 rounded text-sm flex items-center gap-2 hover:bg-accent ${
                    selected === e.path ? "bg-accent" : ""
                  }`}
                >
                  {e.is_dir ? (
                    <FolderOpen className="h-3 w-3 text-muted-foreground" />
                  ) : (
                    <File className="h-3 w-3 text-muted-foreground" />
                  )}
                  <span className="truncate">{e.name}</span>
                  {!e.is_dir && (
                    <span className="ml-auto text-xs text-muted-foreground">
                      {e.size}
                    </span>
                  )}
                </button>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card className="col-span-2 flex flex-col">
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm">
                {selected || "Select a file"}
              </CardTitle>
              {selected && (
                <Button size="sm" variant="outline" onClick={saveFile}>
                  <Save className="h-4 w-4 mr-1" />
                  Save
                </Button>
              )}
            </div>
          </CardHeader>
          <CardContent className="flex-1">
            {selected ? (
              <Textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                className="h-full font-mono text-sm resize-none"
              />
            ) : searchResults.length > 0 ? (
              <div className="space-y-2">
                {searchResults.map((r, i) => (
                  <button
                    key={i}
                    onClick={() => loadFile(r.path)}
                    className="w-full text-left p-2 rounded hover:bg-accent"
                  >
                    <div className="text-xs text-muted-foreground">
                      {r.path}:{r.line_number}
                    </div>
                    <div className="text-sm font-mono">{r.line_content}</div>
                  </button>
                ))}
              </div>
            ) : (
              <div className="h-full flex items-center justify-center text-muted-foreground text-sm">
                Select a file to edit
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
