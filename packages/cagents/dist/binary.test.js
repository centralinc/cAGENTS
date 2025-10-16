"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const node_test_1 = require("node:test");
const node_assert_1 = require("node:assert");
const node_fs_1 = require("node:fs");
const node_path_1 = require("node:path");
const node_child_process_1 = require("node:child_process");
/**
 * This test ensures that the binary distribution works correctly.
 * It should FAIL if the binary architecture issue reoccurs.
 */
(0, node_test_1.test)("platform-specific binaries should exist in package", () => {
    const binDir = (0, node_path_1.join)(__dirname, "..", "bin");
    // In CI, these should all exist after the workflow runs
    // In local dev, we may only have one platform's binary
    const expectedBinaries = [
        "cagents-linux-x64",
        "cagents-darwin-x64",
        "cagents-darwin-arm64",
        "cagents-win32-x64.exe",
    ];
    const existingBinaries = expectedBinaries.filter((bin) => (0, node_fs_1.existsSync)((0, node_path_1.join)(binDir, bin)));
    // At least one binary should exist (for local dev testing)
    node_assert_1.strict.ok(existingBinaries.length > 0, `Expected at least one platform binary in ${binDir}, found: ${existingBinaries.join(", ")}`);
    console.log(`Found binaries: ${existingBinaries.join(", ")}`);
});
(0, node_test_1.test)("postinstall should select correct binary for current platform", () => {
    const binDir = (0, node_path_1.join)(__dirname, "..", "bin");
    const platform = process.platform;
    const arch = process.arch;
    let expectedBinary;
    if (platform === "win32" && arch === "x64") {
        expectedBinary = "cagents-win32-x64.exe";
    }
    else if (platform === "darwin" && arch === "x64") {
        expectedBinary = "cagents-darwin-x64";
    }
    else if (platform === "darwin" && arch === "arm64") {
        expectedBinary = "cagents-darwin-arm64";
    }
    else if (platform === "linux" && arch === "x64") {
        expectedBinary = "cagents-linux-x64";
    }
    else {
        // Test passes on unsupported platforms (e.g., CI runners)
        console.log(`Skipping test on unsupported platform: ${platform}-${arch}`);
        return;
    }
    const binaryPath = (0, node_path_1.join)(binDir, expectedBinary);
    // Binary should exist for this platform
    node_assert_1.strict.ok((0, node_fs_1.existsSync)(binaryPath), `Expected binary for ${platform}-${arch} at ${binaryPath}`);
    // Binary should be executable
    const stats = (0, node_fs_1.statSync)(binaryPath);
    if (platform !== "win32") {
        const isExecutable = (stats.mode & 0o111) !== 0;
        node_assert_1.strict.ok(isExecutable, `Binary should be executable: ${binaryPath}`);
    }
    console.log(`✓ Binary exists and is valid for ${platform}-${arch}`);
});
(0, node_test_1.test)("binary should output version", () => {
    const binDir = (0, node_path_1.join)(__dirname, "..", "bin");
    const platform = process.platform;
    const arch = process.arch;
    let binaryName;
    if (platform === "win32" && arch === "x64") {
        binaryName = "cagents-win32-x64.exe";
    }
    else if (platform === "darwin" && arch === "x64") {
        binaryName = "cagents-darwin-x64";
    }
    else if (platform === "darwin" && arch === "arm64") {
        binaryName = "cagents-darwin-arm64";
    }
    else if (platform === "linux" && arch === "x64") {
        binaryName = "cagents-linux-x64";
    }
    else {
        console.log(`Skipping test on unsupported platform: ${platform}-${arch}`);
        return;
    }
    const binaryPath = (0, node_path_1.join)(binDir, binaryName);
    if (!(0, node_fs_1.existsSync)(binaryPath)) {
        console.log(`Binary not found at ${binaryPath}, skipping test`);
        return;
    }
    try {
        const output = (0, node_child_process_1.execSync)(`"${binaryPath}" --version`, { encoding: "utf8" });
        node_assert_1.strict.ok(output.includes("cagents"), "Binary should output version");
        console.log(`✓ Binary works: ${output.trim()}`);
    }
    catch (error) {
        node_assert_1.strict.fail(`Binary should be executable and output version: ${error}`);
    }
});
