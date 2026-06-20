import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const cbPath = path.join(repoRoot, 'citizenchain/runtime/primitives/china/china_cb.rs');
const chPath = path.join(repoRoot, 'citizenchain/runtime/primitives/china/china_ch.rs');
const outPath = path.join(
  repoRoot,
  'wuminapp/lib/governance/organization-manage/governance_institution_registry.generated.dart',
);
const wuminOutPath = path.join(repoRoot, 'wumin/lib/chain/institutions.dart');

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
    `      mainAccount: ${dartString(item.mainAccount)},`,
    `      feeAccount: ${dartString(item.feeAccount)},`,
  ];
  if (item.anquanAccount) {
    lines.push(`      anquanAccount: ${dartString(item.anquanAccount)},`);
  }
  if (item.heAccount) {
    lines.push(`      heAccount: ${dartString(item.heAccount)},`);
  }
  if (item.stakeAccount) {
    lines.push(`      stakeAccount: ${dartString(item.stakeAccount)},`);
  }
  lines.push('    ),', '  ),');
  return lines.join('\n');
}

function walletType(item) {
  if (item.orgType === 'nrc') return 'InstitutionType.nrc';
  if (item.orgType === 'prc') return 'InstitutionType.prc';
  return 'InstitutionType.prb';
}

function walletInstitution(item) {
  return [
    '  Institution(',
    `    sfidNumber: ${dartString(item.sfidNumber)},`,
    `    name: ${dartString(item.name)},`,
    `    type: ${walletType(item)},`,
    '  ),',
  ].join('\n');
}

const cbSource = fs.readFileSync(cbPath, 'utf8');
const chSource = fs.readFileSync(chPath, 'utf8');
const safetyFundMatch = /NRC_ANQUAN_ACCOUNT:\s*\[u8;\s*32\]\s*=\s*hex!\("([0-9a-fA-F]+)"\)/.exec(
  cbSource,
);
if (!safetyFundMatch) throw new Error('missing NRC_ANQUAN_ACCOUNT');
const anquanAccount = safetyFundMatch[1].toLowerCase();
const heFundMatch = /NRC_HE_ACCOUNT:\s*\[u8;\s*32\]\s*=\s*hex!\("([0-9a-fA-F]+)"\)/.exec(
  cbSource,
);
if (!heFundMatch) throw new Error('missing NRC_HE_ACCOUNT');
const heAccount = heFundMatch[1].toLowerCase();

const cbItems = extractStructs(cbSource, 'ChinaCb').map((block, index) => ({
  name: extractField(block, 'sfid_full_name'),
  sfidNumber: extractField(block, 'sfid_number'),
  orgType: index === 0 ? 'nrc' : 'prc',
  mainAccount: extractField(block, 'main_account'),
  feeAccount: extractField(block, 'fee_account'),
  anquanAccount: index === 0 ? anquanAccount : null,
  heAccount: index === 0 ? heAccount : null,
  stakeAccount: null,
}));

const chItems = extractStructs(chSource, 'ChinaCh').map((block) => ({
  name: extractField(block, 'sfid_full_name'),
  sfidNumber: extractField(block, 'sfid_number'),
  orgType: 'prb',
  mainAccount: extractField(block, 'main_account'),
  feeAccount: extractField(block, 'fee_account'),
  anquanAccount: null,
  heAccount: null,
  stakeAccount: extractField(block, 'stake_account'),
}));

if (cbItems.length !== 44) {
  throw new Error(`CHINA_CB count mismatch: ${cbItems.length}`);
}
if (chItems.length !== 43) {
  throw new Error(`CHINA_CH count mismatch: ${chItems.length}`);
}

const content = [
  "part of 'institution_registry.dart';",
  '',
  '// 本文件由 tools/generate_wuminapp_governance_registry.mjs 自动生成。',
  '// 中文注释：治理机构名称、sfid_number 和制度账户地址来自 runtime primitives；管理员必须动态读取链上 AdminsChange::AdminAccounts。',
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
const wuminContent = [
  '// 链上机构中文名注册表（公民钱包签名校验用）。',
  '//',
  '// 本文件由 tools/generate_wuminapp_governance_registry.mjs 自动生成。',
  '// 中文注释：唯一事实源是 citizenchain/runtime/primitives/china/china_{cb,ch}.rs。',
  '// 冷钱包用同一套映射把 sfid_number 还原成中文名，保证交易摘要与解码结果一致。',
  '',
  '/// 机构分类（与服务端 OrgType 对齐）。',
  'enum InstitutionType {',
  '  /// 国家公民储备委员会。',
  '  nrc,',
  '',
  '  /// 省级公民储备委员会。',
  '  prc,',
  '',
  '  /// 省级公民储备银行。',
  '  prb,',
  '}',
  '',
  'class Institution {',
  '  const Institution({',
  '    required this.sfidNumber,',
  '    required this.name,',
  '    required this.type,',
  '  });',
  '',
  '  final String sfidNumber;',
  '  final String name;',
  '  final InstitutionType type;',
  '}',
  '',
  '/// 国储会（1）。',
  'const List<Institution> kNationalCouncils = [',
  walletInstitution(cbItems[0]),
  '];',
  '',
  '/// 省储会（43）。',
  'const List<Institution> kProvincialCouncils = [',
  cbItems.slice(1).map(walletInstitution).join('\n'),
  '];',
  '',
  '/// 省储行（43）。',
  'const List<Institution> kProvincialBanks = [',
  chItems.map(walletInstitution).join('\n'),
  '];',
  '',
  '/// 所有机构（87）。按服务端 find_entry 的查找顺序：NRC → PRC → PRB。',
  'final List<Institution> kAllInstitutions = List.unmodifiable([',
  '  ...kNationalCouncils,',
  '  ...kProvincialCouncils,',
  '  ...kProvincialBanks,',
  ']);',
  '',
  '/// 根据 sfid_number 查找机构中文名（任意类型：国储会 / 省储会 / 省储行）。',
  '///',
  '/// 返回 null 表示链上交易含未知机构。若遇到此情况，说明链端常量与公民钱包',
  '/// 机构注册表未对齐，应重新运行生成器。',
  'String? sfidFullName(String sfidNumber) {',
  '  for (final inst in kAllInstitutions) {',
  '    if (inst.sfidNumber == sfidNumber) return inst.name;',
  '  }',
  '  return null;',
  '}',
  '',
].join('\n');

fs.writeFileSync(wuminOutPath, wuminContent, 'utf8');
console.log(`generated ${path.relative(repoRoot, outPath)} (${cbItems.length + chItems.length} institutions)`);
console.log(`generated ${path.relative(repoRoot, wuminOutPath)} (${cbItems.length + chItems.length} institutions)`);
