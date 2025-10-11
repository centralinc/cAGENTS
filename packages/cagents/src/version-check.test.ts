import { describe, it, beforeEach, afterEach } from "node:test";
import * as assert from "node:assert";
import { existsSync, writeFileSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import { checkAndRebuildIfNeeded } from "./version-check";

describe("version-check", () => {
  const testDir = join(__dirname, "..", "__test-temp__");
  const mockBinaryPath = join(testDir, "cagents");

  beforeEach(() => {
    // Clean up test directory
    if (existsSync(testDir)) {
      rmSync(testDir, { recursive: true, force: true });
    }
    mkdirSync(testDir, { recursive: true });
  });

  afterEach(() => {
    if (existsSync(testDir)) {
      rmSync(testDir, { recursive: true, force: true });
    }
  });

  it("should detect version mismatch", () => {
    // Create a mock binary that returns wrong version
    const mockBinary = `#!/bin/bash\necho "cagents 0.0.1"`;
    writeFileSync(mockBinaryPath, mockBinary, { mode: 0o755 });

    const result = checkAndRebuildIfNeeded(mockBinaryPath, "0.0.16");

    assert.strictEqual(result.needsRebuild, true);
    assert.strictEqual(result.binaryVersion, "0.0.1");
    assert.strictEqual(result.expectedVersion, "0.0.16");
  });

  it("should not rebuild when versions match", () => {
    // Create a mock binary that returns correct version
    const mockBinary = `#!/bin/bash\necho "cagents 0.0.16"`;
    writeFileSync(mockBinaryPath, mockBinary, { mode: 0o755 });

    const result = checkAndRebuildIfNeeded(mockBinaryPath, "0.0.16");

    assert.strictEqual(result.needsRebuild, false);
    assert.strictEqual(result.binaryVersion, "0.0.16");
  });

  it("should handle binary that doesn't exist", () => {
    const nonExistentPath = join(testDir, "nonexistent");

    const result = checkAndRebuildIfNeeded(nonExistentPath, "0.0.16");

    assert.strictEqual(result.needsRebuild, true);
    assert.strictEqual(result.binaryVersion, undefined);
  });

  it("should handle binary that fails to run", () => {
    // Create a binary that exits with error
    const mockBinary = `#!/bin/bash\nexit 1`;
    writeFileSync(mockBinaryPath, mockBinary, { mode: 0o755 });

    const result = checkAndRebuildIfNeeded(mockBinaryPath, "0.0.16");

    assert.strictEqual(result.needsRebuild, true);
    assert.strictEqual(result.binaryVersion, undefined);
  });
});
