#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
行政区 code 不可变不复用 CI 校验(ADR-021)。

断言:
  1. towns 表无重复 (province_code, city_code, code) —— code 是稳定外键,重复=反查歧义。
  2. 没有 live 镇占用 town_tombstones 里已退役的 code —— 退役 code 永不复用。
  3. cities/provinces 同理无重复 (上级 code)。

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

    conn.close()

    if fail:
        print("行政区 code 不可变校验 FAIL:", file=sys.stderr)
        for f in fail:
            print("  -", f, file=sys.stderr)
        return 1
    print("行政区 code 不可变校验 PASS(towns/cities 无重复、无复用退役 code)。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
