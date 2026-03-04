import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const uiRoot = path.resolve(scriptDir, '..');
const chainRoot = path.resolve(uiRoot, '..');
const srcTauriDir = path.join(uiRoot, 'src-tauri');
const binariesDir = path.join(srcTauriDir, 'binaries');

const hostTriple = execSync('rustc -vV', { encoding: 'utf8' })
  .split('\n')
  .find((line) => line.startsWith('host:'))
  ?.split(':')[1]
  ?.trim();

if (!hostTriple) {
  throw new Error('failed to resolve rust host target triple');
}

console.log(`building citizenchain node sidecar for ${hostTriple}`);
execSync('cargo build -p node --release', {
  cwd: chainRoot,
  stdio: 'inherit',
});

const releaseDir = path.join(chainRoot, 'target', 'release');
const sourceCandidates = [
  path.join(releaseDir, 'citizenchain-node'),
  path.join(releaseDir, 'citizenchain-node.exe'),
  path.join(releaseDir, 'node'),
  path.join(releaseDir, 'node.exe'),
];
const sourceBin = sourceCandidates.find((candidate) => fs.existsSync(candidate));

if (!sourceBin) {
  throw new Error('cannot find built node binary in target/release');
}

fs.mkdirSync(binariesDir, { recursive: true });

const isExe = sourceBin.endsWith('.exe');
const baseName = isExe ? 'citizenchain-node.exe' : 'citizenchain-node';
const targetName = isExe
  ? `citizenchain-node-${hostTriple}.exe`
  : `citizenchain-node-${hostTriple}`;

fs.copyFileSync(sourceBin, path.join(binariesDir, baseName));
fs.copyFileSync(sourceBin, path.join(binariesDir, targetName));

console.log(`sidecar copied: ${baseName}, ${targetName}`);
