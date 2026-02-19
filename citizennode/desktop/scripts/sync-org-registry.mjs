import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(scriptDir, '..');
const root = path.resolve(desktopDir, '..', '..');
const reserveFile = path.join(root, 'primitives', 'src', 'reserve_nodes_const.rs');
const bankFile = path.join(root, 'primitives', 'src', 'shengbank_nodes_const.rs');
const outFile = path.join(desktopDir, 'src', 'constants', 'orgRegistry.generated.ts');

function normalizeOrgName(raw) {
  return raw
    .replace('公民', '')
    .replace('权威节点', '')
    .replace('权益节点', '')
    .replace('  ', ' ')
    .trim();
}

function parseBlocks(content) {
  const blocks = [];
  const nodeRegex = /node_name:\s*"([^"]+)"/g;
  const markers = [];
  let match;
  while ((match = nodeRegex.exec(content)) !== null) {
    markers.push({ index: match.index, nodeName: match[1] });
  }

  for (let i = 0; i < markers.length; i += 1) {
    const start = markers[i].index;
    const end = i + 1 < markers.length ? markers[i + 1].index : content.length;
    const snippet = content.slice(start, end);

    const nodeName = markers[i].nodeName;
    const firstAdmin = snippet.match(/admins:\s*&\[[\s\S]*?hex!\("([0-9a-fA-F]{64})"\)/);
    if (!firstAdmin) continue;

    const provinceMatch = nodeName.match(/^(.+?)省/);
    blocks.push({
      nodeName,
      organizationName: normalizeOrgName(nodeName),
      province: provinceMatch ? provinceMatch[1] : undefined,
      adminHex: `0x${firstAdmin[1].toLowerCase()}`
    });
  }

  return blocks;
}

const reserveContent = fs.readFileSync(reserveFile, 'utf8');
const bankContent = fs.readFileSync(bankFile, 'utf8');

const reserveNodes = parseBlocks(reserveContent);
const bankNodes = parseBlocks(bankContent);

const rows = [];
reserveNodes.forEach((item, idx) => {
  rows.push({
    role: idx === 0 ? 'nrc' : 'prc',
    organizationName: item.organizationName,
    province: idx === 0 ? undefined : item.province,
    adminAddress: item.adminHex
  });
});

bankNodes.forEach((item) => {
  rows.push({
    role: 'prb',
    organizationName: item.organizationName,
    province: item.province,
    adminAddress: item.adminHex
  });
});

const body = `/* eslint-disable */\n// Auto-generated from primitives constants. Do not edit manually.\nimport type { OrganizationRegistryItem } from './orgRegistry.types';\n\nexport const ORG_REGISTRY: OrganizationRegistryItem[] = ${JSON.stringify(rows, null, 2)};\n`;
fs.writeFileSync(outFile, body, 'utf8');

console.log(`generated ${rows.length} org records -> ${outFile}`);
