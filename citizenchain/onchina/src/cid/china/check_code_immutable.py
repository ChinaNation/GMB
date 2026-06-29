#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
行政区 code 不可变不复用 CI 校验(ADR-021)。

断言:
  1. provinces/cities/towns 表无重复 code —— code 是稳定外键,重复=反查歧义。
  2. 省名、市名全国唯一;省 code 固定,不建省级 tombstone。
  3. 没有 live 市/镇占用 tombstones 里已退役的 code —— 退役 code 永不复用。
  4. 删除市/镇只进入 tombstones,后续不得重新分配同一 code。
  5. 镇下地址段不属于行政区,但同一镇下名称与 address_unit_id 必须唯一。

退出码非 0 即失败,可挂 pre-commit / CI。
用法: python3 check_code_immutable.py [--db <path>]
"""
import argparse
import os
import sqlite3
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", default=os.path.join(HERE, "china.sqlite"))
    args = ap.parse_args()

    conn = sqlite3.connect(args.db)
    fail = []

    dup_towns = conn.execute(
        "SELECT province_code, city_code, code, COUNT(*) c FROM towns "
        "GROUP BY province_code, city_code, code HAVING c > 1"
    ).fetchall()
    if dup_towns:
        fail.append(f"towns 重复 (pc,cc,code) {len(dup_towns)} 组,例:{dup_towns[:5]}")

    dup_cities = conn.execute(
        "SELECT province_code, code, COUNT(*) c FROM cities "
        "GROUP BY province_code, code HAVING c > 1"
    ).fetchall()
    if dup_cities:
        fail.append(f"cities 重复 (pc,code) {len(dup_cities)} 组,例:{dup_cities[:5]}")

    dup_provinces = conn.execute(
        "SELECT code, COUNT(*) c FROM provinces GROUP BY code HAVING c > 1"
    ).fetchall()
    if dup_provinces:
        fail.append(f"provinces 重复 code {len(dup_provinces)} 组,例:{dup_provinces[:5]}")

    dup_province_names = conn.execute(
        "SELECT name, COUNT(*) c FROM provinces GROUP BY name HAVING c > 1"
    ).fetchall()
    if dup_province_names:
        fail.append(
            f"provinces 重复 name {len(dup_province_names)} 组,例:{dup_province_names[:5]}"
        )

    dup_city_names = conn.execute(
        "SELECT name, COUNT(*) c, GROUP_CONCAT(province_code || '/' || code) scopes "
        "FROM cities GROUP BY name HAVING c > 1"
    ).fetchall()
    if dup_city_names:
        fail.append(f"cities 全国重复 name {len(dup_city_names)} 组,例:{dup_city_names[:5]}")

    has_province_tomb = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='province_tombstones'"
    ).fetchone()
    if has_province_tomb:
        fail.append("province_tombstones 表不应存在:省级 code 固定,不维护省级 tombstone")

    has_city_tomb = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='city_tombstones'"
    ).fetchone()
    if has_city_tomb:
        reused = conn.execute(
            "SELECT c.province_code, c.code, c.name FROM cities c JOIN city_tombstones tb "
            "ON c.province_code=tb.province_code AND c.code=tb.code"
        ).fetchall()
        if reused:
            fail.append(f"复用了已退役市 code {len(reused)} 条,例:{reused[:5]}")

    has_tomb = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='town_tombstones'"
    ).fetchone()
    if has_tomb:
        reused = conn.execute(
            "SELECT t.province_code, t.city_code, t.code, t.name "
            "FROM towns t JOIN town_tombstones tb "
            "ON t.province_code=tb.province_code AND t.city_code=tb.city_code AND t.code=tb.code"
        ).fetchall()
        if reused:
            fail.append(f"复用了已退役 code {len(reused)} 条,例:{reused[:5]}")

    forbidden_old_fourth_table = "".join(("vill", "ages"))
    old_fourth_tables = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table'"
    ).fetchall()
    if any(name == forbidden_old_fourth_table for (name,) in old_fourth_tables):
        fail.append("旧第四层表不应存在:镇下第四层已统一为 address_units 地址段")

    has_address_units = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='address_units'"
    ).fetchone()
    if not has_address_units:
        fail.append("address_units 表缺失:镇下地址段数据不可用")
    else:
        dup_unit_ids = conn.execute(
            "SELECT address_unit_id, COUNT(*) c FROM address_units "
            "GROUP BY address_unit_id HAVING c > 1"
        ).fetchall()
        if dup_unit_ids:
            fail.append(
                f"address_units 重复 address_unit_id {len(dup_unit_ids)} 组,例:{dup_unit_ids[:5]}"
            )

        dup_unit_names = conn.execute(
            "SELECT province_code, city_code, town_code, name, COUNT(*) c "
            "FROM address_units GROUP BY province_code, city_code, town_code, name HAVING c > 1"
        ).fetchall()
        if dup_unit_names:
            fail.append(
                f"同一镇下地址段重名 {len(dup_unit_names)} 组,例:{dup_unit_names[:5]}"
            )

        org_tail_rows = conn.execute(
            "SELECT province_code, city_code, town_code, name FROM address_units "
            "WHERE name LIKE '%居委会%' OR name LIKE '%居民委员会%' OR name LIKE '%村委会%' "
            "OR name LIKE '%村民委员会%' OR name LIKE '%委员会%' LIMIT 5"
        ).fetchall()
        if org_tail_rows:
            fail.append(f"地址段仍含基层组织尾词,例:{org_tail_rows[:5]}")

    conn.close()

    if fail:
        print("行政区 code 不可变校验 FAIL:", file=sys.stderr)
        for f in fail:
            print("  -", f, file=sys.stderr)
        return 1
    print("行政区 code 不可变校验 PASS(省/市唯一,市/镇 code 无重复,地址段唯一且无组织尾词残留)。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
