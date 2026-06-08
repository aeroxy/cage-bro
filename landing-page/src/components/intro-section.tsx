export function IntroSection() {
  return (
    <section id="intro" className="border-b border-border">
      <div className="mx-auto max-w-6xl px-6 py-20">
        <div className="flex flex-col gap-3">
          <span className="font-mono text-xs uppercase tracking-wider text-rust">Intro</span>
          <h2 className="text-balance font-mono text-3xl font-medium tracking-tight text-foreground">
            Watch cage-bro in 10 minutes
          </h2>
          <p className="max-w-2xl text-pretty leading-relaxed text-muted-foreground">
            A quick tour of spinning up a jailed workspace, wiring it into an MCP client, and watching
            the sandbox reject anything that reaches outside its folder.
          </p>
        </div>

        <div className="mt-10 overflow-hidden rounded-lg border border-border bg-[oklch(0.13_0_0)] shadow-[0_0_0_1px_rgba(255,255,255,0.02)]">
          {/* browser-chrome title bar, matching the terminal motif */}
          <div className="flex items-center gap-2 border-b border-border bg-surface px-4 py-2.5">
            <div className="flex items-center gap-1.5">
              <span className="size-3 rounded-full bg-[oklch(0.6_0.18_25)]" aria-hidden />
              <span className="size-3 rounded-full bg-[oklch(0.75_0.15_85)]" aria-hidden />
              <span className="size-3 rounded-full bg-[oklch(0.72_0.18_140)]" aria-hidden />
            </div>
            <span className="ml-2 font-mono text-xs text-muted-foreground">cage-bro — intro.mp4</span>
          </div>
          {/* 16:9 responsive embed */}
          <div className="relative aspect-video w-full">
            <iframe
              className="absolute inset-0 size-full"
              src="https://www.youtube-nocookie.com/embed/wKRBLfkQZw4?rel=0&modestbranding=1"
              title="cage-bro intro"
              loading="lazy"
              allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
              referrerPolicy="strict-origin-when-cross-origin"
              allowFullScreen
            />
          </div>
        </div>
      </div>
    </section>
  )
}
