#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""扫 memory/ 全部 .md 做 SFID 改造收尾,排除 done/(保留历史真实)。

替换规则:
  1. shenfen_id / sfid_id → sfid_number  (snake)
  2. shenfenId / sfidId → sfidNumber       (camel)
  3. ShenfenId / SfidId → SfidNumber       (Pascal)
  4. SHENFEN_ID / SFID_ID → SFID_NUMBER    (SCREAM)
  5. shenfen_name → sfid_name              (snake)
  6. shenfenName → sfidName                (camel)
  7. ShenfenName → SfidName                (Pascal)
  8. SHENFEN_NAME → SFID_NAME              (SCREAM)
  9. 老 NRC SFID `GFR-LN001-CB0C-617776487-20260222` → 新 NRC `GFR-LN001-CB0X-944805165-2026`
  10. `D8(8)` / `- D8` → `D4(4)` / `- D4`
  11. `YYYYMMDD` → `YYYY`
  12. 占位 `N9-D8` → `N9-D4`(诸如 `SFR-GD-SZ01-CB01-N9-D8` 之类)

排除:
  - memory/08-tasks/done/(历史任务卡)
  - memory/06-quality/(对照表,字面量需保留)
  - memory/MEMORY.md(由本文件处理)→ 也包含进去,只改字段名,不改字面量
"""

import re
from pathlib import Path

ROOT = Path("/Users/rhett/GMB/memory")
EXCLUDE_DIRS = ["08-tasks/done", "06-quality"]

# 顺序敏感:先长后短
RENAMES = [
    # 老 NRC SFID 字面量 → 新 NRC
    ("GFR-LN001-CB0C-617776487-20260222", "GFR-LN001-CB0X-944805165-2026"),
    # SCREAMING
    ("SHENFEN_ID", "SFID_NUMBER"),
    ("SHENFEN_NAME", "SFID_NAME"),
    ("SFID_ID", "SFID_NUMBER"),
    # PascalCase
    ("ShenfenId", "SfidNumber"),
    ("ShenfenName", "SfidName"),
    ("SfidId", "SfidNumber"),
    # camelCase
    ("shenfenId", "sfidNumber"),
    ("shenfenName", "sfidName"),
    ("sfidId", "sfidNumber"),
    # snake_case
    ("shenfen_id", "sfid_number"),
    ("shenfen_name", "sfid_name"),
    ("sfid_id", "sfid_number"),
    # 格式段名
    ("D8(8)", "D4(4)"),
    ("`D8`", "`D4`"),
    ("- D8", "- D4"),
    ("N9-D8", "N9-D4"),
    ("YYYYMMDD", "YYYY"),
]


def should_scan(p: Path) -> bool:
    if p.suffix != ".md":
        return False
    rel = p.relative_to(ROOT).as_posix()
    for ex in EXCLUDE_DIRS:
        if rel.startswith(ex):
            return False
    return True


def main():
    files = [p for p in ROOT.rglob("*.md") if p.is_file() and should_scan(p)]
    print(f"扫描 memory/ 下 {len(files)} 个 .md(排除 done/, 06-quality/)")

    changed = 0
    by_rule = {}
    for fp in files:
        try:
            old = fp.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        new = old
        hits = 0
        for src, dst in RENAMES:
            if src in new:
                cnt = new.count(src)
                new = new.replace(src, dst)
                hits += cnt
                by_rule[src] = by_rule.get(src, 0) + cnt
        if new != old:
            fp.write_text(new, encoding="utf-8")
            changed += 1

    print(f"\n改动 {changed} 个 .md")
    print("按规则分布:")
    for src, cnt in sorted(by_rule.items(), key=lambda x: -x[1]):
        print(f"  {src!r}: {cnt} 次")


if __name__ == "__main__":
    main()
