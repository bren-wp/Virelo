import fs from 'node:fs';

function readJson(path) {
  return JSON.parse(fs.readFileSync(path, 'utf8'));
}

function matchVersion(path, pattern, label) {
  const content = fs.readFileSync(path, 'utf8');
  const match = content.match(pattern);
  if (!match) {
    throw new Error(`Nije moguće pronaći verziju za ${label} u ${path}.`);
  }
  return match[1];
}

const rootPackage = readJson('package.json');
const desktopPackage = readJson('apps/desktop/package.json');
const tauriConfig = readJson('apps/desktop/src-tauri/tauri.conf.json');
const packageLock = readJson('package-lock.json');

const expected = rootPackage.version;
const versions = new Map([
  ['root package.json', expected],
  ['desktop package.json', desktopPackage.version],
  ['Tauri config', tauriConfig.version],
  [
    'Cargo.toml',
    matchVersion(
      'apps/desktop/src-tauri/Cargo.toml',
      /^version\s*=\s*"([^"]+)"/m,
      'Cargo.toml'
    )
  ],
  [
    'Cargo.lock',
    matchVersion(
      'apps/desktop/src-tauri/Cargo.lock',
      /\[\[package\]\]\s*\nname\s*=\s*"virelo-desktop"\s*\nversion\s*=\s*"([^"]+)"/m,
      'Cargo.lock'
    )
  ],
  ['package-lock root', packageLock.version],
  ['package-lock workspace root', packageLock.packages?.['']?.version],
  ['package-lock desktop workspace', packageLock.packages?.['apps/desktop']?.version]
]);

const mismatches = [...versions.entries()].filter(([, version]) => version !== expected);

if (mismatches.length > 0) {
  console.error(`Virelo version mismatch. Očekivana verzija: ${expected}`);
  for (const [label, version] of versions) {
    console.error(`- ${label}: ${version ?? 'nedostaje'}`);
  }
  process.exit(1);
}

process.stdout.write(expected);
