# Branding & Assets

The cage-bro visual identity and where the brand assets live. **The most important thing on this page: the logo exists as two separate SVG files that must be kept in sync.**

---

## The logo

A "caged prompt" mark: rust-orange square brackets `[ ]` (echoing the `[` in the wordmark) enclosing a terminal-green block cursor, on a deep-black rounded square with a hairline rust edge. Flat shapes, no gradients — per the infrastructure-terminal direction.

| Token | Value | Role |
|---|---|---|
| Background | `#0D0D0D` | rounded-square tile |
| Rust Orange | `#CE422B` | brackets + hairline edge (the brand signal) |
| Terminal Green | `#4AF626` | block cursor |

Source viewBox is `0 0 256 256`. It stays legible down to ~24px (the dashboard sidebar size).

---

## ⚠️ Two source files — keep them in sync

The landing page and the dashboard are **separate Vite builds with separate `public/` roots**, so neither can import the other's asset at build time. The logo is therefore duplicated. The files are byte-identical today; **if you change the design, edit both.**

| Source file | Filename | Used by |
|---|---|---|
| [`landing-page/public/icon.svg`](../landing-page/public/icon.svg) | `icon.svg` | Landing-page favicon + apple-touch-icon ([`landing-page/index.html`](../landing-page/index.html)), and the README logo ([`README.md`](../README.md) references this path directly — not a copy) |
| [`crates/cage-bro/dashboard/public/icons.svg`](../crates/cage-bro/dashboard/public/icons.svg) | `icons.svg` (note the plural) | Dashboard favicon ([`index.html`](../crates/cage-bro/dashboard/index.html)) + sidebar logo ([`layout.tsx`](../crates/cage-bro/dashboard/src/components/layout.tsx)) |

So: **2 source files, 3 usage sites.** The README is a reference, not a third copy — only the two `.svg` files are sources of truth.

> Filenames differ on purpose (`icon.svg` vs `icons.svg`) — they predate this consolidation and are wired into each app's references, so renaming would touch HTML + TSX in both apps. Not worth it; just remember both when editing.

---

## Deployed paths

Both ship to GitHub Pages via [`.github/workflows/deploy.yml`](../.github/workflows/deploy.yml):

- Landing: `https://aeroxy.github.io/cage-bro/icon.svg`
- Dashboard: `https://aeroxy.github.io/cage-bro/dashboard/icons.svg`
