import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";

export interface VersionCheckResult {
  needsRebuild: boolean;
  binaryVersion?: string;
  expectedVersion: string;
  error?: string;
}

/**
 * Check if the binary version matches the expected version
 * @param binaryPath Path to the cagents binary
 * @param expectedVersion Expected version (from package.json)
 * @returns Version check result
 */
export function checkAndRebuildIfNeeded(
  binaryPath: string,
  expectedVersion: string
): VersionCheckResult {
  // If binary doesn't exist, needs rebuild
  if (!existsSync(binaryPath)) {
    return {
      needsRebuild: true,
      expectedVersion,
      error: "Binary not found",
    };
  }

  try {
    // Run binary with --version flag
    const result = spawnSync(binaryPath, ["--version"], {
      encoding: "utf8",
      timeout: 5000,
    });

    if (result.error) {
      return {
        needsRebuild: true,
        expectedVersion,
        error: result.error.message,
      };
    }

    if (result.status !== 0) {
      return {
        needsRebuild: true,
        expectedVersion,
        error: `Binary exited with code ${result.status}`,
      };
    }

    // Parse version from output (expected format: "cagents 0.0.16")
    const output = (result.stdout || result.stderr || "").trim();
    const versionMatch = output.match(/(\d+\.\d+\.\d+)/);

    if (!versionMatch) {
      return {
        needsRebuild: true,
        expectedVersion,
        error: "Could not parse version from binary output",
      };
    }

    const binaryVersion = versionMatch[1];

    // Compare versions
    const needsRebuild = binaryVersion !== expectedVersion;

    return {
      needsRebuild,
      binaryVersion,
      expectedVersion,
    };
  } catch (error) {
    return {
      needsRebuild: true,
      expectedVersion,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Trigger a cargo rebuild
 * @param workspaceRoot Path to the workspace root
 * @param release Whether to build in release mode
 * @returns Whether the rebuild succeeded
 */
export function rebuildBinary(
  workspaceRoot: string,
  release: boolean = false
): { success: boolean; error?: string } {
  try {
    const args = ["build", "--workspace"];
    if (release) {
      args.push("--release");
    }

    console.log(`[cagents] Rebuilding binary: cargo ${args.join(" ")}`);

    const result = spawnSync("cargo", args, {
      cwd: workspaceRoot,
      stdio: "inherit",
      timeout: 120000, // 2 minute timeout
    });

    if (result.error) {
      return {
        success: false,
        error: result.error.message,
      };
    }

    if (result.status !== 0) {
      return {
        success: false,
        error: `Cargo build failed with exit code ${result.status}`,
      };
    }

    return { success: true };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}
