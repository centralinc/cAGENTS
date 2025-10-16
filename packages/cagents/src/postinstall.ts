import { chmodSync, copyFileSync, existsSync, realpathSync } from "node:fs";
import { join } from "node:path";

// Detect if we're in the monorepo (local development/CI) or installed from npm
const packageDir = realpathSync(__dirname);
const workspaceRoot = join(packageDir, "..", "..", "..");
const isMonorepo = existsSync(join(workspaceRoot, "Cargo.toml"));

// Skip postinstall in monorepo - the binary is managed by the build process
if (isMonorepo) {
  console.log("[cagents] Skipping postinstall in monorepo environment");
  process.exit(0);
}

// Determine the platform-specific binary name
function getPlatformBinary(): string | null {
  const platform = process.platform;
  const arch = process.arch;

  // Map Node.js platform/arch to our binary naming scheme
  if (platform === "win32") {
    return arch === "x64" ? "cagents-win32-x64.exe" : null;
  } else if (platform === "darwin") {
    if (arch === "x64") return "cagents-darwin-x64";
    if (arch === "arm64") return "cagents-darwin-arm64";
  } else if (platform === "linux") {
    return arch === "x64" ? "cagents-linux-x64" : null;
  }

  return null;
}

const platformBinary = getPlatformBinary();

if (!platformBinary) {
  console.error(`[cagents] FATAL: Unsupported platform: ${process.platform}-${process.arch}`);
  console.error("[cagents] Supported platforms: linux-x64, darwin-x64, darwin-arm64, win32-x64");
  process.exit(1);
}

const binDir = join(__dirname, "..", "bin");
const sourceBinaryPath = join(binDir, platformBinary);
const targetBinaryName = process.platform.startsWith("win") ? "cagents.exe" : "cagents";
const targetBinaryPath = join(binDir, targetBinaryName);

// Check if platform-specific binary exists
if (!existsSync(sourceBinaryPath)) {
  console.error("[cagents] FATAL: Binary not found at:", sourceBinaryPath);
  console.error("[cagents] The package may be corrupted. Try reinstalling: npm install cagents");
  process.exit(1);
}

// Copy the platform-specific binary to the generic name (cagents or cagents.exe)
// This allows the bin/cagents.js shim to work correctly
try {
  copyFileSync(sourceBinaryPath, targetBinaryPath);
  if (!process.platform.startsWith("win")) {
    chmodSync(targetBinaryPath, 0o755); // rwxr-xr-x
  }
  console.log(`[cagents] Installed ${platformBinary} for ${process.platform}-${process.arch}`);
} catch (error) {
  console.error("[cagents] FATAL: Failed to setup binary:", error);
  process.exit(1);
}
