#!/usr/bin/env node

"use strict";

const path = require("path");

const PLATFORMS = {
  "darwin-arm64": "@alaya-mcp/cli-darwin-arm64",
  "darwin-x64": "@alaya-mcp/cli-darwin-x64",
  "linux-x64": "@alaya-mcp/cli-linux-x64",
  "win32-x64": "@alaya-mcp/cli-win32-x64",
};

function getBinaryPath() {
  const platformKey = `${process.platform}-${process.arch}`;
  const pkg = PLATFORMS[platformKey];
  if (!pkg) {
    console.error(
      `alaya-mcp: unsupported platform ${platformKey}\n` +
        `Supported: ${Object.keys(PLATFORMS).join(", ")}\n` +
        `Build from source: cargo build --release --features "mcp llm"`
    );
    process.exit(1);
  }

  const binary = process.platform === "win32" ? "alaya-mcp.exe" : "alaya-mcp";

  // Try optionalDependencies first
  try {
    const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
    return path.join(pkgDir, binary);
  } catch {
    // pass
  }

  // Try postinstall fallback location
  const fallback = path.join(__dirname, "..", "bin", binary);
  try {
    require("fs").accessSync(fallback, require("fs").constants.X_OK);
    return fallback;
  } catch {
    // pass
  }

  console.error(
    `alaya-mcp: could not find binary for ${platformKey}\n` +
      `Package ${pkg} is not installed.\n` +
      `Try: npm install alaya-mcp --include=optional\n` +
      `Or build from source: cargo build --release --features "mcp llm"`
  );
  process.exit(1);
}

const binary = getBinaryPath();
const result = require("child_process").spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

if (result.error) {
  console.error(`alaya-mcp: failed to start: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
