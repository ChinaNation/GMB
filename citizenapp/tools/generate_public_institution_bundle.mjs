#!/usr/bin/env node
// 公权机构创世快照包生成器(ADR-018 §九 混合模式 ①)。
//
// 发布期从 OnChina 链上投影公开接口**keyset 翻页**拉创世公权机构目录,
// 写成 CitizenApp 内置快照缓存:
//   assets/public_institutions/manifest.json =
//     { schema_version, chain_id, snapshot_block_number, snapshot_block_hash,
//       genesis_hash, state_root, public_institution_root, shard_hashes, provinces }
//   assets/public_institutions/<省全名>.json  = { province_name, manifest_version, count, institutions: [...] }
// App 启动后按省级 manifest_version 做本地 reconcile:只写变化行,并删除包内已消失的 cid。
// 快照只作本地缓存,公权机构唯一真源仍是链上状态。
//
// 量级:确定性目录到镇级,单省上万、全国数十万。**必须用 keyset**(after_cid),
// 否则 OFFSET 深翻 O(n²) 会非常慢。
//
// 用法(需 OnChina 后端在跑):
//   ONCHINA_BASE_URL=https://onchina.local:8964 node tools/generate_public_institution_bundle.mjs
//   可选 --provinces 中枢省,岭南省 只生成部分省;--version 2026-06-13 指定包版本。
//   必填 --state-root <块0 state root>;--snapshot-block-hash / --genesis-hash 默认取 BFF 链投影。
//
// 省全名(含"省")与 china.sqlite / OnChina `province` 字段逐字对齐;展示去"省"由客户端做。

