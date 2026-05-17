# cage-bro-cli

CLI installer for [cage-bro](https://github.com/aeroxy/cage-bro) — downloads the Rust binary on first run.

## Install

```bash
pip install cage-bro-cli
```

## Usage

```bash
cage-bro serve --port 8080
cage-bro mcp
cage-bro setup
```

On first run, the CLI downloads the pre-built binary for your platform from GitHub releases and caches it in `~/.cache/cage-bro/` (Linux) or `~/Library/Caches/cage-bro/` (macOS).

## Supported Platforms

| Platform | Status |
|---|---|
| macOS ARM64 | Available |
| Others | Build from source: `cargo install --git https://github.com/aeroxy/cage-bro` |

## License

Apache-2.0
