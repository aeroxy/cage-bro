import { NavLink } from "react-router-dom"
import { cn } from "@/lib/utils"
import {
  Terminal,
  Code,
  FolderOpen,
  Globe,
  LayoutDashboard,
} from "lucide-react"

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/terminal", icon: Terminal, label: "Terminal" },
  { to: "/code", icon: Code, label: "Code" },
  { to: "/files", icon: FolderOpen, label: "Files" },
  { to: "/browser", icon: Globe, label: "Browser" },
]

function CageBroLogo({ className }: { className?: string }) {
  return (
    <img src={`${import.meta.env.BASE_URL}icons.svg`} alt="cage-bro" className={className} />
  )
}

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-screen bg-background">
      <aside className="w-56 border-r border-border flex flex-col">
        <div className="p-4 border-b border-border flex items-center gap-2">
          <CageBroLogo className="h-6 w-6" />
          <span className="font-semibold text-sm">cage-bro</span>
        </div>
        <nav className="flex-1 p-2 space-y-1">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors",
                  isActive
                    ? "bg-accent text-accent-foreground font-medium"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                )
              }
            >
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>
        <div className="p-4 border-t border-border">
          <div className="text-xs text-muted-foreground">v0.1.0</div>
        </div>
      </aside>
      <main className="flex-1 overflow-auto">
        {children}
      </main>
    </div>
  )
}
