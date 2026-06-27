#!/usr/bin/env node
// 行政区字典数据包生成器(ADR-021 §A2)。
//
// 唯一真源 = citizenchain/registry/src/cid/china/china.sqlite。本生成器**直接 dump 三表,零映射**:
// 任何「修正名字」逻辑禁止进此文件——改名只改 china.sqlite,这里纯搬运。
// 铁律:china.sqlite 行政区 code 不可变不复用(见 memory/07-ai/agent-rules.md)。
//
// 产物(按省分片,客户端按需懒加载;首启灌 Isar AdminDivisionEntity 作字典):
//   assets/admin_divisions/manifest.json
//     = { version, generated_at, china_sqlite_sha256, province_count, city_count, town_count }
//   assets/admin_divisions/provinces.json          = [{ code, name }]
//   assets/admin_divisions/cities/<省code>.json    = [{ code, name }]
//   assets/admin_divisions/towns/<省code>.json     = [{ city_code, code, name }]
//
// manifest 带 china_sqlite_sha256 + version,与机构包同批生成、版本耦合:客户端可校验
// 机构包与字典是否同一份 china.sqlite 派生(hash 不一致即提示需更新)。
//
// 用法:
//   node tools/generate_admin_division_bundle.mjs [--version 2] [--db <路径>]

import { DatabaseSync } from 'node:sqlite';
import { writeFileSync, mkdirSync, readFileSync, rmSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT_DIR = join(__dirname, '..', 'assets', 'admin_divisions');
const DEFAULT_DB = resolve(
  __dirname,
  '..',
  '..',
  'citizenchain',
  'registry',
  'src',
  'cid',
  'china',
  'china.sqlite',
);

function arg(name, fallback) {
  const i = process.argv.indexOf(name);
  return i >= 0 && i + 1 < process.argv.length ? process.argv[i + 1] : fallback;
}

function main() {
  const dbPath = arg('--db', DEFAULT_DB);
  const sha256 = createHash('sha256').update(readFileSync(dbPath)).digest('hex');

  const db = new DatabaseSync(dbPath, { readOnly: true });
  const metadataVersion = db
    .prepare("SELECT value FROM metadata WHERE key = 'admin_division_version'")
    .get()?.value;
  const version = arg('--version', metadataVersion ? String(metadataVersion) : '0');

  // 省:全量一份
  const provinces = db
    .prepare('SELECT code, name FROM provinces ORDER BY sort_order, code')
    .all();

  // 市:按省分片
  const cities = db
    .prepare('SELECT province_code, code, name FROM cities ORDER BY province_code, sort_order, code')
    .all();
  const citiesByProv = new Map();
  for (const r of cities) {
    if (!citiesByProv.has(r.province_code)) citiesByProv.set(r.province_code, []);
    citiesByProv.get(r.province_code).push({ code: r.code, name: r.name });
  }

  // 镇:按省分片(最大头)
  const towns = db
    .prepare('SELECT province_code, city_code, code, name FROM towns ORDER BY province_code, city_code, code')
    .all();
  const townsByProv = new Map();
  for (const r of towns) {
    if (!townsByProv.has(r.province_code)) townsByProv.set(r.province_code, []);
    townsByProv.get(r.province_code).push({ city_code: r.city_code, code: r.code, name: r.name });
  }

  db.close();

  // 中文注释:先清空分片目录,避免省 code 改名后旧分片继续留在安装包中。
  rmSync(join(OUT_DIR, 'cities'), { recursive: true, force: true });
  rmSync(join(OUT_DIR, 'towns'), { recursive: true, force: true });
  mkdirSync(join(OUT_DIR, 'cities'), { recursive: true });
  mkdirSync(join(OUT_DIR, 'towns'), { recursive: true });

  writeFileSync(join(OUT_DIR, 'provinces.json'), JSON.stringify(provinces, null, 0));
  for (const [pcode, list] of citiesByProv) {
    writeFileSync(join(OUT_DIR, 'cities', `${pcode}.json`), JSON.stringify(list, null, 0));
  }
  for (const [pcode, list] of townsByProv) {
    writeFileSync(join(OUT_DIR, 'towns', `${pcode}.json`), JSON.stringify(list, null, 0));
  }

  // 省级内容版本(增量同步用):客户端按 ver 跳过没变的省,只 reconcile 变了的省。
  // ver = 该省"市分片 + 镇分片"内容的 sha256 前 16 位,内容(含改名/删码/重排)一变即变。
  const provinceVersions = provinces.map((p) => {
    const payload =
      JSON.stringify(citiesByProv.get(p.code) ?? []) +
      JSON.stringify(townsByProv.get(p.code) ?? []);
    return { code: p.code, ver: createHash('sha256').update(payload).digest('hex').slice(0, 16) };
  });

  writeFileSync(
    join(OUT_DIR, 'manifest.json'),
    JSON.stringify(
      {
        version,
        generated_at: version,
        china_sqlite_sha256: sha256,
        province_count: provinces.length,
        city_count: cities.length,
        town_count: towns.length,
        // 省级版本表:[{ code, ver }],客户端逐省比对,只重灌 ver 变了的省。
        provinces: provinceVersions,
      },
      null,
      2,
    ),
  );

  console.log(
    `行政区字典生成完成:省 ${provinces.length} / 市 ${cities.length} / 镇 ${towns.length}` +
      `\n  version=${version}\n  china_sqlite_sha256=${sha256.slice(0, 16)}…\n  out=${OUT_DIR}`,
  );
}

main();