import { createHash } from 'node:crypto';
import { writeFileSync, mkdirSync, readFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_DIR = join(__dirname, '..', 'assets', 'public_institutions');
const CHAIN_SPEC = join(__dirname, '..', 'assets', 'chainspec.json');
const BASE_URL = process.env.ONCHINA_BASE_URL || 'https://onchina.local:8964';
const PAGE_SIZE = 500;
// 后端默认限流 120 请求/分钟/IP。页间默认延时 550ms(≈109/min,留余量)+ 429 退避重试。
// 后端临时调高限流(ONCHINA_RATE_LIMIT_PER_MIN=大值)时可设 GEN_DELAY_MS=0 跑满速。
const DELAY_MS = Number(process.env.GEN_DELAY_MS ?? '550');
const MAX_RETRY_429 = 8;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// 43 省规范全名(含中枢省,与 china.sqlite provinces 表逐字对齐,含"省")。
const DEFAULT_PROVINCES = [
  '中枢省', '岭南省', '广东省', '广西省', '福建省', '海南省', '云南省', '贵州省',
  '湖南省', '江西省', '浙江省', '江苏省', '山东省', '山西省', '河南省', '河北省',
  '湖北省', '陕西省', '重庆省', '四川省', '甘肃省', '北平省', '海滨省', '松江省',
  '龙江省', '吉林省', '辽宁省', '宁夏省', '青海省', '安徽省', '台湾省', '西藏省',
  '新疆省', '西康省', '阿里省', '葱岭省', '伊犁省', '河西省', '昆仑省', '河套省',
  '热河省', '兴安省', '合江省',
];

function arg(name, fallback) {
  const i = process.argv.indexOf(name);
  return i >= 0 && i + 1 < process.argv.length ? process.argv[i + 1] : fallback;
}

function sha256Text(text) {
  return createHash('sha256').update(text).digest('hex');
}

function sha256File(path) {
  return existsSync(path) ? sha256Text(readFileSync(path)) : '';
}

async function fetchPage(province, afterCid) {
  const url = new URL(`${BASE_URL}/api/v1/app/public-institutions`);
  url.searchParams.set('province_name', province);
  url.searchParams.set('page_size', String(PAGE_SIZE));
  if (afterCid) url.searchParams.set('after_cid', afterCid);

  // 429 限流退避重试:读 Retry-After,否则指数退避(2s/4s/8s… 上限 30s)。
  for (let attempt = 0; ; attempt++) {
    if (DELAY_MS > 0) await sleep(DELAY_MS);
    const res = await fetch(url);
    if (res.ok) return (await res.json()).data;
    if (res.status === 429 && attempt < MAX_RETRY_429) {
      const retryAfter = Number(res.headers.get('retry-after'));
      const waitMs = Number.isFinite(retryAfter) && retryAfter > 0
        ? retryAfter * 1000
        : Math.min(2000 * 2 ** attempt, 30000);
      console.log(`    ${province} 限流 429,等待 ${(waitMs / 1000).toFixed(0)}s 重试…`);
      await sleep(waitMs);
      continue;
    }
    throw new Error(`${province} page failed: ${res.status}`);
  }
}

async function fetchVersion(province) {
  const url = new URL(`${BASE_URL}/api/v1/app/public-institutions/version`);
  url.searchParams.set('province_name', province);
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${province} version failed: ${res.status}`);
  return (await res.json()).data ?? {};
}

async function fetchProvince(province) {
  const institutions = [];
  let afterCid = '';
  let manifestVersion = null;
  // keyset:每页用上一页末尾 cid 作游标,恒定快。
  // eslint-disable-next-line no-constant-condition
  while (true) {
    const data = await fetchPage(province, afterCid);
    manifestVersion = data.manifest_version ?? manifestVersion;
    const items = data.items ?? [];
    institutions.push(...items);
    if (!data.has_more || items.length === 0) break;
    afterCid = data.next_cursor || items[items.length - 1].cid_number;
  }
  return {
    province_name: province,
    manifest_version: manifestVersion ?? '',
    count: institutions.length,
    institutions,
  };
}

async function main() {
  const provincesArg = arg('--provinces', '');
  const chainId = arg('--chain-id', 'citizenchain');
  const provinces = provincesArg
    ? provincesArg.split(',').map((s) => s.trim()).filter(Boolean)
    : DEFAULT_PROVINCES;
  const projection = await fetchVersion(provinces[0]);
  const version = arg('--version', projection.manifest_version || new Date().toISOString());
  const snapshotBlockNumber = Number(
    arg('--snapshot-block-number', projection.chain_block_number?.toString() ?? '0'),
  );
  const snapshotBlockHash = arg(
    '--snapshot-block-hash',
    projection.chain_block_hash || projection.chain_genesis_hash || '',
  );
  const genesisHash = arg('--genesis-hash', projection.chain_genesis_hash || snapshotBlockHash);
  const stateRoot = arg('--state-root', '');
  const chainspecHash = arg('--chainspec-hash', sha256File(CHAIN_SPEC));
  const adminDivisionRoot = arg('--admin-division-root', '');
  if (!snapshotBlockHash || !genesisHash || !stateRoot) {
    throw new Error(
      'public institution bundle requires genesis_hash, snapshot_block_hash and state_root; pass --state-root from genesis-state manifest',
    );
  }

  mkdirSync(OUT_DIR, { recursive: true });
  let total = 0;
  // 省级版本表(增量同步用):[{ province_name, manifest_version }]。
  // 客户端逐省比对 manifest_version,只重灌版本变了的省,没变的省连分片都不读。
  const provinceVersions = [];
  const shardHashes = {};
  const rootParts = [];
  for (const province of provinces) {
    const t0 = Date.now();
    const shard = await fetchProvince(province);
    const shardJson = `${JSON.stringify(shard, null, 0)}\n`;
    writeFileSync(join(OUT_DIR, `${province}.json`), shardJson);
    const shardHash = sha256Text(shardJson);
    shardHashes[province] = shardHash;
    provinceVersions.push({
      province_name: province,
      manifest_version: shard.manifest_version,
      shard_hash: shardHash,
      count: shard.count,
    });
    rootParts.push({
      province_name: province,
      manifest_version: shard.manifest_version,
      shard_hash: shardHash,
      count: shard.count,
    });
    total += shard.count;
    console.log(
      `  ${province}: ${shard.count} 机构 (mv=${shard.manifest_version}) ${((Date.now() - t0) / 1000).toFixed(1)}s`,
    );
  }
  writeFileSync(
    join(OUT_DIR, 'manifest.json'),
    JSON.stringify(
      {
        schema_version: 1,
        chain_id: chainId,
        snapshot_block_number: Number.isFinite(snapshotBlockNumber) ? snapshotBlockNumber : 0,
        snapshot_block_hash: snapshotBlockHash,
        genesis_hash: genesisHash,
        state_root: stateRoot,
        chainspec_hash: chainspecHash,
        admin_division_root: adminDivisionRoot,
        public_institution_root: sha256Text(JSON.stringify(rootParts)),
        version,
        generated_at: version,
        shard_hashes: shardHashes,
        provinces: provinceVersions,
      },
      null,
      2,
    ),
  );
  console.log(`manifest.json 写入完成,version=${version},${provinces.length} 省,共 ${total} 机构。`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
