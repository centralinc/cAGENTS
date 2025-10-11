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
export declare function checkAndRebuildIfNeeded(binaryPath: string, expectedVersion: string): VersionCheckResult;
/**
 * Trigger a cargo rebuild
 * @param workspaceRoot Path to the workspace root
 * @param release Whether to build in release mode
 * @returns Whether the rebuild succeeded
 */
export declare function rebuildBinary(workspaceRoot: string, release?: boolean): {
    success: boolean;
    error?: string;
};
