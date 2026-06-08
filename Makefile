.PHONY: build release release-linux release-all check run test clean setup dashboard bump-patch bump-minor bump-major publish publish-npm setup-pypi publish-pypi

LINUX_TARGET = x86_64-unknown-linux-gnu
LINUX_OUT    = target/$(LINUX_TARGET)/release

## Build the project (debug)
build: dashboard
	cargo build

## Release build — use this before manual testing
release: dashboard
	cargo build --release

## Cross-build a Linux x86_64 release binary (requires: brew install zig && cargo install cargo-zigbuild)
release-linux: dashboard
	cargo zigbuild --release --target $(LINUX_TARGET)

## Build + archive release binaries for all shipped platforms (macOS arm64 + Linux x86_64).
## One .tar.gz per platform — consumed by Homebrew and the npm/pip CLIs alike.
release-all: release release-linux
	tar -C target/release -czf target/release/cage-bro-macos-arm64.tar.gz cage-bro
	tar -C $(LINUX_OUT) -czf $(LINUX_OUT)/cage-bro-linux-x86_64.tar.gz cage-bro
	@echo ""
	@echo "Release archives ready:"
	@echo "  target/release/cage-bro-macos-arm64.tar.gz"
	@echo "  $(LINUX_OUT)/cage-bro-linux-x86_64.tar.gz"

## Type-check without producing a binary
check:
	cargo check

## Run the server; pass flags via ARGS
##   make run
##   make run ARGS="--port 9090"
run:
	cargo run -- serve $(ARGS)

## Run MCP server (stdio)
mcp:
	cargo run -- mcp

## Run tests
test:
	cargo test

## Build dashboard frontend
dashboard:
	cd crates/cage-bro/dashboard && bun run build

## Install dependencies (obscura browser)
setup:
	cargo run -- setup

## Remove build artifacts
clean:
	cargo clean
	rm -rf crates/cage-bro/dashboard/dist crates/cage-bro/dashboard/node_modules

## Publish to crates.io
publish: dashboard
	cargo publish -p cage-bro-code --allow-dirty
	cargo publish -p cage-bro-runtime --allow-dirty
	cargo publish -p cage-bro --allow-dirty

## Publish @cage-bro/cli to npm
publish-npm:
	cd cli-typescript && npm publish --access public

## Setup Python venv for cli-python (run once)
setup-pypi:
	python3 -m venv cli-python/.venv
	cli-python/.venv/bin/pip install build twine httpx

