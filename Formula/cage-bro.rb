class CageBro < Formula
  desc "Sandboxed execution environment for AI agents — single Rust binary with browser, shell, code, files, and MCP"
  homepage "https://github.com/aeroxy/cage-bro"
  url "https://github.com/aeroxy/cage-bro/releases/download/0.2.0/cage-bro-macos-arm64.zip"
  sha256 "8df7858fd0494fca89a1fa20e5b4ae1f3cc9f0aa952f9a45ffde41dcc4a996e7"
  license "Apache-2.0"

  def install
    bin.install "cage-bro"
  end

  def caveats
    <<~EOS
      To install the Obscura browser engine:
        cage-bro setup

      To start the sandbox server:
        cage-bro serve

      Dashboard will be available at http://localhost:8080

      For Claude Desktop MCP integration, add to config:
        {
          "mcpServers": {
            "cage-bro": {
              "command": "cage-bro",
              "args": ["mcp"]
            }
          }
        }
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cage-bro --version")
  end
end
