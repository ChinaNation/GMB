#!/usr/bin/env python3
"""
重新计算 primitives/china 中所有 duoqian_address。

算法（与 duoqian-transaction-pow pallet 完全一致）：
  address = blake2b_256("DUOQIAN_SFID_V1" || ss58_prefix_u16_le || sfid_id_bytes)

SS58 前缀 2027 (0xEB07 小端) 作为链域标识，不同链不同前缀，地址自然不同。

用法：
  # 仅打印，不修改文件（默认）：
  python3 recompute_duoqian_addresses.py

  # 直接修改 .rs 文件：
  python3 recompute_duoqian_addresses.py --apply
"""

import argparse
import hashlib
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


CHINA_DIR = Path(__file__).resolve().parent.parent / "primitives" / "china"
PREFIX = b"DUOQIAN_SFID_V1"
SS58_FORMAT: int = 2027  # 与 primitives::core_const::SS58_FORMAT 一致

# 需要处理的文件
FILES_CONFIG = [
    "china_cb.rs",
    "china_ch.rs",
    "china_sf.rs",
    "china_jc.rs",
    "china_lf.rs",
    "china_jy.rs",
    "china_zf.rs",
]


def blake2b_256(data: bytes) -> bytes:
    """计算 blake2b-256 哈希。"""
    return hashlib.blake2b(data, digest_size=32).digest()


def derive_duoqian_address(sfid_id: str) -> bytes:
    """与 Rust pallet 中 derive_duoqian_address_from_sfid_id 完全一致的派生逻辑。

    preimage = "DUOQIAN_SFID_V1" + ss58_prefix.to_le_bytes() + sfid_id.as_bytes()
    address  = blake2b_256(preimage)
    """
    ss58_le = SS58_FORMAT.to_bytes(2, byteorder="little")
    preimage = PREFIX + ss58_le + sfid_id.encode("utf-8")
    return blake2b_256(preimage)


def bytes_to_hex_literal(b: bytes) -> str:
    """转换为 Rust hex!("...") 内部格式。"""
    return b.hex()


@dataclass
class AddressEntry:
    sfid_id: str
    old_hex: str
    new_hex: str
    file_name: str
    line_num: int


def extract_and_recompute(file_path: Path) -> list[AddressEntry]:
    """从 .rs 文件中提取 shenfen_id 和 duoqian_address，计算新地址。"""
    content = file_path.read_text(encoding="utf-8")
    lines = content.split("\n")

    entries: list[AddressEntry] = []

    # 状态机：找到 shenfen_id 后找下一个 duoqian_address
    current_sfid: Optional[str] = None
    sfid_pattern = re.compile(r'shenfen_id:\s*"([^"]+)"')
    addr_pattern = re.compile(r'duoqian_address:\s*hex!\("([0-9a-fA-F]{64})"\)')

    for i, line in enumerate(lines):
        sfid_match = sfid_pattern.search(line)
        if sfid_match:
            current_sfid = sfid_match.group(1)

        addr_match = addr_pattern.search(line)
        if addr_match and current_sfid is not None:
            old_hex = addr_match.group(1).lower()
            new_address = derive_duoqian_address(current_sfid)
            new_hex = bytes_to_hex_literal(new_address)
            entries.append(AddressEntry(
                sfid_id=current_sfid,
                old_hex=old_hex,
                new_hex=new_hex,
                file_name=file_path.name,
                line_num=i + 1,
            ))
            current_sfid = None  # reset for next entry

    return entries


def apply_changes(file_path: Path, entries: list[AddressEntry]) -> None:
    """将新地址写入 .rs 文件。"""
    content = file_path.read_text(encoding="utf-8")
    for entry in entries:
        old_pattern = f'hex!("{entry.old_hex}")'
        new_pattern = f'hex!("{entry.new_hex}")'
        content = content.replace(old_pattern, new_pattern, 1)
    file_path.write_text(content, encoding="utf-8")


def recompute_zb(
    all_entries: list[AddressEntry], dry_run: bool
) -> None:
    """重新生成 china_zb.rs（汇总所有 duoqian_address 的保留列表）。"""
    zb_path = CHINA_DIR / "china_zb.rs"

    # 收集所有新地址，排序后去重
    all_addresses = sorted(set(e.new_hex for e in all_entries))

    lines = [
        '//! 汇总 primitives/china 目录下所有制度保留 duoqian_address。',
        '//! 用于禁止 duoqian-transaction-pow 抢注这些机构地址。',
        '',
        'use hex_literal::hex;',
        '',
        f'pub const CHINA_RESERVED_DUOQIAN_ADDRESSES: &[[u8; 32]; {len(all_addresses)}] = &[',
    ]
    for addr in all_addresses:
        lines.append(f'    hex!("{addr}"),')
    lines.append('];')
    lines.append('')  # trailing newline

    new_content = '\n'.join(lines)

    if dry_run:
        print(f"\n=== china_zb.rs 将包含 {len(all_addresses)} 个保留地址 ===")
    else:
        zb_path.write_text(new_content, encoding="utf-8")
        print(f"✅ china_zb.rs 已更新：{len(all_addresses)} 个保留地址")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="重新计算 primitives/china 中的 duoqian_address（blake2b-256 + SS58 前缀 2027）"
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument(
        "--dry-run",
        action="store_true",
        default=True,
        help="仅打印变更，不修改文件（默认）",
    )
    group.add_argument(
        "--apply",
        action="store_true",
        help="直接修改 .rs 源文件",
    )
    args = parser.parse_args()

    dry_run = not args.apply
    if dry_run:
        print("🔍 干运行模式（不修改文件）\n")
    else:
        print("✏️  应用模式（将修改文件）\n")

    print(f"SS58 前缀: {SS58_FORMAT} (0x{SS58_FORMAT:04x}, LE: {SS58_FORMAT.to_bytes(2, 'little').hex()})\n")

    all_entries: list[AddressEntry] = []
    changed_count = 0
    unchanged_count = 0

    for fname in FILES_CONFIG:
        fpath = CHINA_DIR / fname
        if not fpath.exists():
            print(f"⚠️  跳过不存在的文件: {fname}")
            continue

        entries = extract_and_recompute(fpath)
        all_entries.extend(entries)

        file_changed = [e for e in entries if e.old_hex != e.new_hex]
        file_unchanged = [e for e in entries if e.old_hex == e.new_hex]
        changed_count += len(file_changed)
        unchanged_count += len(file_unchanged)

        print(f"📄 {fname}: {len(entries)} 个地址（{len(file_changed)} 变更，{len(file_unchanged)} 不变）")

        for e in entries:
            marker = "  ✅" if e.old_hex == e.new_hex else "  🔄"
            print(f"{marker} {e.sfid_id}")
            if e.old_hex != e.new_hex:
                print(f"     旧: {e.old_hex}")
                print(f"     新: {e.new_hex}")

        if not dry_run and file_changed:
            apply_changes(fpath, entries)
            print(f"  ✅ 文件已更新")

    print(f"\n{'='*60}")
    print(f"总计: {len(all_entries)} 个地址，{changed_count} 变更，{unchanged_count} 不变")

    if not dry_run:
        recompute_zb(all_entries, dry_run=False)
    else:
        recompute_zb(all_entries, dry_run=True)

    if dry_run and changed_count > 0:
        print(f"\n💡 使用 --apply 参数来应用变更")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
