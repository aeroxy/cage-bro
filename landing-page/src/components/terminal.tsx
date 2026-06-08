import { cn } from "@/lib/utils"
import type { ReactNode } from "react"

export function Terminal({
  title = "bash",
  className,
  children,
}: {
  title?: string
  className?: string
  children: ReactNode
}) {
  return (
    <div
      className={cn(
        "overflow-hidden rounded-lg border border-border bg-[oklch(0.13_0_0)] font-mono text-sm shadow-[0_0_0_1px_rgba(255,255,255,0.02)]",
        className,
      )}
    >
      <div className="flex items-center gap-2 border-b border-border bg-surface px-4 py-2.5">
        <div className="flex items-center gap-1.5">
          <span className="size-3 rounded-full bg-[oklch(0.6_0.18_25)]" aria-hidden />
          <span className="size-3 rounded-full bg-[oklch(0.75_0.15_85)]" aria-hidden />
          <span className="size-3 rounded-full bg-[oklch(0.72_0.18_140)]" aria-hidden />
        </div>
        <span className="ml-2 text-xs text-muted-foreground">{title}</span>
      </div>
      <div className="overflow-x-auto p-4 leading-relaxed">{children}</div>
    </div>
  )
}

export function Line({
  prompt,
  children,
}: {
  prompt?: boolean
  children: ReactNode
}) {
  return (
    <div className="whitespace-pre text-foreground/90">
      {prompt && <span className="select-none text-rust">$ </span>}
      {children}
    </div>
  )
}

export function Out({ children, color }: { children: ReactNode; color?: "green" | "blue" | "muted" }) {
  const c =
    color === "green"
      ? "text-terminal"
      : color === "blue"
        ? "text-browser"
        : "text-muted-foreground"
  return <div className={cn("whitespace-pre", c)}>{children}</div>
}

export function Comment({ children }: { children: ReactNode }) {
  return <div className="whitespace-pre text-muted-foreground/70"># {children}</div>
}
