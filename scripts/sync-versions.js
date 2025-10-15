#!/usr/bin/env node

/**
 * Syncs version from package.json to Cargo.toml files
 * Run this after changesets updates the npm package version
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const rootDir = path.join(__dirname, '..');
const packageJsonPath = path.join(rootDir, 'packages/cagents/package.json');
const coreCargoPath = path.join(rootDir, 'crates/cagents-core/Cargo.toml');
const cliCargoPath = path.join(rootDir, 'crates/cagents-cli/Cargo.toml');

// Read version from package.json
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const newVersion = packageJson.version;

console.log(`Syncing version ${newVersion} to Cargo.toml files...`);

// Update Cargo.toml files
function updateCargoToml(filePath) {
  let content = fs.readFileSync(filePath, 'utf8');

  // Match version = "x.y.z" in the [package] section
  content = content.replace(
    /(\[package\][^\[]*version\s*=\s*)"[^"]*"/,
    `$1"${newVersion}"`
  );

  fs.writeFileSync(filePath, content, 'utf8');
  console.log(`✓ Updated ${path.relative(rootDir, filePath)}`);
}

updateCargoToml(coreCargoPath);
updateCargoToml(cliCargoPath);

// Update Cargo.lock to reflect the new versions
console.log('Updating Cargo.lock...');
try {
  execSync('cargo update --workspace', {
    cwd: rootDir,
    stdio: 'inherit'
  });
  console.log('✓ Updated Cargo.lock');
} catch (error) {
  console.error('Failed to update Cargo.lock:', error.message);
  process.exit(1);
}

console.log('✓ Version sync complete');
