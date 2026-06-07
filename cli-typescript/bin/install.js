/**
 * cage-bro binary installer — downloads from GitHub releases on postinstall.
 */

const { execSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const https = require("https");

const VERSION = "0.2.0";
const GITHUB_REPO = "aeroxy/cage-bro";
const BINARY_NAME = "cage-bro";

function getCacheDir() {
  const dir = path.join(os.homedir(), ".cache", "cage-bro");
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

function getPlatform() {
  const platform = os.platform();
  const arch = os.arch();

  const osMap = { darwin: "macos", linux: "linux", win32: "windows" };
  const archMap = { arm64: "arm64", x64: "x86_64" };

  const osName = osMap[platform];
  const archName = archMap[arch];

  if (!osName || !archName) {
    throw new Error(
      `No pre-built binary for ${platform}/${arch}. ` +
      `Build from source: cargo install --git https://github.com/aeroxy/cage-bro`
    );
  }

  // Check if pre-built binary exists (only macos-arm64 for now)
  const available = new Set(["macos-arm64"]);
  if (!available.has(`${osName}-${archName}`)) {
    throw new Error(
      `No pre-built binary for ${osName}-${archName} yet (available: macos-arm64). ` +
      `Build from source: cargo install --git https://github.com/aeroxy/cage-bro`
    );
  }

  return { osName, archName };
}

function getBinaryPath() {
  const cacheDir = getCacheDir();
  const ext = os.platform() === "win32" ? ".exe" : "";
  return path.join(cacheDir, `${BINARY_NAME}${ext}`);
}

function downloadBinary() {
  const cacheDir = getCacheDir();
  const binaryPath = getBinaryPath();

  if (fs.existsSync(binaryPath)) {
    return binaryPath;
  }

  const { osName, archName } = getPlatform();
  const ext = osName === "windows" ? ".zip" : ".tar.gz";
  const url = `https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/${BINARY_NAME}-${osName}-${archName}${ext}`;

  console.log(`Downloading cage-bro ${VERSION} for ${osName}-${archName}...`);
  console.log(`  ${url}`);

  const archivePath = path.join(cacheDir, `archive${ext}`);

  // Download
  const data = execSync(`curl -fsSL "${url}"`, { maxBuffer: 100 * 1024 * 1024 });
  fs.writeFileSync(archivePath, data);

  // Extract
  if (ext === ".tar.gz") {
    execSync(`tar xzf "${archivePath}" -C "${cacheDir}"`);
  } else {
    execSync(`cd "${cacheDir}" && unzip -o "${archivePath}"`);
  }

  fs.unlinkSync(archivePath);

  // Make executable
  if (os.platform() !== "win32") {
    fs.chmodSync(binaryPath, 0o755);
  }

  console.log(`Installed cage-bro to ${binaryPath}`);
  return binaryPath;
}

// Run on postinstall
if (require.main === module) {
  try {
    downloadBinary();
  } catch (err) {
    console.error("Failed to install cage-bro binary:", err.message);
    console.error("You can manually install by running: cage-bro setup");
    // Don't fail install — user can still use the SDK
  }
}

module.exports = { getBinaryPath, downloadBinary };
