#!/usr/bin/env node
// 校验密码学金标向量在各端镜像之间保持一致。
//
// 契约:citizenchain/runtime/primitives 是签名域与账户派生的规范实现,其 tests/fixtures/
// 下的向量文件是唯一真源(文件 `_comment` 已自述 "canonical, Rust 权威源")。
// citizenapp / citizenwallet / Worker 各自持有镜像副本,供本端金标测试独立回归。
//
// 为什么需要本脚本:各端金标测试只验证「本端实现算出的值 == 本端向量文件」。同时改动
// 本端实现与本端向量,该端 CI 依旧全绿,两端就此静默分叉。只有跨端比对能拦住这种漂移。
//
// 比对规则:
//   1. 按语义键对齐,**禁止按数组下标对齐**——各端向量顺序与条数本就不同,
//      按下标比会产生大量假阳性(2026-08-01 审计曾因此误报 45 处冲突)。
//   2. 只比密码学值。`layout` / `_comment` / `source` 等人类可读描述各端措辞不同
//      (如 `pubkey(32)` 与 `signer_public_key(32)`),属噪音,强制统一只会逼人改文案凑 CI。
//   3. 镜像必须是真源的子集:镜像出现真源没有的向量即失败(说明该端自造了向量)。
//   4. 镜像未覆盖到的真源向量只报告、不失败——各端职责不同,覆盖子集是正常设计。
//
// 用法:node .github/scripts/check-golden-vectors-sync.mjs
// 退出码:0=一致;1=发现漂移或读文件失败。

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join, relative } from 'node:path';

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), '..', '..');

const CANONICAL_DIR = join(
  repoRoot, 'citizenchain', 'runtime', 'primitives', 'tests', 'fixtures',
);

/**
 * 各金标组的比对定义。
 *
 * keyFields   决定「同一条向量」的语义键,必须唯一确定该向量的密码学输出。
 * valueFields 参与比对的密码学值;缺字段即视为不一致(挡住镜像悄悄删字段)。
 * topFields   顶层标量参数,四端必须全等。
 */
const GROUPS = [
  {
    name: 'signing_domain_vectors',
    file: 'signing_domain_vectors.json',
    // op_tag + payload 唯一决定 blake2_256(GMB || op_tag || payload)
    keyFields: ['op_tag', 'scale_payload_hex'],
    valueFields: ['message_hex'],
    topFields: ['domain'],
    mirrors: [
      'citizenapp/test/signer/fixtures/signing_domain_vectors.json',
      // Worker(TS) 与 citizenwallet 不在此登记:两端都没有镜像文件,分别由
      // citizenapp/cloudflare/test/signing_message.test.ts 和
      // citizenwallet/test/signer/signing_domain_golden_test.dart 直读上面这份真源。
      // 零副本比"副本 + 比对"更强——漂移在物理上不可能发生。
    ],
  },
  {
    name: 'binary_prefix_domain_vectors',
    file: 'binary_prefix_domain_vectors.json',
    keyFields: ['name'],
    valueFields: ['op_tag', 'prefix_hex', 'payload_hex', 'total_len'],
    topFields: ['domain'],
    mirrors: [
      'citizenapp/test/signer/fixtures/binary_prefix_domain_vectors.json',
      'citizenwallet/test/signer/fixtures/binary_prefix_domain_vectors.json',
    ],
  },
  {
    name: 'account_derive_vectors',
    file: 'account_derive_vectors.json',
    keyFields: ['cid_number', 'kind'],
    valueFields: ['account_id'],
    topFields: ['domain', 'ss58_format'],
    mirrors: [
      'citizenapp/test/governance/shared/fixtures/account_derive_vectors.json',
    ],
  },
];

let failed = false;

function fail(message) {
  console.error(`[golden-vectors] ${message}`);
  failed = true;
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch (error) {
    fail(`无法读取或解析 ${relative(repoRoot, path)}: ${error.message}`);
    return null;
  }
}

/** hex 与标识符大小写在各端书写习惯不同,比对前统一归一;数字保持原值。 */
function normalize(value) {
  return typeof value === 'string' ? value.toLowerCase() : value;
}

function vectorKey(vector, keyFields) {
  return keyFields.map((field) => normalize(vector[field])).join('|');
}

/** 建索引,顺带挡住同一文件内语义键重复(会让比对静默漏掉后一条)。 */
function indexVectors(vectors, keyFields, label) {
  const map = new Map();
  for (const vector of vectors) {
    const key = vectorKey(vector, keyFields);
    if (map.has(key)) {
      fail(`${label} 存在重复向量键 ${key}`);
    }
    map.set(key, vector);
  }
  return map;
}

function checkGroup(group) {
  const canonicalPath = join(CANONICAL_DIR, group.file);
  const canonical = readJson(canonicalPath);
  if (canonical === null) return;
  if (!Array.isArray(canonical.vectors) || canonical.vectors.length === 0) {
    fail(`${group.name} 真源没有向量`);
    return;
  }

  const canonicalMap = indexVectors(
    canonical.vectors, group.keyFields, `${group.name} 真源`,
  );

  for (const mirrorRelative of group.mirrors) {
    const mirror = readJson(join(repoRoot, mirrorRelative));
    if (mirror === null) continue;

    for (const field of group.topFields) {
      if (normalize(mirror[field]) !== normalize(canonical[field])) {
        fail(
          `${mirrorRelative} 顶层 ${field} 与真源不一致:` +
          `真源=${canonical[field]} 镜像=${mirror[field]}`,
        );
      }
    }

    if (!Array.isArray(mirror.vectors)) {
      fail(`${mirrorRelative} 缺少 vectors 数组`);
      continue;
    }
    const mirrorMap = indexVectors(
      mirror.vectors, group.keyFields, mirrorRelative,
    );

    for (const [key, mirrorVector] of mirrorMap) {
      const canonicalVector = canonicalMap.get(key);
      if (canonicalVector === undefined) {
        fail(
          `${mirrorRelative} 存在真源没有的向量 ${key};` +
          `镜像不得自造向量,请先在 ${relative(repoRoot, canonicalPath)} 登记`,
        );
        continue;
      }
      for (const field of group.valueFields) {
        const expected = normalize(canonicalVector[field]);
        const actual = normalize(mirrorVector[field]);
        if (expected === undefined) {
          fail(`${group.name} 真源向量 ${key} 缺少字段 ${field}`);
          continue;
        }
        if (actual !== expected) {
          fail(
            `${mirrorRelative} 向量 ${key} 的 ${field} 与真源不一致\n` +
            `    真源 = ${canonicalVector[field]}\n` +
            `    镜像 = ${mirrorVector[field]}`,
          );
        }
      }
    }

    const uncovered = [...canonicalMap.keys()].filter((k) => !mirrorMap.has(k));
    if (uncovered.length > 0) {
      // 覆盖子集是正常设计(各端职责不同),只提示便于人工判断是否遗漏。
      console.log(
        `[golden-vectors] 提示:${mirrorRelative} 未覆盖 ${uncovered.length} 条真源向量:` +
        ` ${uncovered.join(', ')}`,
      );
    }
    console.log(
      `[golden-vectors] ${group.name}: ${mirrorRelative} 已比对 ${mirrorMap.size} 条`,
    );
  }
}

for (const group of GROUPS) {
  checkGroup(group);
}

if (failed) {
  console.error(
    '[golden-vectors] 金标向量跨端不一致。' +
    '真源在 citizenchain/runtime/primitives/tests/fixtures/,改动真源后必须同步全部镜像。',
  );
  process.exit(1);
}

console.log('[golden-vectors] 全部金标向量跨端一致');
