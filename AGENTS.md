# AGENTS.md

Guidance for AI agents working in this repo. Start with the [wiki](wiki/overview.md).

## Documentation

- [Overview](wiki/overview.md) — what cage-bro is, architecture, crate structure
- [API Reference](wiki/apis.md) — endpoints, MCP tools, SDK methods
- [Common Patterns](wiki/common-patterns.md) — the main use cases
- [Gotchas](wiki/gotchas.md) — edge cases and things that will bite you
- [Related](wiki/related.md) — integrations and ecosystem
- [Branding & Assets](wiki/branding.md) — logo, palette, and asset locations

## Brand assets — read before touching the logo

The logo lives in **two byte-identical source files** (separate Vite builds, separate `public/` roots — neither can import the other):

- `landing-page/public/icon.svg` (landing favicon + README logo)
- `crates/cage-bro/dashboard/public/icons.svg` (dashboard favicon + sidebar logo)

**If you change the design, edit both.** Details and deployed paths: [wiki/branding.md](wiki/branding.md).
