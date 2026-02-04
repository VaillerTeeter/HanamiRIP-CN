#!/usr/bin/env node

/**
 * Cross-platform cleanup script for HanamiRIP-CN
 * Removes build artifacts and node_modules
 */

import { rmSync } from 'fs';
import { resolve } from 'path';
import { fileURLToPath } from 'url';

const rootDir = resolve(fileURLToPath(import.meta.url), '../../');

const pathsToRemove = [
  // 'node_modules',
  'build',
  'dist',
  'src-tauri/gen',
  // 'src-tauri/target',
  // 'src-tauri/baidu_verify/target',
];

// Also try to remove baidu_verify library files
const libPaths = [
  // 'src-tauri/baidu_verify/baidu_verify.dll',
  // 'src-tauri/baidu_verify/libbaidu_verify.so',
  // 'src-tauri/baidu_verify/libbaidu_verify.dylib',
];

console.log('Cleaning up build artifacts...\n');

pathsToRemove.forEach((path) => {
  const fullPath = resolve(rootDir, path);
  try {
    rmSync(fullPath, { recursive: true, force: true });
    console.log(`✓ Removed: ${path}`);
  } catch (err) {
    if (err.code !== 'ENOENT') {
      console.warn(`⚠ Failed to remove ${path}: ${err.message}`);
    }
  }
});

console.log();

libPaths.forEach((path) => {
  const fullPath = resolve(rootDir, path);
  try {
    rmSync(fullPath, { force: true });
    console.log(`✓ Removed: ${path}`);
  } catch (err) {
    if (err.code !== 'ENOENT') {
      console.warn(`⚠ Failed to remove ${path}: ${err.message}`);
    }
  }
});

console.log('\n✓ Cleanup complete!');
