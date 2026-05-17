.PHONY: build release check run test clean setup dashboard bump-patch bump-minor bump-major publish

## Build the project (debug)
build: dashboard
	cargo build

## Release build — use this before manual testing
release: dashboard
	cargo build --release
	zip -j target/release/cage-bro-macos-arm64.zip target/release/cage-bro

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

## Update Formula/cage-bro.rb SHA256 from local release zip (run after release, before upload)
##   make update-formula
update-formula:
	@mac_zip="target/release/cage-bro-macos-arm64.zip"; \
	echo "Computing macOS SHA256 …"; \
	mac_sha=$$(shasum -a 256 "$$mac_zip" | cut -d' ' -f1); \
	echo "macOS SHA256: $$mac_sha"; \
	sed -i '' "s/sha256 \"[A-Za-z0-9_-]*\"/sha256 \"$$mac_sha\"/" Formula/cage-bro.rb; \
	echo "Formula/cage-bro.rb updated"

## Bump the patch version (0.1.0 → 0.1.1) and update all version references
bump-patch:
	@old=$$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'); \
	major=$$(echo $$old | cut -d. -f1); \
	minor=$$(echo $$old | cut -d. -f2); \
	patch=$$(echo $$old | cut -d. -f3); \
	new="$$major.$$minor.$$((patch+1))"; \
	sed -i '' "s/^version = \"$$old\"/version = \"$$new\"/" Cargo.toml; \
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
