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
// Fix binary permissions after npm install
// npm doesn't preserve executable permissions when publishing
const binaryName = process.platform.startsWith("win") ? "cagents.exe" : "cagents";
const binaryPath = (0, node_path_1.join)(__dirname, "..", "bin", binaryName);
if (!(0, node_fs_1.existsSync)(binaryPath)) {
    console.error("[cagents] FATAL: Binary not found at:", binaryPath);
    console.error("[cagents] The package may be corrupted. Try reinstalling: npm install cagents");
    process.exit(1);
}
try {
    (0, node_fs_1.chmodSync)(binaryPath, 0o755); // rwxr-xr-x
    console.log("[cagents] Binary permissions set successfully");
}
catch (error) {
    console.error("[cagents] FATAL: Failed to set binary permissions:", error);
    console.error("[cagents] This package requires permission to make the binary executable");
    process.exit(1);
}
