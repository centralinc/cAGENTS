"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const node_test_1 = require("node:test");
const assert = require("node:assert");
const node_fs_1 = require("node:fs");
const node_path_1 = require("node:path");
const version_check_1 = require("./version-check");
(0, node_test_1.describe)("version-check", () => {
    const testDir = (0, node_path_1.join)(__dirname, "..", "__test-temp__");
    const mockBinaryPath = (0, node_path_1.join)(testDir, "cagents");
    (0, node_test_1.beforeEach)(() => {
        // Clean up test directory
        if ((0, node_fs_1.existsSync)(testDir)) {
            (0, node_fs_1.rmSync)(testDir, { recursive: true, force: true });
        }
        (0, node_fs_1.mkdirSync)(testDir, { recursive: true });
    });
    (0, node_test_1.afterEach)(() => {
        if ((0, node_fs_1.existsSync)(testDir)) {
            (0, node_fs_1.rmSync)(testDir, { recursive: true, force: true });
        }
    });
    (0, node_test_1.it)("should detect version mismatch", () => {
        // Create a mock binary that returns wrong version
        const mockBinary = `#!/bin/bash\necho "cagents 0.0.1"`;
        (0, node_fs_1.writeFileSync)(mockBinaryPath, mockBinary, { mode: 0o755 });
        const result = (0, version_check_1.checkAndRebuildIfNeeded)(mockBinaryPath, "0.0.16");
        assert.strictEqual(result.needsRebuild, true);
        assert.strictEqual(result.binaryVersion, "0.0.1");
        assert.strictEqual(result.expectedVersion, "0.0.16");
    });
    (0, node_test_1.it)("should not rebuild when versions match", () => {
        // Create a mock binary that returns correct version
        const mockBinary = `#!/bin/bash\necho "cagents 0.0.16"`;
        (0, node_fs_1.writeFileSync)(mockBinaryPath, mockBinary, { mode: 0o755 });
        const result = (0, version_check_1.checkAndRebuildIfNeeded)(mockBinaryPath, "0.0.16");
        assert.strictEqual(result.needsRebuild, false);
        assert.strictEqual(result.binaryVersion, "0.0.16");
    });
    (0, node_test_1.it)("should handle binary that doesn't exist", () => {
        const nonExistentPath = (0, node_path_1.join)(testDir, "nonexistent");
        const result = (0, version_check_1.checkAndRebuildIfNeeded)(nonExistentPath, "0.0.16");
        assert.strictEqual(result.needsRebuild, true);
        assert.strictEqual(result.binaryVersion, undefined);
    });
    (0, node_test_1.it)("should handle binary that fails to run", () => {
        // Create a binary that exits with error
        const mockBinary = `#!/bin/bash\nexit 1`;
        (0, node_fs_1.writeFileSync)(mockBinaryPath, mockBinary, { mode: 0o755 });
        const result = (0, version_check_1.checkAndRebuildIfNeeded)(mockBinaryPath, "0.0.16");
        assert.strictEqual(result.needsRebuild, true);
        assert.strictEqual(result.binaryVersion, undefined);
    });
});
