#!/usr/bin/env node
// 校验前端 pallet 注册表(Dart)与链上 construct_runtime 的 pallet 索引一致。
//
// 契约:citizenchain runtime 是 pallet 索引唯一真源;citizenwallet 与 citizenapp 各有一份
// pallet_registry.dart 镜像。链上 pallet 重排后若 Dart 注册表未同步,会往错误 pallet 发交易。
// 本脚本在 CI 静态比对:每个 Dart `xxxPallet = N` 常量,其 N 必须等于链上同名 pallet 的索引。
//
// 用法:node .github/scripts/check-pallet-registry-sync.mjs
// 退出码:0=一致;1=发现漂移或读文件失败。

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

const RUNTIME_LIB = join(repoRoot, 'citizenchain', 'runtime', 'src', 'lib.rs');
const DART_REGISTRIES = [
  join(repoRoot, 'citizenwallet', 'lib', 'signer', 'pallet_registry.dart'),
  join(repoRoot, 'citizenapp', 'lib', 'rpc', 'pallet_registry.dart'),
];

function read(path) {
  try {
    return readFileSync(path, 'utf8');
  } catch (error) {
    console.error(`[pallet-registry-sync] 无法读取 ${path}: ${error.message}`);
    process.exit(1);
  }
}

// 链上真源:construct_runtime 的 `#[runtime::pallet_index(N)] pub type Name = ...;`
function parseChainIndexToName(source) {
  const map = new Map();
  const re = /#\[runtime::pallet_index\((\d+)\)\]\s*\n\s*pub type (\w+)\s*=/g;
  let match;
  while ((match = re.exec(source)) !== null) {
    map.set(Number(match[1]), match[2]);
  }
  return map;
}

// Dart 端 pallet 常量:`static const int? xxxPallet = N;`(排除引用 PalletRegistry 的间接赋值)
function parseDartPalletConsts(source) {
  const consts = [];
  const re = /static const (?:int\s+)?(\w+Pallet)\s*=\s*(\d+);/g;
  let match;
  while ((match = re.exec(source)) !== null) {
    consts.push({ constName: match[1], index: Number(match[2]) });
  }
  return consts;
}

// Dart camelCase 常量名 → 链上 PascalCase pallet 名:去 `Pallet` 后缀,首字母大写。
// 约定:citizenwallet/citizenapp 的 pallet 常量按链上 pallet 类型名的 camelCase 命名。
function dartConstToPascal(constName) {
  const base = constName.replace(/Pallet$/, '');
  return base.charAt(0).toUpperCase() + base.slice(1);
}

const chainIndexToName = parseChainIndexToName(read(RUNTIME_LIB));
if (chainIndexToName.size === 0) {
  console.error('[pallet-registry-sync] 未能从 construct_runtime 解析到任何 pallet 索引,检查 runtime/src/lib.rs 格式');
  process.exit(1);
}

const problems = [];
let checked = 0;

for (const registryPath of DART_REGISTRIES) {
  const consts = parseDartPalletConsts(read(registryPath));
  const rel = registryPath.slice(repoRoot.length + 1);
  if (consts.length === 0) {
    problems.push(`${rel}: 未解析到任何 pallet 常量(格式变了?)`);
    continue;
  }
  for (const { constName, index } of consts) {
    checked += 1;
    const expectedName = dartConstToPascal(constName);
    const chainName = chainIndexToName.get(index);
    if (chainName === undefined) {
      problems.push(`${rel}: ${constName}=${index} 指向的索引在链上不存在`);
    } else if (chainName !== expectedName) {
      problems.push(
        `${rel}: ${constName}=${index} 期望链上 pallet「${expectedName}」,但索引 ${index} 实为「${chainName}」(pallet 重排未同步)`,
      );
    }
  }
}

if (problems.length > 0) {
  console.error('[pallet-registry-sync] 前端 pallet 注册表与链上 construct_runtime 不一致:');
  for (const problem of problems) console.error(`  ✗ ${problem}`);
  console.error('修复:更新对应 pallet_registry.dart 使索引与 citizenchain/runtime/src/lib.rs 一致。');
  process.exit(1);
}

console.log(`[pallet-registry-sync] OK:${checked} 个前端 pallet 常量与链上 construct_runtime 全部一致。`);
