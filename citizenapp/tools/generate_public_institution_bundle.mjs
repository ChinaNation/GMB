#!/usr/bin/env node
// CitizenApp 公权机构 finalized 链快照生成器。
//
// 从节点 JSON-RPC 的同一 finalized 块读取 `PublicManage.Institutions` 与
// `PublicManage.InstitutionAccounts`，直接生成链快照索引。
// 生成结果只是 App 本地查询索引；身份、绑定、付款和权限仍在操作前精确读链。
//
// 用法:
//   CHAIN_RPC_URL=http://127.0.0.1:9944 node tools/generate_public_institution_bundle.mjs
//   node tools/generate_public_institution_bundle.mjs \
//     --rpc-url http://127.0.0.1:9944 --at 0x... \
//     --chainspec ../target/chainspec/chainspec.app.json \
//     --out-dir ../target/chainspec/public_institutions
//
// 正式冻结由 bake-chainspec.sh 显式传入块 0、轻形态 chainspec 和暂存目录；
// 默认路径只服务于人工刷新当前 App 缓存，不构成第二个创世真源。
// Cloudflare Access HTTP 入口可选从环境读取 `CF_ACCESS_CLIENT_ID` 和
// `CF_ACCESS_CLIENT_SECRET`，脚本不会把凭据写入产物或日志。

