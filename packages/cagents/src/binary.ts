import { existsSync, realpathSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { checkAndRebuildIfNeeded, rebuildBinary } from "./version-check";

export function resolveBinary(): string {
  const binaryName = process.platform.startsWith("win") ? "cagents.exe" : "cagents";

  // Resolve symlinks to find the actual package location
  const packageDir = realpathSync(__dirname);
  const workspaceRoot = join(packageDir, "..", "..", "..");

  // Check if we're in a workspace (local development)
  const isLocalDev = existsSync(join(workspaceRoot, "Cargo.toml"));

  // Define binary paths
  const bundled = join(packageDir, "..", "bin", binaryName);
  const release = join(workspaceRoot, "target", "release", binaryName);
  const debug = join(workspaceRoot, "target", "debug", binaryName);

  // For local development: always rebuild to ensure latest code
  if (isLocalDev) {
    console.log("[cagents] Local development mode - rebuilding...");

    // Use debug build for faster iteration
    const rebuild = rebuildBinary(workspaceRoot, false);

    if (rebuild.success) {
      if (existsSync(debug)) {
        return debug;
      }
    } else {
      console.error("[cagents] Build failed:", rebuild.error);

      // Fall back to existing binary if build fails
      if (existsSync(debug)) {
        console.warn("[cagents] Using existing debug binary");
        return debug;
      }
      if (existsSync(release)) {
        console.warn("[cagents] Using existing release binary");
        return release;
      }
    }
  }

  // For npm package distribution: use bundled binary
  if (existsSync(bundled)) {
    return bundled;
  }

  console.error("[cagents] No binary found. Build the Rust workspace: `cargo build --workspace`");
  console.error("[cagents] Looked in:", packageDir);
  return bundled; // Return bundled path as fallback
}
