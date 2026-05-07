#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Big-bang 全仓库改名 + 277 条 sfid_number 字面量替换。

改名规则(顺序敏感):
  1. sfid_number → sfid_number     (snake_case)
  2. sfid_number → sfid_number        (snake_case,统一)
  3. SFID_NUMBER → SFID_NUMBER     (SCREAMING)
  4. SFID_NUMBER → SFID_NUMBER        (SCREAMING,统一)
  5. SfidNumber → SfidNumber       (PascalCase,类型名)
  6. SfidNumber → SfidNumber          (PascalCase,统一)
  7. sfidNumber → sfidNumber       (camelCase)
  8. sfidNumber → sfidNumber          (camelCase,统一)
  9. sfid_name → sfid_name
  10. SFID_NAME → SFID_NAME
  11. SfidName → SfidName
  12. sfidName → sfidName
  13. 277 条 SFID 字面量按 CSV 映射 1:1 替换

扫描范围:
  - 根:citizenchain / sfid / wuminapp / wumin (citizenchain/node/frontend 为 TS 一并扫)
  - 后缀:.rs .dart .ts .tsx .md .json .toml .py
  - 排除:target/ node_modules/ dist/ build/ .dart_tool/ .lock 文件
"""

import csv
from pathlib import Path

ROOT = Path("/Users/rhett/GMB")
SCAN_ROOTS = [
    ROOT / "citizenchain",
    ROOT / "sfid",
    ROOT / "wuminapp",
    ROOT / "wumin",
    ROOT / "tools",  # 2026-05-07 补:tools/duoqian.py 等脚本也要统一改名
]
EXTS = {".rs", ".dart", ".ts", ".tsx", ".md", ".json", ".toml", ".py"}
EXCLUDE_DIR_PARTS = {
    "target",
    "node_modules",
    "dist",
    "build",
    ".dart_tool",
    ".gradle",
    ".next",
    ".venv",
    "__pycache__",
}
EXCLUDE_FILES = {
    "Cargo.lock",
    "pubspec.lock",
    "package-lock.json",
    "yarn.lock",
}

# 改名规则(text 替换,plain string,顺序敏感不重要因互不嵌套)
RENAMES = [
    ("sfid_number", "sfid_number"),
    ("sfid_number", "sfid_number"),
    ("SFID_NUMBER", "SFID_NUMBER"),
    ("SFID_NUMBER", "SFID_NUMBER"),
    ("SfidNumber", "SfidNumber"),
    ("SfidNumber", "SfidNumber"),
    ("sfidNumber", "sfidNumber"),
    ("sfidNumber", "sfidNumber"),
    ("sfid_name", "sfid_name"),
    ("SFID_NAME", "SFID_NAME"),
    ("SfidName", "SfidName"),
    ("sfidName", "sfidName"),
]


def load_sfid_mapping(csv_path: Path) -> list[tuple[str, str]]:
    """读 CSV 提取 (old_sfid, new_sfid) 对,按 old_sfid 长度倒序排列防止前缀冲突。"""
    pairs = []
    with csv_path.open() as f:
        reader = csv.DictReader(f)
        for row in reader:
            old = row["old_sfid"].strip()
            new = row["new_sfid"].strip()
            if old and new and old != new:
                pairs.append((old, new))
    # 按 old 长度倒序,确保长串先替换避免被短串吞
    pairs.sort(key=lambda p: -len(p[0]))
    return pairs


def should_scan(p: Path) -> bool:
    if p.name in EXCLUDE_FILES:
        return False
    if p.suffix not in EXTS:
        return False
    parts = set(p.parts)
    if parts & EXCLUDE_DIR_PARTS:
        return False
    return True


def collect_files() -> list[Path]:
    all_files = []
    for root in SCAN_ROOTS:
        for p in root.rglob("*"):
            if p.is_file() and should_scan(p):
                all_files.append(p)
    return all_files


def apply_replacements(text: str, replacements: list[tuple[str, str]]) -> tuple[str, int]:
    total_hits = 0
    for old, new in replacements:
        if old in text:
            count = text.count(old)
            text = text.replace(old, new)
            total_hits += count
    return text, total_hits


def main():
    sfid_pairs = load_sfid_mapping(Path("/tmp/china_sfid_remap.csv"))
    all_replacements = RENAMES + sfid_pairs
    print(f"加载改名规则 {len(RENAMES)} 条 + SFID 映射 {len(sfid_pairs)} 条 = 共 {len(all_replacements)} 条")

    files = collect_files()
    print(f"扫描 {len(files)} 个文件…")

    changed_files = 0
    total_hits = 0
    by_ext: dict[str, int] = {}
    for fp in files:
        try:
            old_text = fp.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        new_text, hits = apply_replacements(old_text, all_replacements)
        if new_text != old_text:
            fp.write_text(new_text, encoding="utf-8")
            changed_files += 1
            total_hits += hits
            ext = fp.suffix
            by_ext[ext] = by_ext.get(ext, 0) + 1

    print(f"\n改动文件: {changed_files} | 总替换次数: {total_hits}")
    print("按后缀分布:")
    for ext, cnt in sorted(by_ext.items(), key=lambda x: -x[1]):
        print(f"  {ext}: {cnt} 文件")


if __name__ == "__main__":
    main()
