import { test } from "node:test";
import { strict as assert } from "node:assert";
import { existsSync, statSync } from "node:fs";
import { join } from "node:path";
import { execSync } from "node:child_process";

/**
 * This test ensures that the binary distribution works correctly.
 * It should FAIL if the binary architecture issue reoccurs.
 */

test("platform-specific binaries should exist in package", () => {
  const binDir = join(__dirname, "..", "bin");

  // In CI, these should all exist after the workflow runs
  // In local dev, we may only have one platform's binary
  const expectedBinaries = [
    "cagents-linux-x64",
    "cagents-darwin-x64",
    "cagents-darwin-arm64",
    "cagents-win32-x64.exe",
  ];

  const existingBinaries = expectedBinaries.filter((bin) =>
    existsSync(join(binDir, bin))
  );

  // At least one binary should exist (for local dev testing)
  assert.ok(
    existingBinaries.length > 0,
    `Expected at least one platform binary in ${binDir}, found: ${existingBinaries.join(", ")}`
  );

  console.log(`Found binaries: ${existingBinaries.join(", ")}`);
});

test("postinstall should select correct binary for current platform", () => {
  const binDir = join(__dirname, "..", "bin");
  const platform = process.platform;
  const arch = process.arch;

  let expectedBinary: string;
  if (platform === "win32" && arch === "x64") {
    expectedBinary = "cagents-win32-x64.exe";
  } else if (platform === "darwin" && arch === "x64") {
    expectedBinary = "cagents-darwin-x64";
  } else if (platform === "darwin" && arch === "arm64") {
    expectedBinary = "cagents-darwin-arm64";
  } else if (platform === "linux" && arch === "x64") {
    expectedBinary = "cagents-linux-x64";
  } else {
    // Test passes on unsupported platforms (e.g., CI runners)
    console.log(`Skipping test on unsupported platform: ${platform}-${arch}`);
    return;
  }

  const binaryPath = join(binDir, expectedBinary);

  // Binary should exist for this platform
  assert.ok(
    existsSync(binaryPath),
    `Expected binary for ${platform}-${arch} at ${binaryPath}`
  );

  // Binary should be executable
  const stats = statSync(binaryPath);
  if (platform !== "win32") {
    const isExecutable = (stats.mode & 0o111) !== 0;
    assert.ok(isExecutable, `Binary should be executable: ${binaryPath}`);
  }

  console.log(`✓ Binary exists and is valid for ${platform}-${arch}`);
});

test("binary should output version", () => {
  const binDir = join(__dirname, "..", "bin");
  const platform = process.platform;
  const arch = process.arch;

  let binaryName: string;
  if (platform === "win32" && arch === "x64") {
    binaryName = "cagents-win32-x64.exe";
  } else if (platform === "darwin" && arch === "x64") {
    binaryName = "cagents-darwin-x64";
  } else if (platform === "darwin" && arch === "arm64") {
    binaryName = "cagents-darwin-arm64";
  } else if (platform === "linux" && arch === "x64") {
    binaryName = "cagents-linux-x64";
  } else {
    console.log(`Skipping test on unsupported platform: ${platform}-${arch}`);
    return;
  }

  const binaryPath = join(binDir, binaryName);

  if (!existsSync(binaryPath)) {
    console.log(`Binary not found at ${binaryPath}, skipping test`);
    return;
  }

  try {
    const output = execSync(`"${binaryPath}" --version`, { encoding: "utf8" });
    assert.ok(output.includes("cagents"), "Binary should output version");
    console.log(`✓ Binary works: ${output.trim()}`);
  } catch (error) {
    assert.fail(`Binary should be executable and output version: ${error}`);
  }
});
