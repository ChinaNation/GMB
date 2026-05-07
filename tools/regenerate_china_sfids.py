#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
按新规则(D4 年份段 + 市码 000→001)重新生成 china_*.rs 中所有机构的 sfid_number,
输出新旧对照表。

新规则:
- 末段日期由 YYYYMMDD(8 位)缩为 YYYY(4 位),固定 "2026"
- 所有 city_code "000"(省辖市) 一律改为 "001"(各省首府市)
- n9 = blake2b-256(account_pubkey | a3 | province | city | institution | "2026").first_4_bytes_as_u32 % 10^9
  (account_pubkey 用 sfid_name 作为稳定 seed,可复现)
- c1 = checksum 重算

输出:csv 与 markdown 对照表到 stdout。
"""

import hashlib
import re
import sys
from pathlib import Path

ROOT = Path("/Users/rhett/GMB")
PRIMITIVES = ROOT / "citizenchain/runtime/primitives/china"
CITY_CODES = ROOT / "sfid/backend/sfid/city_codes"
PROVINCE_RS = ROOT / "sfid/backend/sfid/province.rs"

ALPHABET = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
TARGET_YEAR = "2026"

PRIMITIVE_FILES = [
    ("china_cb", "ChinaCb"),
    ("china_ch", "ChinaCh"),
    ("china_lf", "ChinaLf"),
    ("china_sf", "ChinaSf"),
    ("china_jc", "ChinaJc"),
    ("china_jy", "ChinaJy"),
    ("china_zf", "ChinaZf"),
]


def hash_text(s: str) -> int:
    """Reproduce sfid::generator::hash_text — blake2b-256 first 4 bytes as u32 LE."""
    h = hashlib.blake2b(s.encode("utf-8"), digest_size=32).digest()
    return int.from_bytes(h[:4], "little")


def checksum(payload: str) -> str:
    """Reproduce sfid::generator::checksum."""
    total = 0
    for idx, ch in enumerate(payload):
        pos = ALPHABET.find(ch)
        if pos < 0:
            pos = 0
        total = (total + (idx + 1) * pos) % 36
    return ALPHABET[total]


def parse_provinces():
    """解析 PROVINCES 数组,返回 {province_code: province_name}。"""
    text = PROVINCE_RS.read_text(encoding="utf-8")
    out = {}
    for m in re.finditer(
        r'ProvinceCode\s*\{\s*name:\s*"([^"]+)"\s*,\s*code:\s*"([^"]+)"', text
    ):
        out[m.group(2)] = m.group(1)
    return out


def parse_cities_for_province(province_code: str):
    """读取 city_codes/{NN}_{XX}.rs,返回 {city_code: city_name}。"""
    matches = list(CITY_CODES.glob(f"*_{province_code}.rs"))
    if not matches:
        return {}
    text = matches[0].read_text(encoding="utf-8")
    out = {}
    # CityCode { name: "锦程市", code: "001", ... }
    for m in re.finditer(
        r'CityCode\s*\{\s*name:\s*"([^"]+)"\s*,\s*code:\s*"([^"]+)"', text
    ):
        out[m.group(2)] = m.group(1)
    return out


def parse_primitive(file_stem: str, struct_name: str):
    """从 china_*.rs 提取所有 (sfid_number, sfid_name) 对。

    2026-05-07 字段顺序调整为 sfid_name 在前 / sfid_number 在后,正则同步。
    """
    p = PRIMITIVES / f"{file_stem}.rs"
    text = p.read_text(encoding="utf-8")
    pattern = re.compile(
        r'sfid_name:\s*"([^"]+)"\s*,\s*sfid_number:\s*"([^"]+)"',
        re.DOTALL,
    )
    return [(m.group(2), m.group(1)) for m in pattern.finditer(text)]


def regenerate(old_sfid: str, sfid_name: str, provinces: dict, cities_cache: dict):
    """根据新规则重新生成 SFID。"""
    parts = old_sfid.split("-")
    if len(parts) != 5:
        raise ValueError(f"bad sfid: {old_sfid}")
    a3 = parts[0]
    r5 = parts[1]
    province_code = r5[:2]
    old_city_code = r5[2:5]
    t2p1c1 = parts[2]
    t2 = t2p1c1[:2]
    p1 = t2p1c1[2:3]
    # 市码:000 → 001;其它保持
    # 用户决议:全部内置机构市码统一 001(无例外,LN002 等也强制改 001)。
    new_city_code = "001"
    # 查省名/市名
    province_name = provinces.get(province_code, province_code)
    if province_code not in cities_cache:
        cities_cache[province_code] = parse_cities_for_province(province_code)
    cities = cities_cache[province_code]
    new_city_name = cities.get(new_city_code, new_city_code)
    # n9 = hash(sfid_name | a3 | province_name | city_name | t2 | "2026") % 10^9
    hash_input = f"{sfid_name}|{a3}|{province_name}|{new_city_name}|{t2}|{TARGET_YEAR}"
    n9 = f"{hash_text(hash_input) % 1_000_000_000:09d}"
    # c1 重算
    new_r5 = f"{province_code}{new_city_code}"
    payload = f"{a3}{new_r5}{t2}{p1}{n9}{TARGET_YEAR}"
    c1 = checksum(payload)
    new_sfid = f"{a3}-{new_r5}-{t2}{p1}{c1}-{n9}-{TARGET_YEAR}"
    return new_sfid


def main():
    provinces = parse_provinces()
    cities_cache = {}
    rows = []
    counts = {"total": 0, "city_changed": 0, "city_kept": 0}
    for stem, struct_name in PRIMITIVE_FILES:
        for old_sfid, name in parse_primitive(stem, struct_name):
            new_sfid = regenerate(old_sfid, name, provinces, cities_cache)
            old_city_code = old_sfid.split("-")[1][2:5]
            new_city_code = new_sfid.split("-")[1][2:5]
            city_changed = old_city_code != new_city_code
            counts["total"] += 1
            if city_changed:
                counts["city_changed"] += 1
            else:
                counts["city_kept"] += 1
            rows.append((stem, name, old_sfid, new_sfid, "→" if city_changed else " "))

    # 输出 markdown 表
    print(f"# 机构 SFID 新旧对照表(共 {counts['total']} 条,市码改动 {counts['city_changed']} 条)\n")
    print("| 文件 | 机构名 | 旧 SFID | 新 SFID | 市码改 |")
    print("|---|---|---|---|---|")
    for stem, name, old, new, mark in rows:
        print(f"| {stem} | {name} | `{old}` | `{new}` | {mark} |")

    # CSV 备份
    csv_path = Path("/tmp/china_sfid_remap.csv")
    with csv_path.open("w", encoding="utf-8") as f:
        f.write("file,institution_name,old_sfid,new_sfid,city_changed\n")
        for stem, name, old, new, mark in rows:
            f.write(f"{stem},{name},{old},{new},{mark.strip() == '→'}\n")
    print(f"\n> CSV: {csv_path}", file=sys.stderr)
    print(
        f"> 统计: 总 {counts['total']} | 市码 000→001 改动 {counts['city_changed']} | 市码保留 {counts['city_kept']}",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