## Build and publish cage-bro-cli to PyPI
publish-pypi:
	rm -rf cli-python/dist
	cd cli-python && .venv/bin/python -m build && .venv/bin/twine upload dist/*

## Update Formula/cage-bro.rb SHA256s from local release tarballs (run after release-all, before upload)
##   make update-formula
update-formula:
	@mac_tar="target/release/cage-bro-macos-arm64.tar.gz"; \
	lin_tar="$(LINUX_OUT)/cage-bro-linux-x86_64.tar.gz"; \
	echo "Computing SHA256s …"; \
	export MAC_SHA=$$(shasum -a 256 "$$mac_tar" | cut -d' ' -f1); \
	export LIN_SHA=$$(shasum -a 256 "$$lin_tar" | cut -d' ' -f1); \
	echo "macOS arm64    SHA256: $$MAC_SHA"; \
	echo "Linux x86_64   SHA256: $$LIN_SHA"; \
	perl -0pi -e 's/(macos-arm64\.tar\.gz"\s*\n\s*sha256 ")[0-9a-f]+/$$1$$ENV{MAC_SHA}/' Formula/cage-bro.rb; \
	perl -0pi -e 's/(linux-x86_64\.tar\.gz"\s*\n\s*sha256 ")[0-9a-f]+/$$1$$ENV{LIN_SHA}/' Formula/cage-bro.rb; \
	echo "Formula/cage-bro.rb updated"

## Bump the patch version (0.1.0 → 0.1.1) and update all version references
bump-patch:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	minor=$$(echo $$old | cut -d. -f2); \
	patch=$$(echo $$old | cut -d. -f3); \
	new="$$major.$$minor.$$((patch+1))"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' -E "s|version = \"[0-9.]+\"(, path = \"\.\./cage-bro-)|version = \"$$new\"\1|g" crates/cage-bro/Cargo.toml; \
	sed -i '' "s/version = \"$$old\"/version = \"$$new\"/" crates/cage-bro/dashboard/package.json; \
	sed -i '' "s/version \"$$old\"/version \"$$new\"/" Formula/cage-bro.rb; \
	sed -i '' "s|/$$old/|/$$new/|g" Formula/cage-bro.rb; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" sdk/python/pyproject.toml; \
	sed -i '' "s/__version__ = \"$$old\"/__version__ = \"$$new\"/" sdk/python/cage_bro/__init__.py; \
	sed -i '' "s/\"version\": \"$$old\"/\"version\": \"$$new\"/" sdk/typescript/package.json; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" cli-python/pyproject.toml; \
	sed -i '' "s/VERSION = \"$$old\"/VERSION = \"$$new\"/" cli-python/cage_bro_cli/__init__.py; \
	sed -i '' "s/\"version\": \"$$old\"/\"version\": \"$$new\"/" cli-typescript/package.json; \
	sed -i '' "s/const VERSION = \"$$old\"/const VERSION = \"$$new\"/" cli-typescript/bin/install.js; \
	sed -i '' "s/VERSION = \"$$old\"/VERSION = \"$$new\"/" cli-typescript/bin/install.js; \
	echo "$$old → $$new"

## Bump the minor version (0.1.1 → 0.2.0) and update all version references
bump-minor:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	minor=$$(echo $$old | cut -d. -f2); \
	new="$$major.$$((minor+1)).0"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' -E "s|version = \"[0-9.]+\"(, path = \"\.\./cage-bro-)|version = \"$$new\"\1|g" crates/cage-bro/Cargo.toml; \
	sed -i '' "s/version = \"$$old\"/version = \"$$new\"/" crates/cage-bro/dashboard/package.json; \
	sed -i '' "s/version \"$$old\"/version \"$$new\"/" Formula/cage-bro.rb; \
	sed -i '' "s|/$$old/|/$$new/|g" Formula/cage-bro.rb; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" sdk/python/pyproject.toml; \
	sed -i '' "s/__version__ = \"$$old\"/__version__ = \"$$new\"/" sdk/python/cage_bro/__init__.py; \
	sed -i '' "s/\"version\": \"$$old\"/\"version\": \"$$new\"/" sdk/typescript/package.json; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" cli-python/pyproject.toml; \
	sed -i '' "s/VERSION = \"$$old\"/VERSION = \"$$new\"/" cli-python/cage_bro_cli/__init__.py; \
	sed -i '' "s/\"version\": \"$$old\"/\"version\": \"$$new\"/" cli-typescript/package.json; \
	sed -i '' "s/const VERSION = \"$$old\"/const VERSION = \"$$new\"/" cli-typescript/bin/install.js; \
	sed -i '' "s/VERSION = \"$$old\"/VERSION = \"$$new\"/" cli-typescript/bin/install.js; \
	echo "$$old → $$new"

## Bump the major version (0.2.0 → 1.0.0) and update all version references
bump-major:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	new="$$((major+1)).0.0"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
	sed -i '' -E "s|version = \"[0-9.]+\"(, path = \"\.\./cage-bro-)|version = \"$$new\"\1|g" crates/cage-bro/Cargo.toml; \
	sed -i '' "s/version = \"$$old\"/version = \"$$new\"/" crates/cage-bro/dashboard/package.json; \
	sed -i '' "s/version \"$$old\"/version \"$$new\"/" Formula/cage-bro.rb; \
	sed -i '' "s|/$$old/|/$$new/|g" Formula/cage-bro.rb; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" sdk/python/pyproject.toml; \
	sed -i '' "s/__version__ = \"$$old\"/__version__ = \"$$new\"/" sdk/python/cage_bro/__init__.py; \
	sed -i '' "s/\"version\": \"$$old\"/\"version\": \"$$new\"/" sdk/typescript/package.json; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" cli-python/pyproject.toml; \
	sed -i '' "s/VERSION = \"$$old\"/VERSION = \"$$new\"/" cli-python/cage_bro_cli/__init__.py; \
	sed -i '' "s/\"version\": \"$$old\"/\"version\": \"$$new\"/" cli-typescript/package.json; \
	sed -i '' "s/const VERSION = \"$$old\"/const VERSION = \"$$new\"/" cli-typescript/bin/install.js; \
	sed -i '' "s/VERSION = \"$$old\"/VERSION = \"$$new\"/" cli-typescript/bin/install.js; \
	echo "$$old → $$new"
