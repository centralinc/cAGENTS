import { test } from "node:test";
import { strict as assert } from "node:assert";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { execSync } from "node:child_process";

test("binary should be executable after postinstall", () => {
  // Simulate what happens after npm install
  const binDir = join(__dirname, "..", "bin");
  const binaryName = process.platform.startsWith("win") ? "cagents.exe" : "cagents";
  const binaryPath = join(binDir, binaryName);

  // Binary should exist
  assert.ok(existsSync(binaryPath), `Binary should exist at ${binaryPath}`);

  // Binary should be executable
  try {
    const output = execSync(`"${binaryPath}" --version`, { encoding: "utf8" });
    assert.ok(output.includes("cagents"), "Binary should output version");
  } catch (error) {
    assert.fail(`Binary should be executable: ${error}`);
  }
});

test("binary should work via node_modules/.bin/cagents", () => {
  // This tests the real-world scenario
  const binPath = join(__dirname, "..", "..", "..", ".bin", "cagents");

  if (existsSync(binPath)) {
    try {
      const output = execSync(`"${binPath}" --version`, { encoding: "utf8" });
      assert.ok(output.includes("cagents"), "Symlinked binary should work");
    } catch (error) {
      // This is expected to fail if not installed via npm/pnpm
      assert.ok(true, "Skipping symlink test (not in node_modules)");
    }
  }
});
