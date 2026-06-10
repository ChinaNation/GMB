#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
镇级行政区命名统一脚本（省 / 市 / 镇 三级规整）。

背景：china.sqlite 的 towns 表里同一层级混着「镇 / 街道 / 乡 / 开发区 / 林场 /
农场 / 苏木 / 管委会 …」十几种叫法。本脚本把镇一级所有非「镇」名字改写为「X镇」。

设计：
- 单一有序后缀表 SUFFIX_RULES（按长度优先匹配），每条标 tier:
    auto    —— 规整行政单位，自动去尾加「镇」（街道/乡/开发区/园区/林场/农场/苏木…）
    confirm —— 语义模糊或非居住单位（管委会/监狱/水库/矿务局/岛/港/度假区…），
               需人工在清单上逐条确认后再转。
- 幂等：名字已以「镇」结尾 → 跳过。
- town_code = (province_code, city_code, code) 是唯一键；改名后即使同名也不破坏数据。

用法：
    python3 normalize_towns.py --scan
        只读扫描，输出 towns_longtail_confirm.csv（长尾需确认清单）
        与 towns_auto_preview.csv（自动转预览，供审计），并打印统计。
    python3 normalize_towns.py --apply towns_longtail_confirm.csv
        套用 auto 规则 + 清单里 final_name 列，写回 china.sqlite；输出同名碰撞报告。
