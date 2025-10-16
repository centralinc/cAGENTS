"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const node_fs_1 = require("node:fs");
const node_path_1 = require("node:path");
// Detect if we're in the monorepo (local development/CI) or installed from npm
const packageDir = (0, node_fs_1.realpathSync)(__dirname);
const workspaceRoot = (0, node_path_1.join)(packageDir, "..", "..", "..");
const isMonorepo = (0, node_fs_1.existsSync)((0, node_path_1.join)(workspaceRoot, "Cargo.toml"));
// Skip postinstall in monorepo - the binary is managed by the build process
if (isMonorepo) {
    console.log("[cagents] Skipping postinstall in monorepo environment");
    process.exit(0);
}
// Determine the platform-specific binary name
function getPlatformBinary() {
    const platform = process.platform;
    const arch = process.arch;
    // Map Node.js platform/arch to our binary naming scheme
    if (platform === "win32") {
        return arch === "x64" ? "cagents-win32-x64.exe" : null;
    }
    else if (platform === "darwin") {
        if (arch === "x64")
            return "cagents-darwin-x64";
        if (arch === "arm64")
            return "cagents-darwin-arm64";
    }
    else if (platform === "linux") {
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
const binDir = (0, node_path_1.join)(__dirname, "..", "bin");
const sourceBinaryPath = (0, node_path_1.join)(binDir, platformBinary);
const targetBinaryName = process.platform.startsWith("win") ? "cagents.exe" : "cagents";
const targetBinaryPath = (0, node_path_1.join)(binDir, targetBinaryName);
// Check if platform-specific binary exists
if (!(0, node_fs_1.existsSync)(sourceBinaryPath)) {
    console.error("[cagents] FATAL: Binary not found at:", sourceBinaryPath);
    console.error("[cagents] The package may be corrupted. Try reinstalling: npm install cagents");
    process.exit(1);
}
// Copy the platform-specific binary to the generic name (cagents or cagents.exe)
// This allows the bin/cagents.js shim to work correctly
try {
    (0, node_fs_1.copyFileSync)(sourceBinaryPath, targetBinaryPath);
    if (!process.platform.startsWith("win")) {
        (0, node_fs_1.chmodSync)(targetBinaryPath, 0o755); // rwxr-xr-x
    }
    console.log(`[cagents] Installed ${platformBinary} for ${process.platform}-${process.arch}`);
}
catch (error) {
    console.error("[cagents] FATAL: Failed to setup binary:", error);
    process.exit(1);
}
