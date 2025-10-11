"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.resolveBinary = resolveBinary;
const node_fs_1 = require("node:fs");
const node_path_1 = require("node:path");
const version_check_1 = require("./version-check");
function resolveBinary() {
    const binaryName = process.platform.startsWith("win") ? "cagents.exe" : "cagents";
    // Resolve symlinks to find the actual package location
    const packageDir = (0, node_fs_1.realpathSync)(__dirname);
    const workspaceRoot = (0, node_path_1.join)(packageDir, "..", "..", "..");
    // Check if we're in a workspace (local development)
    const isLocalDev = (0, node_fs_1.existsSync)((0, node_path_1.join)(workspaceRoot, "Cargo.toml"));
    // Define binary paths
    const bundled = (0, node_path_1.join)(packageDir, "..", "bin", binaryName);
    const release = (0, node_path_1.join)(workspaceRoot, "target", "release", binaryName);
    const debug = (0, node_path_1.join)(workspaceRoot, "target", "debug", binaryName);
    // For local development: always rebuild to ensure latest code
    if (isLocalDev) {
        console.log("[cagents] Local development mode - rebuilding...");
        // Use debug build for faster iteration
        const rebuild = (0, version_check_1.rebuildBinary)(workspaceRoot, false);
        if (rebuild.success) {
            if ((0, node_fs_1.existsSync)(debug)) {
                return debug;
            }
        }
        else {
            console.error("[cagents] Build failed:", rebuild.error);
            // Fall back to existing binary if build fails
            if ((0, node_fs_1.existsSync)(debug)) {
                console.warn("[cagents] Using existing debug binary");
                return debug;
            }
            if ((0, node_fs_1.existsSync)(release)) {
                console.warn("[cagents] Using existing release binary");
                return release;
            }
        }
    }
    // For npm package distribution: use bundled binary
    if ((0, node_fs_1.existsSync)(bundled)) {
        return bundled;
    }
    console.error("[cagents] No binary found. Build the Rust workspace: `cargo build --workspace`");
    console.error("[cagents] Looked in:", packageDir);
    return bundled; // Return bundled path as fallback
}
