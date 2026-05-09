import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const cbPath = path.join(repoRoot, 'citizenchain/runtime/primitives/china/china_cb.rs');
const chPath = path.join(repoRoot, 'citizenchain/runtime/primitives/china/china_ch.rs');
const outPath = path.join(
  repoRoot,
  'wuminapp/lib/institution/governance_institution_registry.generated.dart',
);

function extractField(block, name) {
  const quoted = new RegExp(`${name}:\\s*"([^"]+)"`).exec(block);
  if (quoted) return quoted[1];
  const hex = new RegExp(`${name}:\\s*hex!\\("([0-9a-fA-F]+)"\\)`).exec(block);
  if (hex) return hex[1].toLowerCase();
  throw new Error(`missing ${name} in block:\n${block.slice(0, 240)}`);
}

function extractStructs(source, structName) {
  const blocks = [];
  const pattern = new RegExp(`${structName}\\s*\\{([\\s\\S]*?)\\n\\s*\\},`, 'g');
  let match;
  while ((match = pattern.exec(source)) !== null) {
    blocks.push(match[1]);
  }
  return blocks;
}

function dartString(value) {
  return `'${value.replaceAll('\\', '\\\\').replaceAll("'", "\\'")}'`;
}

function dartInstitution(item) {
  const lines = [
    '  InstitutionInfo(',
    `    name: ${dartString(item.name)},`,
    `    sfidNumber: ${dartString(item.sfidNumber)},`,
    `    orgType: OrgType.${item.orgType},`,
    '    accounts: InstitutionAccounts(',
    `      mainAddress: ${dartString(item.mainAddress)},`,
    `      feeAddress: ${dartString(item.feeAddress)},`,
  ];
  if (item.safetyFundAddress) {
    lines.push(`      safetyFundAddress: ${dartString(item.safetyFundAddress)},`);
  }
  if (item.stakeAddress) {
    lines.push(`      stakeAddress: ${dartString(item.stakeAddress)},`);
  }
  lines.push('    ),', '  ),');
  return lines.join('\n');
}

const cbSource = fs.readFileSync(cbPath, 'utf8');
const chSource = fs.readFileSync(chPath, 'utf8');
const safetyFundMatch = /NRC_ANQUAN_ADDRESS:\s*\[u8;\s*32\]\s*=\s*hex!\("([0-9a-fA-F]+)"\)/.exec(
  cbSource,
);
if (!safetyFundMatch) throw new Error('missing NRC_ANQUAN_ADDRESS');
const safetyFundAddress = safetyFundMatch[1].toLowerCase();

const cbItems = extractStructs(cbSource, 'ChinaCb').map((block, index) => ({
  name: extractField(block, 'sfid_name'),
  sfidNumber: extractField(block, 'sfid_number'),
  orgType: index === 0 ? 'nrc' : 'prc',
  mainAddress: extractField(block, 'main_address'),
  feeAddress: extractField(block, 'fee_address'),
  safetyFundAddress: index === 0 ? safetyFundAddress : null,
  stakeAddress: null,
}));

const chItems = extractStructs(chSource, 'ChinaCh').map((block) => ({
  name: extractField(block, 'sfid_name'),
  sfidNumber: extractField(block, 'sfid_number'),
  orgType: 'prb',
  mainAddress: extractField(block, 'main_address'),
  feeAddress: extractField(block, 'fee_address'),
  safetyFundAddress: null,
  stakeAddress: extractField(block, 'stake_address'),
}));

if (cbItems.length !== 44) {
  throw new Error(`CHINA_CB count mismatch: ${cbItems.length}`);
}
if (chItems.length !== 43) {
  throw new Error(`CHINA_CH count mismatch: ${chItems.length}`);
}

const content = [
  "part of 'institution_data.dart';",
  '',
  '// 本文件由 tools/generate_wuminapp_governance_registry.mjs 自动生成。',
  '// 中文注释：治理机构名称、身份 ID 和制度账户地址来自 runtime primitives；管理员必须动态读取链上 AdminsChange::Subjects。',
  '',
  '/// 国储会（1 个）。',
  'const List<InstitutionInfo> kNationalCouncil = [',
  dartInstitution(cbItems[0]),
  '];',
  '',
  '/// 省储会（43 个）。',
  'const List<InstitutionInfo> kProvincialCouncils = [',
  cbItems.slice(1).map(dartInstitution).join('\n'),
  '];',
  '',
  '/// 省储行（43 个）。',
  'const List<InstitutionInfo> kProvincialBanks = [',
  chItems.map(dartInstitution).join('\n'),
  '];',
  '',
].join('\n');

fs.writeFileSync(outPath, content, 'utf8');
console.log(`generated ${path.relative(repoRoot, outPath)} (${cbItems.length + chItems.length} institutions)`);
