#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');
const { spawn, execSync } = require('child_process');
const https = require('https');

const BINARY_NAME = process.platform === 'win32' ? 'cgrill.exe' : 'cgrill';
const GITHUB_REPO = 'rishiicreates/context-grill';
const VERSION = '0.1.0';

/**
 * Find candidate native binary locations
 */
function findNativeBinary() {
  const candidates = [];

  // 1. Check workspace target directory (development mode)
  const repoRoot = path.resolve(__dirname, '..');
  candidates.push(path.join(repoRoot, 'target', 'release', BINARY_NAME));
  candidates.push(path.join(repoRoot, 'target', 'debug', BINARY_NAME));

  // 2. Check local package bin directory
  candidates.push(path.join(repoRoot, '.bin', BINARY_NAME));

  // 3. Check user home cache directories
  const home = os.homedir();
  candidates.push(path.join(home, '.cgrill', 'bin', BINARY_NAME));
  candidates.push(path.join(home, '.cargo', 'bin', BINARY_NAME));

  // 4. Check system paths
  if (process.platform !== 'win32') {
    candidates.push(path.join('/usr/local/bin', BINARY_NAME));
    candidates.push(path.join('/opt/homebrew/bin', BINARY_NAME));
  }

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      try {
        const stat = fs.statSync(candidate);
        if (stat.isFile()) {
          return candidate;
        }
      } catch (_) {}
    }
  }

  // 5. Check PATH environment variable (excluding this JS script itself)
  const pathDirs = (process.env.PATH || '').split(path.delimiter);
  const selfPath = fs.realpathSync(path.resolve(__filename));

  for (const dir of pathDirs) {
    if (!dir) continue;
    const fullPath = path.join(dir, BINARY_NAME);
    if (fs.existsSync(fullPath)) {
      try {
        const real = fs.realpathSync(fullPath);
        if (real !== selfPath) {
          return fullPath;
        }
      } catch (_) {}
    }
  }

  return null;
}

/**
 * Map system architecture and OS to release target triple
 */
function getTargetTriple() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'darwin') {
    if (arch === 'arm64') return 'aarch64-apple-darwin';
    if (arch === 'x64') return 'x86_64-apple-darwin';
  } else if (platform === 'linux') {
    if (arch === 'x64') return 'x86_64-unknown-linux-gnu';
    if (arch === 'arm64') return 'aarch64-unknown-linux-gnu';
  } else if (platform === 'win32') {
    if (arch === 'x64') return 'x86_64-pc-windows-msvc';
  }

  return null;
}

/**
 * Download file following redirects
 */
function downloadFile(url, destPath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destPath);
    
    function get(currentUrl, redirectCount = 0) {
      if (redirectCount > 5) {
        reject(new Error('Too many redirects while downloading binary'));
        return;
      }

      https.get(currentUrl, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          get(res.headers.location, redirectCount + 1);
          return;
        }

        if (res.statusCode !== 200) {
          reject(new Error(`Failed to download binary: HTTP ${res.statusCode} ${res.statusMessage}`));
          return;
        }

        res.pipe(file);
        file.on('finish', () => {
          file.close(() => resolve(destPath));
        });
      }).on('error', (err) => {
        fs.unlink(destPath, () => {});
        reject(err);
      });
    }

    get(url);
  });
}

/**
 * Download and install native binary for current platform
 */
async function downloadAndInstall() {
  const target = getTargetTriple();
  if (!target) {
    throw new Error(`Unsupported platform/architecture: ${process.platform} (${process.arch})`);
  }

  const installDir = path.join(os.homedir(), '.cgrill', 'bin');
  fs.mkdirSync(installDir, { recursive: true });

  const archiveName = `cgrill-${target}.tar.gz`;
  const url = `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${archiveName}`;
  const archivePath = path.join(installDir, archiveName);
  const binaryPath = path.join(installDir, BINARY_NAME);

  console.error(`cgrill: native binary not found locally. Fetching prebuilt binary for ${target}...`);
  console.error(`cgrill: downloading from ${url}`);

  try {
    await downloadFile(url, archivePath);
    // Extract archive
    execSync(`tar -xzf "${archivePath}" -C "${installDir}"`);
    fs.unlinkSync(archivePath);
    fs.chmodSync(binaryPath, 0o755);
    console.error(`cgrill: installed native binary to ${binaryPath}`);
    return binaryPath;
  } catch (err) {
    if (fs.existsSync(archivePath)) {
      try { fs.unlinkSync(archivePath); } catch (_) {}
    }
    throw err;
  }
}

/**
 * Run native binary with forwarded arguments and stdio
 */
function runBinary(binaryPath) {
  const child = spawn(binaryPath, process.argv.slice(2), {
    stdio: 'inherit',
    env: process.env,
  });

  const forwardSignal = (signal) => {
    if (child && !child.killed) {
      child.kill(signal);
    }
  };

  process.on('SIGINT', () => forwardSignal('SIGINT'));
  process.on('SIGTERM', () => forwardSignal('SIGTERM'));

  child.on('error', (err) => {
    console.error(`cgrill: failed to start process:`, err.message);
    process.exit(1);
  });

  child.on('close', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code || 0);
    }
  });
}

async function main() {
  let binaryPath = findNativeBinary();

  if (!binaryPath) {
    try {
      binaryPath = await downloadAndInstall();
    } catch (downloadErr) {
      console.error(`\ncgrill: Could not find or download prebuilt binary.`);
      console.error(`Details: ${downloadErr.message}\n`);
      console.error(`You can install cgrill via any of the following methods:`);
      console.error(`  1. One-line installer:`);
      console.error(`     curl -fsSL https://raw.githubusercontent.com/${GITHUB_REPO}/main/install.sh | sh\n`);
      console.error(`  2. Cargo (from source):`);
      console.error(`     cargo install --git https://github.com/${GITHUB_REPO}.git cgrill-cli\n`);
      console.error(`  3. Homebrew / prebuilt binaries from releases:`);
      console.error(`     https://github.com/${GITHUB_REPO}/releases/latest\n`);
      process.exit(1);
    }
  }

  runBinary(binaryPath);
}

main();
