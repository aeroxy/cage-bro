import { HashRouter, Routes, Route } from "react-router-dom"
import { Layout } from "@/components/layout"
import { DemoBanner } from "@/components/demo-banner"
import { HomePage } from "@/pages/home"
import { TerminalPage } from "@/pages/terminal"
import { CodePage } from "@/pages/code"
import { FilesPage } from "@/pages/files"
import { BrowserPage } from "@/pages/browser"
import { Toaster } from "@/components/ui/sonner"

export default function App() {
  return (
    <HashRouter>
      <DemoBanner />
      <Layout>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/terminal" element={<TerminalPage />} />
          <Route path="/code" element={<CodePage />} />
          <Route path="/files" element={<FilesPage />} />
          <Route path="/browser" element={<BrowserPage />} />
        </Routes>
      </Layout>
      <Toaster />
    </HashRouter>
  )
}