"""
import argparse
import csv
import os
import sqlite3
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
DB_PATH = os.path.join(HERE, "china.sqlite")
CONFIRM_CSV = os.path.join(HERE, "towns_longtail_confirm.csv")
AUTO_CSV = os.path.join(HERE, "towns_auto_preview.csv")

ZHEN = "镇"

# 有序后缀表：必须「长在前、短在后」，按顺序首次命中即用（最长后缀优先）。
# tier=auto 自动转；tier=confirm 进确认清单。
SUFFIX_RULES = [
    # ---- auto：规整行政单位 ----
    ("经济技术开发区", "auto"),
    ("高新技术产业开发区", "auto"),
    ("高新技术产业园区", "auto"),
    ("高新技术开发区", "auto"),
    ("经济开发区", "auto"),
    ("高新区", "auto"),
    ("工业园区", "auto"),
    ("产业园区", "auto"),
    ("科技园区", "auto"),
    ("工业园", "auto"),
    ("产业园", "auto"),
    ("科技园", "auto"),
    ("街道办事处", "auto"),
    ("街道", "auto"),
    ("开发区", "auto"),
    ("管理区", "auto"),
    ("园区", "auto"),
    ("苏木", "auto"),          # 蒙古族乡级单位，等价镇
    ("林场", "auto"),
    ("农场", "auto"),
    ("乡", "auto"),            # 含民族乡：默认只换尾字「乡」，保留「回族」等描述
    # ---- confirm：语义模糊 / 非居住单位，需人工确认 ----
    ("管理委员会", "confirm"),
    ("管委会", "confirm"),
    ("委员会", "confirm"),
    ("工程管理局", "confirm"),
    ("管理局", "confirm"),
    ("管理处", "confirm"),
    ("办事处", "confirm"),
    ("旅游度假区", "confirm"),
    ("生态旅游度假区", "confirm"),
    ("度假区", "confirm"),
    ("风景区", "confirm"),
    ("示范区", "confirm"),
    ("试验区", "confirm"),
    ("生态区", "confirm"),
    ("生态城", "confirm"),
    ("综合保税区", "confirm"),
    ("保税港区", "confirm"),
    ("保税区", "confirm"),
    ("出口加工区", "confirm"),
    ("加工区", "confirm"),
    ("地区", "confirm"),
    ("林业局", "confirm"),
    ("林区", "confirm"),
    ("矿务局", "confirm"),
    ("煤矿", "confirm"),
    ("矿区", "confirm"),
    ("水库", "confirm"),
    ("灌区", "confirm"),
    ("监狱", "confirm"),
    ("种畜场", "confirm"),
    ("原种场", "confirm"),
    ("牧场", "confirm"),
    ("渔场", "confirm"),
    ("茶场", "confirm"),
    ("盐场", "confirm"),
    ("半岛", "confirm"),
    ("群岛", "confirm"),
    ("兵团", "confirm"),
    ("苗圃", "confirm"),
    ("基地", "confirm"),
    ("中心", "confirm"),
    ("新村", "confirm"),
    ("新区", "confirm"),
    ("社区", "confirm"),
    # 兜底单字（必须放最后，避免吃掉上面的长后缀）
    ("港", "confirm"),
    ("岛", "confirm"),
    ("团", "confirm"),
    ("城", "confirm"),
    ("区", "confirm"),
]


# 用户决策(2026-06-08):长尾「只转干净的」—— 仅下列类目去尾加镇,其余(非居住
# 功能单位 + 无后缀杂项)一律保留原名。退化名(名=后缀)与基底不足 2 字者亦保留。
CLEAN_CONFIRM_CATEGORIES = {
    "地区", "牧场", "种畜场", "原种场", "渔场", "茶场", "盐场", "新区", "社区",
}


def decide_final(category, old, proposed):
    """长尾最终名:仅干净类目且非退化、基底≥2字 → 转镇;否则保留原名。"""
    if category not in CLEAN_CONFIRM_CATEGORIES:
        return old
    if proposed == old:
        return old  # 退化:名字本身就是后缀(如就叫「原种场」)
    base = proposed[: -len(ZHEN)]  # 去掉末尾「镇」得基底
    if len(base) < 2:
        return old  # 基底太短(县镇/市镇/省镇之类),保留原名
    return proposed


def match_suffix(name):
    """返回 (suffix, tier) 或 None。最长后缀优先（表已按长在前排序）。"""
    for suffix, tier in SUFFIX_RULES:
        if name.endswith(suffix):
            return suffix, tier
    return None


def proposed_name(name, suffix):
    """去掉 suffix 末尾后接「镇」。base 为空则返回 None（异常，保留原名待人工）。"""
    base = name[: -len(suffix)]
    if not base:
        return None
    return base + ZHEN


def classify(name):
    """
    返回 (tier, proposed)：
      tier='noop'    已是镇，跳过
      tier='auto'    自动转，proposed 为新名
      tier='confirm' 进确认清单，proposed 为最佳猜测新名（可能=原名）
      tier='nomatch' 无可识别后缀，proposed=原名，进确认清单
    """
    if name.endswith(ZHEN):
        return "noop", name
    m = match_suffix(name)
    if m is None:
        return "nomatch", name
    suffix, tier = m
    prop = proposed_name(name, suffix)
    if prop is None:
        # 名字本身就等于后缀（如就叫「开发区」），无法去尾 → 当作需确认，保留原名
        return "confirm", name
    return tier, prop


def load_towns(conn):
    cur = conn.execute(
        "SELECT province_code, city_code, code, name FROM towns ORDER BY province_code, city_code, code"
    )
    return cur.fetchall()


def scan(conn):
    rows = load_towns(conn)
    auto_rows = []       # (pc, cc, code, old, new)
    confirm_rows = []    # (pc, cc, code, category, old, proposed)
    stats = defaultdict(int)
    cat_count = defaultdict(int)

    for pc, cc, code, name in rows:
        tier, prop = classify(name)
        stats[tier] += 1
        if tier == "auto":
            auto_rows.append((pc, cc, code, name, prop))
        elif tier in ("confirm", "nomatch"):
            m = match_suffix(name)
            category = m[0] if (m and tier == "confirm") else ("无后缀杂项" if tier == "nomatch" else m[0])
            cat_count[category] += 1
            confirm_rows.append((pc, cc, code, category, name, prop))

    # --- 写自动转预览 CSV（审计用）---
    with open(AUTO_CSV, "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f)
        w.writerow(["province_code", "city_code", "code", "old_name", "new_name"])
        w.writerows(auto_rows)

    # --- 写长尾确认 CSV ---
    # final_name 列预填 proposed，用户可改；若要保留原名，把 final_name 改回原名即可。
    confirm_rows.sort(key=lambda r: (r[3], r[4]))
    confirm_convert = 0
    confirm_keep = 0
    with open(CONFIRM_CSV, "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f)
        w.writerow(
            ["province_code", "city_code", "code", "category", "old_name", "proposed_name", "final_name"]
        )
        for pc, cc, code, category, old, prop in confirm_rows:
            final = decide_final(category, old, prop)
            if final != old:
                confirm_convert += 1
            else:
                confirm_keep += 1
            w.writerow([pc, cc, code, category, old, prop, final])

    # --- 同名碰撞预演（auto + confirm 的 proposed 合在一起，按 省市 分组）---
    name_by_city = defaultdict(lambda: defaultdict(list))
    for pc, cc, code, old, new in auto_rows:
        name_by_city[(pc, cc)][new].append(("auto", code, old))
    for pc, cc, code, category, old, prop in confirm_rows:
        name_by_city[(pc, cc)][prop].append(("confirm", code, old))
    # 还要算上「已是镇」的现有名，否则会漏掉 X街道→X镇 撞上已有 X镇
    for pc, cc, code, name in rows:
        if name.endswith(ZHEN):
            name_by_city[(pc, cc)][name].append(("noop", code, name))
    collisions = 0
    collision_samples = []
    for (pc, cc), names in name_by_city.items():
        for new, members in names.items():
            if len(members) > 1:
                collisions += 1
                if len(collision_samples) < 15:
                    collision_samples.append((pc, cc, new, members))

    # --- 打印统计 ---
    total = len(rows)
    print("=" * 60)
    print(f"towns 总数: {total}")
    print(f"  已是镇 (noop)      : {stats['noop']}")
    print(f"  自动转 (auto)      : {stats['auto']}  -> {AUTO_CSV}")
    print(f"  需确认 (confirm)   : {stats['confirm']}")
    print(f"  无后缀 (nomatch)   : {stats['nomatch']}")
    print(f"  确认清单合计        : {stats['confirm'] + stats['nomatch']}  -> {CONFIRM_CSV}")
    print(f"    其中 干净类目转镇 : {confirm_convert}")
    print(f"    其中 保留原名     : {confirm_keep}")
    total_to_zhen = stats['auto'] + confirm_convert
    print(f"  >>> 最终会改名为「镇」总数 = auto {stats['auto']} + 长尾 {confirm_convert} = {total_to_zhen}")
    print(f"  >>> 改名后仍非「镇」的条数(=长尾保留) = {confirm_keep}")
    print("=" * 60)
    print("确认清单类目分布（行数）:")
    for cat, n in sorted(cat_count.items(), key=lambda x: -x[1]):
        print(f"  {cat:<12} {n}")
    print("=" * 60)
    print(f"同名碰撞组数（同一省市内 proposed 撞名，含撞上已有镇）: {collisions}")
    print("  注: town_code 是唯一键，同名仅展示层面，不破坏数据。样例:")
    for pc, cc, new, members in collision_samples:
        olds = ", ".join(f"{old}({code})" for _, code, old in members)
        print(f"  [{pc}-{cc}] {new}  <=  {olds}")
    print("=" * 60)


def apply_changes(conn, confirmed_csv):
    rows = load_towns(conn)
    # 读确认清单：以 (pc,cc,code) -> final_name
    confirm_map = {}
    with open(confirmed_csv, "r", encoding="utf-8-sig") as f:
        r = csv.DictReader(f)
        for row in r:
            key = (row["province_code"], row["city_code"], row["code"])
            confirm_map[key] = row["final_name"].strip()

    updates = []  # (new_name, pc, cc, code)
    kept = 0
    for pc, cc, code, name in rows:
        tier, prop = classify(name)
        if tier == "noop":
            continue
        if tier == "auto":
            new = prop
        else:  # confirm / nomatch
            new = confirm_map.get((pc, cc, code))
            if new is None:
                print(f"  [警告] 确认清单缺失: {pc}-{cc}-{code} {name} -> 保留原名")
                new = name
        if new == name:
            kept += 1
            continue
        updates.append((new, pc, cc, code))

    conn.executemany(
        "UPDATE towns SET name = ? WHERE province_code = ? AND city_code = ? AND code = ?",
        updates,
    )
    conn.commit()

    # 应用后碰撞报告
    after = conn.execute(
        "SELECT province_code, city_code, name, COUNT(*) c FROM towns "
        "GROUP BY province_code, city_code, name HAVING c > 1 ORDER BY c DESC"
    ).fetchall()
    non_zhen = conn.execute(
        "SELECT COUNT(*) FROM towns WHERE name NOT LIKE '%镇'"
    ).fetchone()[0]

    print("=" * 60)
    print(f"已写回 {len(updates)} 条改名；保留原名 {kept} 条。")
    print(f"改名后仍非「镇」结尾的条数: {non_zhen}（应等于你在清单里显式保留的条数）")
    print(f"同名碰撞组数（省市内同名）: {len(after)}")
    for pc, cc, name, c in after[:20]:
        print(f"  [{pc}-{cc}] {name} x{c}")
    print("=" * 60)


def main():
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--scan", action="store_true", help="只读扫描，生成确认清单")
    g.add_argument("--apply", metavar="CONFIRMED_CSV", help="套规则+清单写回 sqlite")
    args = ap.parse_args()

    if not os.path.exists(DB_PATH):
        print(f"找不到 {DB_PATH}", file=sys.stderr)
        sys.exit(1)

    conn = sqlite3.connect(DB_PATH)
    try:
        if args.scan:
            scan(conn)
        else:
            apply_changes(conn, args.apply)
    finally:
        conn.close()


if __name__ == "__main__":
    main()
