class CageBro < Formula
  desc "Sandboxed execution environment for AI agents — single Rust binary with browser, shell, code, files, and MCP"
  homepage "https://github.com/aeroxy/cage-bro"
  version "0.2.0"
  license "Apache-2.0"

  on_macos do
    on_arm do
      url "https://github.com/aeroxy/cage-bro/releases/download/0.2.0/cage-bro-macos-arm64.tar.gz"
      sha256 "9efb2d42b3e89e7f5de90a52e35d52d1446091d293d2ebfdb38b1ea473eaecda"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/aeroxy/cage-bro/releases/download/0.2.0/cage-bro-linux-x86_64.tar.gz"
      sha256 "6f39a06170acd32ed0ed418a21a4361d76daf612abae7def896e5ef0bb9cee76"
    end
  end

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
