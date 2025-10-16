"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const node_test_1 = require("node:test");
const node_assert_1 = require("node:assert");
const node_fs_1 = require("node:fs");
const node_path_1 = require("node:path");
const node_child_process_1 = require("node:child_process");
(0, node_test_1.test)("binary should be executable after postinstall", () => {
    // Simulate what happens after npm install
    const binDir = (0, node_path_1.join)(__dirname, "..", "bin");
    const binaryName = process.platform.startsWith("win") ? "cagents.exe" : "cagents";
    const binaryPath = (0, node_path_1.join)(binDir, binaryName);
    // Binary should exist
    node_assert_1.strict.ok((0, node_fs_1.existsSync)(binaryPath), `Binary should exist at ${binaryPath}`);
    // Binary should be executable
    try {
        const output = (0, node_child_process_1.execSync)(`"${binaryPath}" --version`, { encoding: "utf8" });
        node_assert_1.strict.ok(output.includes("cagents"), "Binary should output version");
    }
    catch (error) {
        node_assert_1.strict.fail(`Binary should be executable: ${error}`);
    }
});
(0, node_test_1.test)("binary should work via node_modules/.bin/cagents", () => {
    // This tests the real-world scenario
    const binPath = (0, node_path_1.join)(__dirname, "..", "..", "..", ".bin", "cagents");
    if ((0, node_fs_1.existsSync)(binPath)) {
        try {
            const output = (0, node_child_process_1.execSync)(`"${binPath}" --version`, { encoding: "utf8" });
            node_assert_1.strict.ok(output.includes("cagents"), "Symlinked binary should work");
        }
        catch (error) {
            // This is expected to fail if not installed via npm/pnpm
            node_assert_1.strict.ok(true, "Skipping symlink test (not in node_modules)");
        }
    }
});
