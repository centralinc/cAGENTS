#!/usr/bin/env node

/**
 * Builds the Rust binary and copies it to packages/cagents/bin/
 * This ensures the npm package contains the latest binary
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const rootDir = path.join(__dirname, '..');
const targetBinary = path.join(rootDir, 'target', 'release', 'cagents');
const targetBinaryWin = path.join(rootDir, 'target', 'release', 'cagents.exe');
const destDir = path.join(rootDir, 'packages', 'cagents', 'bin');
const destBinary = path.join(destDir, 'cagents');
const destBinaryWin = path.join(destDir, 'cagents.exe');

console.log('[build-binary] Building Rust workspace in release mode...');

try {
  execSync('cargo build --release --workspace', {
    cwd: rootDir,
    stdio: 'inherit'
  });
} catch (error) {
  console.error('[build-binary] Failed to build Rust workspace');
  process.exit(1);
}

// Ensure destination directory exists
if (!fs.existsSync(destDir)) {
  fs.mkdirSync(destDir, { recursive: true });
}

// Copy the appropriate binary for the platform
const isWindows = process.platform === 'win32';
const sourceBinary = isWindows ? targetBinaryWin : targetBinary;
const destBinaryPath = isWindows ? destBinaryWin : destBinary;

if (!fs.existsSync(sourceBinary)) {
  console.error(`[build-binary] Binary not found at: ${sourceBinary}`);
  process.exit(1);
}

console.log(`[build-binary] Copying binary from ${sourceBinary} to ${destBinaryPath}`);
fs.copyFileSync(sourceBinary, destBinaryPath);

// Set executable permissions (Unix-like systems)
if (!isWindows) {
  fs.chmodSync(destBinaryPath, 0o755);
}

// Verify the binary works
console.log('[build-binary] Verifying binary...');
try {
  const version = execSync(`"${destBinaryPath}" --version`, { encoding: 'utf8' });
  console.log(`[build-binary] ✓ Binary ready: ${version.trim()}`);
} catch (error) {
  console.error('[build-binary] Failed to verify binary:', error.message);
  process.exit(1);
}

console.log('[build-binary] ✓ Build complete');
