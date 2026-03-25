#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
从 /Users/rhett/Documents 下的 china_*.txt 提取“公钥:”行，
按顺序回填到 citizenchain/runtime/primitives/china/*.rs 的 duoqian_admins 字段。

默认 dry-run，不改文件；加 --apply 才真正写入。
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


DOCS_DIR = Path("/Users/rhett/Documents")
GMB_DIR = Path("/Users/rhett/GMB")
CHINA_DIR = GMB_DIR / "citizenchain" / "runtime" / "primitives" / "china"


@dataclass(frozen=True)
class FileConfig:
    name: str
    txt_path: Path
    rs_path: Path
    struct_name: str
    field_name: str
    admins_per_institution: int


FILES = [
    FileConfig(
        name="china_zf",
        txt_path=DOCS_DIR / "china_zf.txt",
        rs_path=CHINA_DIR / "china_zf.rs",
        struct_name="ChinaZf",
        field_name="duoqian_admins",
        admins_per_institution=9,
    ),
    FileConfig(
        name="china_lf",
        txt_path=DOCS_DIR / "china_lf.txt",
        rs_path=CHINA_DIR / "china_lf.rs",
        struct_name="ChinaLf",
        field_name="duoqian_admins",
        admins_per_institution=9,
    ),
    FileConfig(
        name="china_jc",
        txt_path=DOCS_DIR / "china_jc.txt",
        rs_path=CHINA_DIR / "china_jc.rs",
        struct_name="ChinaJc",
        field_name="duoqian_admins",
        admins_per_institution=9,
    ),
    FileConfig(
        name="china_sf",
        txt_path=DOCS_DIR / "china_sf.txt",
        rs_path=CHINA_DIR / "china_sf.rs",
        struct_name="ChinaSf",
        field_name="duoqian_admins",
        admins_per_institution=9,
    ),
]


PUBKEY_LINE_RE = re.compile(r"公钥:\s*(?:0x)?([0-9a-fA-F]{64})")


def find_duplicates(items: list[str]) -> list[str]:
    """返回重复公钥列表，便于提前阻断错误输入。"""
    seen: set[str] = set()
    duplicates: list[str] = []
    for item in items:
        if item in seen and item not in duplicates:
            duplicates.append(item)
        seen.add(item)
    return duplicates


def extract_pubkeys(txt_path: Path) -> list[str]:
    """只提取每个 txt 中“公钥:”行后的 32 字节十六进制公钥。"""
    if not txt_path.exists():
        raise FileNotFoundError(f"找不到输入文件: {txt_path}")

    text = txt_path.read_text(encoding="utf-8")
    pubkeys = [match.group(1).lower() for match in PUBKEY_LINE_RE.finditer(text)]

    if not pubkeys:
        raise ValueError(f"{txt_path} 中没有提取到任何“公钥:”行")

    duplicates = find_duplicates(pubkeys)
    if duplicates:
        sample = ", ".join(duplicates[:5])
        raise ValueError(f"{txt_path} 中存在重复公钥，示例: {sample}")

    return pubkeys


def count_struct_entries(rs_text: str, struct_name: str) -> int:
    """统计 .rs 文件中指定机构结构体条目数量。"""
    pattern = re.compile(rf"^\s*{struct_name}\s*\{{\s*$", re.MULTILINE)
    return len(pattern.findall(rs_text))


def group_pubkeys(pubkeys: list[str], group_size: int) -> list[list[str]]:
    """按机构管理员数量固定分组。"""
    if len(pubkeys) % group_size != 0:
        raise ValueError(f"公钥数量 {len(pubkeys)} 不能被每组数量 {group_size} 整除")
    return [pubkeys[i : i + group_size] for i in range(0, len(pubkeys), group_size)]


def build_admin_block(pubkeys: list[str], indent: str, field_name: str) -> str:
    """构建 Rust 数组字面量，保持原字段缩进。"""
    lines = [f"{indent}{field_name}: &["]
    for key in pubkeys:
        lines.append(f'{indent}    hex!("{key}"),')
    lines.append(f"{indent}],")
    return "\n".join(lines)


def replace_admin_field_in_block(block_text: str, field_name: str, pubkeys: list[str]) -> str:
    """只替换单个机构块中的管理员字段，避免全局误替换。"""
    field_re = re.compile(
        rf"^(\s*){field_name}:\s*(?:EMPTY_DUOQIAN_ADMINS|&\[[\s\S]*?\]),\s*$",
        re.MULTILINE,
    )
    match = field_re.search(block_text)
    if not match:
        raise ValueError(f"结构块中未找到可替换的 {field_name} 字段")

    indent = match.group(1)
    replacement = build_admin_block(pubkeys, indent, field_name)
    return field_re.sub(replacement, block_text, count=1)


def replace_struct_blocks(rs_text: str, cfg: FileConfig, grouped_pubkeys: list[list[str]]) -> tuple[str, int]:
    """按结构体块顺序逐个替换管理员列表。"""
    lines = rs_text.splitlines()
    output_lines: list[str] = []

    struct_start_re = re.compile(rf"^\s*{cfg.struct_name}\s*\{{\s*$")
    inside_struct = False
    brace_depth = 0
    current_block: list[str] = []
    replaced_count = 0
    group_index = 0

    for line in lines:
        if not inside_struct and struct_start_re.match(line):
            inside_struct = True
            brace_depth = line.count("{") - line.count("}")
            current_block = [line]
            continue

        if inside_struct:
            current_block.append(line)
            brace_depth += line.count("{") - line.count("}")
            if brace_depth == 0:
                if group_index >= len(grouped_pubkeys):
                    raise ValueError(f"{cfg.name} 分组数量不足，无法继续替换")

                block_text = "\n".join(current_block)
                new_block = replace_admin_field_in_block(
                    block_text,
                    cfg.field_name,
                    grouped_pubkeys[group_index],
                )
                output_lines.extend(new_block.splitlines())
                replaced_count += 1
                group_index += 1
                inside_struct = False
                current_block = []
            continue

        output_lines.append(line)

    if inside_struct:
        raise ValueError(f"{cfg.rs_path} 结构块解析未正常结束")

    if group_index != len(grouped_pubkeys):
        raise ValueError(
            f"{cfg.name} 替换后分组未用完: 已使用 {group_index}, 总分组 {len(grouped_pubkeys)}"
        )

    return "\n".join(output_lines) + "\n", replaced_count


def process_file(cfg: FileConfig, apply_changes: bool) -> None:
    """处理单个 china_*.txt 与 china_*.rs 对。"""
    pubkeys = extract_pubkeys(cfg.txt_path)
    rs_text = cfg.rs_path.read_text(encoding="utf-8")

    institution_count = count_struct_entries(rs_text, cfg.struct_name)
    if institution_count == 0:
        raise ValueError(f"{cfg.rs_path} 中未找到任何 {cfg.struct_name} 条目")

    expected_count = institution_count * cfg.admins_per_institution
    actual_count = len(pubkeys)
    if actual_count != expected_count:
        raise ValueError(
            f"{cfg.name} 公钥数量不匹配: 机构数 {institution_count} × 每机构 "
            f"{cfg.admins_per_institution} = {expected_count}，但 txt 实际提取到 {actual_count}"
        )

    grouped_pubkeys = group_pubkeys(pubkeys, cfg.admins_per_institution)
    new_text, replaced_count = replace_struct_blocks(rs_text, cfg, grouped_pubkeys)

    if replaced_count != institution_count:
        raise ValueError(
            f"{cfg.name} 替换数量不匹配: 预期 {institution_count}，实际 {replaced_count}"
        )

    print(
        f"[OK] {cfg.name}: 提取 {actual_count} 个公钥，"
        f"{institution_count} 个机构，每机构 {cfg.admins_per_institution} 个"
    )

    if apply_changes:
        cfg.rs_path.write_text(new_text, encoding="utf-8")
        print(f"[WRITE] 已写入 {cfg.rs_path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="批量回填 china 机构管理员公钥")
    parser.add_argument("--apply", action="store_true", help="真正写入 .rs 文件")
    parser.add_argument(
        "--only",
        choices=[cfg.name for cfg in FILES],
        help="只处理单个文件，例如 china_zf",
    )
    args = parser.parse_args()

    selected = [cfg for cfg in FILES if args.only in (None, cfg.name)]

    if args.apply:
        print("当前为 apply 模式，将修改目标 .rs 文件。")
    else:
        print("当前为 dry-run 模式，不会修改文件。")

    try:
        for cfg in selected:
            process_file(cfg, apply_changes=args.apply)
    except Exception as exc:  # noqa: BLE001
        print(f"[ERROR] {exc}", file=sys.stderr)
        return 1

    print("完成。")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