import { createHash } from 'node:crypto';
import { writeFileSync, mkdirSync, readFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_DIR = arg(
  '--out-dir',
  join(__dirname, '..', 'assets', 'public_institutions'),
);
const CHAIN_SPEC = arg(
  '--chainspec',
  join(__dirname, '..', 'assets', 'chainspec.json'),
);
const CHAIN_RPC_URL = arg(
  '--rpc-url',
  process.env.CHAIN_RPC_URL || 'http://127.0.0.1:9944',
);
const PAGE_SIZE = 500;
const BATCH_SIZE = 200;

// twox128("PublicManage") + twox128(storage)。名称变化时必须同时更新链元数据契约。
const INSTITUTIONS_PREFIX =
  '0x3fdf0b7e6001a27b1f4c2913ca162bf72ef145c44f710c6fe55cae381219f7b2';
const ACCOUNTS_PREFIX =
  '0x3fdf0b7e6001a27b1f4c2913ca162bf7ca63afb529001c3370b9aa5ba2bd1fd7';

const PROVINCES = [
  ['ZS', '中枢省'], ['LN', '岭南省'], ['GD', '广东省'], ['GX', '广西省'],
  ['FJ', '福建省'], ['HN', '海南省'], ['YN', '云南省'], ['GZ', '贵州省'],
  ['HU', '湖南省'], ['JX', '江西省'], ['ZJ', '浙江省'], ['JS', '江苏省'],
  ['SD', '山东省'], ['SX', '山西省'], ['HE', '河南省'], ['HB', '河北省'],
  ['HI', '湖北省'], ['SI', '陕西省'], ['CQ', '重庆省'], ['SC', '四川省'],
  ['GS', '甘肃省'], ['BP', '北平省'], ['HA', '海滨省'], ['SJ', '松江省'],
  ['LJ', '龙江省'], ['JL', '吉林省'], ['LI', '辽宁省'], ['NX', '宁夏省'],
  ['QH', '青海省'], ['AH', '安徽省'], ['TW', '台湾省'], ['XZ', '西藏省'],
  ['XJ', '新疆省'], ['XK', '西康省'], ['AL', '阿里省'], ['CL', '葱岭省'],
  ['YL', '伊犁省'], ['HX', '河西省'], ['KL', '昆仑省'], ['HT', '河套省'],
  ['RH', '热河省'], ['XA', '兴安省'], ['HJ', '合江省'],
];
const RESERVED_ACCOUNT_NAMES = new Set([
  '主账户', '费用账户', '永久质押', '安全基金', '两和基金',
]);

function arg(name, fallback) {
  const index = process.argv.indexOf(name);
  return index >= 0 && index + 1 < process.argv.length
    ? process.argv[index + 1]
    : fallback;
}

function sha256Text(text) {
  return createHash('sha256').update(text).digest('hex');
}

function sha256File(path) {
  return existsSync(path) ? sha256Text(readFileSync(path)) : '';
}

function bytes(hex) {
  const clean = hex.startsWith('0x') ? hex.slice(2) : hex;
  return Buffer.from(clean, 'hex');
}

function readCompact(data, offset) {
  const first = data[offset];
  const mode = first & 3;
  if (mode === 0) return [first >> 2, 1];
  if (mode === 1) return [data.readUInt16LE(offset) >> 2, 2];
  if (mode === 2) return [data.readUInt32LE(offset) >>> 2, 4];
  throw new Error('不支持大整数 SCALE compact 长度');
}

function readVec(data, offset) {
  const [length, lengthBytes] = readCompact(data, offset);
  const start = offset + lengthBytes;
  const end = start + length;
  if (end > data.length) throw new Error('SCALE Vec 越界');
  return [data.subarray(start, end), end];
}

// SCALE Option:None=1 字节(0x00);Some=0x01 + 值。readInner(data, o) 返回值结束后的
// offset。仅用于跳过字段推进 offset(生成包不需要这些法定代表人字段的值)。
function skipOption(data, offset, readInner) {
  const tag = data[offset];
  if (tag === 0) return offset + 1;
  if (tag === 1) return readInner(data, offset + 1);
  throw new Error(`非法 SCALE Option tag=${tag}`);
}

// 机构码真源:从 cid_number 核心段还原,与链上 parse_cid_number_parts 逐字一致,
// 免疫链上 InstitutionInfo 结构体字段漂移(primitives/cid/number.rs)。
// 格式 R5(5)-核心段(5)-N9(9)-D4(4);核心段 seg2[3] 为数字→3 字符码,为字母→4 字符码。
function institutionCodeFromCid(cidNumber) {
  const seg2 = cidNumber.split('-')[1] ?? '';
  if (seg2.length !== 5) throw new Error(`cid 核心段格式非法: ${cidNumber}`);
  const isDigit = seg2[3] >= '0' && seg2[3] <= '9';
  return isDigit ? seg2.slice(0, 3) : seg2.slice(0, 4);
}

function decodeInstitutionKey(keyHex) {
  const key = bytes(keyHex);
  const [cid] = readVec(key, 48);
  return cid.toString('utf8');
}

function decodeAccountKey(keyHex) {
  const key = bytes(keyHex);
  const [cid, afterCid] = readVec(key, 48);
  const [accountName] = readVec(key, afterCid + 16);
  return [cid.toString('utf8'), accountName.toString('utf8')];
}

function decodeInstitution(cidNumber, valueHex) {
  const value = bytes(valueHex);
  let offset = 0;
  const [fullName, afterFullName] = readVec(value, offset);
  offset = afterFullName;
  const [shortName, afterShortName] = readVec(value, offset);
  offset = afterShortName;
  const [townCode, afterTownCode] = readVec(value, offset);
  offset = afterTownCode;
  // 链上 InstitutionInfo 在 town_code 与 institution_code 之间还有三个法定代表人 Option
  // 字段,必须逐个跳过,否则后续 created_at/status 偏移全错(字段顺序真源见链端
  // entity-primitives InstitutionInfo)。
  offset = skipOption(value, offset, (d, o) => readVec(d, o)[1]); // Option<AccountName 姓名>
  offset = skipOption(value, offset, (d, o) => readVec(d, o)[1]); // Option<CidNumber>
  offset = skipOption(value, offset, (d, o) => o + 32); // Option<AccountId 32 字节>
  if (offset + 8 > value.length) throw new Error(`机构 ${cidNumber} 链值长度不足`);
  // 机构码不从 value blob 切(易随结构漂移),直接从 cid_number 核心段还原(方案B)。
  const institutionCode = institutionCodeFromCid(cidNumber);
  offset += 4; // 跳过 blob 里的 [u8;4] institution_code
  const createdAt = value.readUInt32LE(offset);
  offset += 4;
  // InstitutionInfo 已删除重复 lifecycle/status；Institutions 中存在即为当前有效机构，
  // 关闭时整条身份记录被清理。派生缓存不得继续猜测历史状态字节。
  if (offset !== value.length) {
    throw new Error(`机构 ${cidNumber} 存在未识别 SCALE 尾部字段`);
  }
  const match = /^([A-Z]{2})(\d{3})-/u.exec(cidNumber);
  if (!match) throw new Error(`机构号格式无效: ${cidNumber}`);
  return {
    cid_number: cidNumber,
    cid_full_name: fullName.toString('utf8'),
    cid_short_name: shortName.toString('utf8'),
    status: 'ACTIVE',
    province_code: match[1],
    city_code: match[2],
    town_code: townCode.toString('utf8'),
    institution_code: institutionCode,
    account_count: 0,
    custom_account_names: [],
    created_at_block: createdAt,
  };
}

class JsonRpc {
  constructor(url) {
    this.url = url;
    this.nextId = 1;
    this.socket = null;
    this.pending = new Map();
  }

  async request(method, params = []) {
    if (this.url.startsWith('http://') || this.url.startsWith('https://')) {
      return this.requestHttp(method, params);
    }
    return this.requestWebSocket(method, params);
  }

  async requestHttp(method, params) {
    const id = this.nextId++;
    const headers = { 'content-type': 'application/json' };
    if (process.env.CF_ACCESS_CLIENT_ID && process.env.CF_ACCESS_CLIENT_SECRET) {
      headers['CF-Access-Client-Id'] = process.env.CF_ACCESS_CLIENT_ID;
      headers['CF-Access-Client-Secret'] = process.env.CF_ACCESS_CLIENT_SECRET;
    }
    const response = await fetch(this.url, {
      method: 'POST',
      headers,
      body: JSON.stringify({ jsonrpc: '2.0', id, method, params }),
    });
    if (!response.ok) throw new Error(`${method} HTTP ${response.status}`);
    const payload = await response.json();
    if (payload.error) throw new Error(`${method}: ${JSON.stringify(payload.error)}`);
    return payload.result;
  }

  async requestWebSocket(method, params) {
    await this.ensureSocket();
    const id = this.nextId++;
    const result = new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
    });
    this.socket.send(JSON.stringify({ jsonrpc: '2.0', id, method, params }));
    return result;
  }

  async ensureSocket() {
    if (this.socket?.readyState === WebSocket.OPEN) return;
    this.socket = new WebSocket(this.url);
    this.socket.addEventListener('message', (event) => {
      const payload = JSON.parse(event.data.toString());
      const pending = this.pending.get(payload.id);
      if (!pending) return;
      this.pending.delete(payload.id);
      if (payload.error) pending.reject(new Error(JSON.stringify(payload.error)));
      else pending.resolve(payload.result);
    });
    await new Promise((resolve, reject) => {
      this.socket.addEventListener('open', resolve, { once: true });
      this.socket.addEventListener('error', reject, { once: true });
    });
  }

  close() {
    this.socket?.close();
  }
}

async function readKeys(rpc, prefix, blockHash) {
  const keys = [];
  let startKey = null;
  for (;;) {
    const page = await rpc.request('state_getKeysPaged', [
      prefix, PAGE_SIZE, startKey, blockHash,
    ]);
    if (!page?.length) break;
    keys.push(...page);
    if (page.length < PAGE_SIZE) break;
    startKey = page.at(-1);
  }
  return keys;
}

async function readValues(rpc, keys, blockHash) {
  const values = new Map();
  for (let start = 0; start < keys.length; start += BATCH_SIZE) {
    const batch = keys.slice(start, start + BATCH_SIZE);
    const result = await rpc.request('state_queryStorageAt', [batch, blockHash]);
    for (const [key, value] of result?.[0]?.changes ?? []) values.set(key, value);
  }
  return values;
}

async function main() {
  if (process.argv.includes('--self-test')) {
    const compactSmall = (length) => {
      if (!Number.isInteger(length) || length < 0 || length >= 64) {
        throw new Error('self-test 只编码单字节 compact');
      }
      return Buffer.from([length << 2]);
    };
    const vec = (value) => {
      const data = Buffer.from(value, 'utf8');
      return Buffer.concat([compactSmall(data.length), data]);
    };
    const raw = Buffer.concat([
      vec('测试公权机构'),
      vec('测试机构'),
      vec(''),
      Buffer.from([0, 0, 0]), // 三个法定代表人 Option=None
      Buffer.from('CAGR', 'ascii'),
      Buffer.from([7, 0, 0, 0]),
    ]);
    const decoded = decodeInstitution(
      'HE036-CAGRA-251174662-2026',
      `0x${raw.toString('hex')}`,
    );
    if (decoded.status !== 'ACTIVE' || decoded.created_at_block !== 7
        || decoded.institution_code !== 'CAGR') {
      throw new Error(`InstitutionInfo SCALE self-test 失败:${JSON.stringify(decoded)}`);
    }
    console.log('public institution SCALE self-test ok');
    return;
  }
  if (!existsSync(CHAIN_SPEC)) {
    throw new Error(`chainspec 不存在: ${CHAIN_SPEC}`);
  }
  const requestedBlockHash = arg('--at', '').trim().toLowerCase();
  if (requestedBlockHash && !/^0x[0-9a-f]{64}$/.test(requestedBlockHash)) {
    throw new Error('--at 必须为 0x + 64 位十六进制块哈希');
  }
  const selectedNames = new Set(
    arg('--provinces', '').split(',').map((value) => value.trim()).filter(Boolean),
  );
  const provinces = selectedNames.size === 0
    ? PROVINCES
    : PROVINCES.filter(([, name]) => selectedNames.has(name));
  if (provinces.length === 0) throw new Error('没有匹配的省份');

  const rpc = new JsonRpc(CHAIN_RPC_URL);
  try {
    const snapshotBlockHash = requestedBlockHash
      || await rpc.request('chain_getFinalizedHead');
    const header = await rpc.request('chain_getHeader', [snapshotBlockHash]);
    const genesisHash = await rpc.request('chain_getBlockHash', [0]);
    if (!header) throw new Error(`找不到快照块头: ${snapshotBlockHash}`);
    if (requestedBlockHash && snapshotBlockHash !== requestedBlockHash) {
      throw new Error(`节点返回了非请求块: ${snapshotBlockHash}`);
    }
    const snapshotBlockNumber = Number.parseInt(header.number, 16);

    const institutionKeys = await readKeys(rpc, INSTITUTIONS_PREFIX, snapshotBlockHash);
    const institutionValues = await readValues(rpc, institutionKeys, snapshotBlockHash);
    const accountKeys = await readKeys(rpc, ACCOUNTS_PREFIX, snapshotBlockHash);

    const accountNames = new Map();
    for (const key of accountKeys) {
      const [cidNumber, accountName] = decodeAccountKey(key);
      if (!accountNames.has(cidNumber)) accountNames.set(cidNumber, []);
      accountNames.get(cidNumber).push(accountName);
    }

    const institutions = [];
    for (const key of institutionKeys) {
      const value = institutionValues.get(key);
      if (!value) continue;
      const institution = decodeInstitution(decodeInstitutionKey(key), value);
      const names = [...new Set(accountNames.get(institution.cid_number) ?? [])];
      institution.account_count = names.length;
      institution.custom_account_names = names
        .filter((name) => !RESERVED_ACCOUNT_NAMES.has(name))
        .sort();
      institutions.push(institution);
    }
    institutions.sort((a, b) => a.cid_number.localeCompare(b.cid_number));

    mkdirSync(OUT_DIR, { recursive: true });
    const shardHashes = {};
    const provinceVersions = [];
    const rootParts = [];
    let total = 0;
    for (const [provinceCode, provinceName] of provinces) {
      const rows = institutions.filter((row) => row.province_code === provinceCode);
      const manifestVersion = sha256Text(JSON.stringify(rows));
      const shard = {
        province_name: provinceName,
        manifest_version: manifestVersion,
        count: rows.length,
        institutions: rows,
      };
      const shardJson = `${JSON.stringify(shard)}\n`;
      writeFileSync(join(OUT_DIR, `${provinceName}.json`), shardJson);
      const shardHash = sha256Text(shardJson);
      shardHashes[provinceName] = shardHash;
      const item = {
        province_name: provinceName,
        manifest_version: manifestVersion,
        shard_hash: shardHash,
        count: rows.length,
      };
      provinceVersions.push(item);
      rootParts.push(item);
      total += rows.length;
      console.log(`  ${provinceName}: ${rows.length} 机构`);
    }

    const publicInstitutionRoot = sha256Text(JSON.stringify(rootParts));
    writeFileSync(
      join(OUT_DIR, 'manifest.json'),
      `${JSON.stringify({
        schema_version: 2,
        chain_id: arg('--chain-id', 'citizenchain'),
        snapshot_block_number: snapshotBlockNumber,
        snapshot_block_hash: snapshotBlockHash,
        genesis_hash: genesisHash,
        state_root: header.stateRoot,
        chainspec_hash: sha256File(CHAIN_SPEC),
        public_institution_root: publicInstitutionRoot,
        version: `${snapshotBlockHash}:${publicInstitutionRoot}`,
        shard_hashes: shardHashes,
        provinces: provinceVersions,
      }, null, 2)}\n`,
    );
    console.log(
      `snapshot #${snapshotBlockNumber}: ${provinces.length} 省，共 ${total} 机构；`
      + `public_institution_root=${publicInstitutionRoot}`,
    );
  } finally {
    rpc.close();
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
