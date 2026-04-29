#!/usr/bin/env python3
"""
统一派生 citizenchain/runtime/primitives/china 下所有链上保留地址。

统一方案（DUOQIAN_V1 单域 + op_tag 子命名空间）：

    preimage = b"DUOQIAN_V1"  (10B)
             || op_tag        (1B)
             || ss58_prefix   (2B, LE, = [0xEB, 0x07] for 2027)
             || payload       (按 op_tag 规范拼接)
    address  = blake2b_256(preimage)

op_tag 分配：
    0x00 = OP_MAIN      → input: shenfen_id [+ name]（机构主账户）
    0x01 = OP_FEE       → input: shenfen_id          （费用账户）
    0x02 = OP_STAKE     → input: citizens_number_u64_le（省储行质押）
    0x03 = OP_AN        → input: NRC_shenfen_id      （国储会安全基金）
    0x04 = OP_PERSONAL  → input: creator(32B) + name_utf8（个人多签，链上派生）

本工具一次性重算：
  - main_address（所有 7 个机构常量文件：cb/ch/zf/jc/lf/sf/jy）
  - fee_address （cb + ch 共 87 个）
  - stake_address（ch 专有 43 个）
  - NRC_ANQUAN_ADDRESS（cb 内 1 个全局常量）
  - CHINA_RESERVED_MAIN_ADDRESSES 保留名单（zb.rs 汇总表，365 条）

用法：
  python3 duoqian.py               # dry-run，仅打印差异
  python3 duoqian.py --apply       # 写回源码
"""

import argparse
import hashlib
import re
import struct
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


CHINA_DIR = (
    Path(__file__).resolve().parent.parent
    / "citizenchain"
    / "runtime"
    / "primitives"
    / "china"
)

# ── 统一域 ─────────────────────────────────────────
DOMAIN = b"DUOQIAN_V1"  # 10 字节
SS58_FORMAT: int = 2027

OP_MAIN = 0x00
OP_FEE = 0x01
OP_STAKE = 0x02
OP_AN = 0x03
# OP_PERSONAL = 0x04 由链上 derive_personal 使用，不参与 primitives 派生

# 按 china_cb.rs 硬编码，第一条是 NRC
NRC_SHENFEN_ID = "GFR-LN001-CB0C-617776487-20260222"

# 需要处理的机构文件（含 main_address 字段）
FILES_WITH_MAIN = [
    "china_cb.rs",
    "china_ch.rs",
    "china_sf.rs",
    "china_jc.rs",
    "china_lf.rs",
    "china_jy.rs",
    "china_zf.rs",
]

# 只在 cb/ch 里有 fee_address
FILES_WITH_FEE = ["china_cb.rs", "china_ch.rs"]

# 只在 ch 里有 stake_address 和 citizens_number
FILES_WITH_STAKE = ["china_ch.rs"]


# ── 哈希基元 ───────────────────────────────────────
def blake2b_256(data: bytes) -> bytes:
    return hashlib.blake2b(data, digest_size=32).digest()


def ss58_le() -> bytes:
    return SS58_FORMAT.to_bytes(2, byteorder="little")


def derive(op_tag: int, payload: bytes) -> bytes:
    """统一派生入口：DUOQIAN_V1 + op_tag + ss58_le + payload → blake2b_256"""
    preimage = DOMAIN + bytes([op_tag]) + ss58_le() + payload
    return blake2b_256(preimage)


def derive_main(sfid_id: str) -> bytes:
    return derive(OP_MAIN, sfid_id.encode("utf-8"))


def derive_fee(shenfen_id: str) -> bytes:
    return derive(OP_FEE, shenfen_id.encode("utf-8"))


def derive_stake(citizens_number: int) -> bytes:
    return derive(OP_STAKE, struct.pack("<Q", citizens_number))


def derive_anquan() -> bytes:
    return derive(OP_AN, NRC_SHENFEN_ID.encode("utf-8"))


# ── Rust 文件扫描 ───────────────────────────────────
@dataclass
class MainEntry:
    sfid_id: str
    old_hex: str
    new_hex: str
    file_name: str
    line_num: int


@dataclass
class FeeEntry:
    shenfen_id: str
    old_hex: str
    new_hex: str
    file_name: str
    line_num: int


@dataclass
class StakeEntry:
    citizens_number: int
    old_hex: str
    new_hex: str
    file_name: str
    line_num: int


def hexstr(b: bytes) -> str:
    return b.hex()


def extract_main(file_path: Path) -> list[MainEntry]:
    """按 shenfen_id → 下一个 main_address hex!(...) 配对。"""
    text = file_path.read_text(encoding="utf-8")
    lines = text.split("\n")
    out: list[MainEntry] = []

    sfid_re = re.compile(r'shenfen_id:\s*"([^"]+)"')
    addr_re = re.compile(r'main_address:\s*hex!\("([0-9a-fA-F]{64})"\)')

    current_sfid: Optional[str] = None
    for i, line in enumerate(lines):
        m1 = sfid_re.search(line)
        if m1:
            current_sfid = m1.group(1)
        m2 = addr_re.search(line)
        if m2 and current_sfid is not None:
            old = m2.group(1).lower()
            new = hexstr(derive_main(current_sfid))
            out.append(
                MainEntry(
                    sfid_id=current_sfid,
                    old_hex=old,
                    new_hex=new,
                    file_name=file_path.name,
                    line_num=i + 1,
                )
            )
            current_sfid = None
    return out


def extract_fee(file_path: Path) -> list[FeeEntry]:
    """按 shenfen_id → 下一个 fee_address hex!(...) 配对。"""
    text = file_path.read_text(encoding="utf-8")
    lines = text.split("\n")
    out: list[FeeEntry] = []

    sfid_re = re.compile(r'shenfen_id:\s*"([^"]+)"')
    addr_re = re.compile(r'fee_address:\s*hex!\("([0-9a-fA-F]{64})"\)')

    current_sfid: Optional[str] = None
    for i, line in enumerate(lines):
        m1 = sfid_re.search(line)
        if m1:
            current_sfid = m1.group(1)
        m2 = addr_re.search(line)
        if m2 and current_sfid is not None:
            old = m2.group(1).lower()
            new = hexstr(derive_fee(current_sfid))
            out.append(
                FeeEntry(
                    shenfen_id=current_sfid,
                    old_hex=old,
                    new_hex=new,
                    file_name=file_path.name,
                    line_num=i + 1,
                )
            )
    return out


def extract_stake(file_path: Path) -> list[StakeEntry]:
    """按 citizens_number → 下一个 stake_address hex!(...) 配对。"""
    text = file_path.read_text(encoding="utf-8")
    lines = text.split("\n")
    out: list[StakeEntry] = []

    # 处理 Rust 里的下划线分隔符：10_913_902 → 10913902
    cn_re = re.compile(r"citizens_number:\s*([0-9_]+)")
    addr_re = re.compile(r'stake_address:\s*hex!\("([0-9a-fA-F]{64})"\)')

    current_cn: Optional[int] = None
    for i, line in enumerate(lines):
        m1 = cn_re.search(line)
        if m1:
            current_cn = int(m1.group(1).replace("_", ""))
        m2 = addr_re.search(line)
        if m2 and current_cn is not None:
            old = m2.group(1).lower()
            new = hexstr(derive_stake(current_cn))
            out.append(
                StakeEntry(
                    citizens_number=current_cn,
                    old_hex=old,
                    new_hex=new,
                    file_name=file_path.name,
                    line_num=i + 1,
                )
            )
            current_cn = None
    return out


# ── 写回 ────────────────────────────────────────────
def rewrite_main(path: Path, entries: list[MainEntry]) -> None:
    text = path.read_text(encoding="utf-8")
    for e in entries:
        text = text.replace(
            f'main_address: hex!("{e.old_hex}")',
            f'main_address: hex!("{e.new_hex}")',
            1,
        )
    path.write_text(text, encoding="utf-8")


def rewrite_fee(path: Path, entries: list[FeeEntry]) -> None:
    text = path.read_text(encoding="utf-8")
    for e in entries:
        text = text.replace(
            f'fee_address: hex!("{e.old_hex}")',
            f'fee_address: hex!("{e.new_hex}")',
            1,
        )
    path.write_text(text, encoding="utf-8")


def rewrite_stake(path: Path, entries: list[StakeEntry]) -> None:
    text = path.read_text(encoding="utf-8")
    for e in entries:
        text = text.replace(
            f'stake_address: hex!("{e.old_hex}")',
            f'stake_address: hex!("{e.new_hex}")',
            1,
        )
    path.write_text(text, encoding="utf-8")


def rewrite_anquan(cb_path: Path, new_hex: str) -> None:
    """重写 china_cb.rs 里 NRC_ANQUAN_ADDRESS 常量。"""
    text = cb_path.read_text(encoding="utf-8")
    # 匹配形如 pub const NRC_ANQUAN_ADDRESS: [u8; 32] = hex!("...")
    pattern = re.compile(
        r'(pub const NRC_ANQUAN_ADDRESS:\s*\[u8;\s*32\]\s*=\s*\n?\s*hex!\(")([0-9a-fA-F]{64})("\))'
    )
    new_text, n = pattern.subn(rf"\g<1>{new_hex}\g<3>", text)
    if n == 0:
        print("⚠️  china_cb.rs 中没找到 NRC_ANQUAN_ADDRESS 常量，跳过")
        return
    cb_path.write_text(new_text, encoding="utf-8")


def regen_zb(all_addresses: list[str], dry_run: bool) -> None:
    """重建 china_zb.rs：汇总所有保留地址 main + fee + stake + anquan。"""
    zb_path = CHINA_DIR / "china_zb.rs"
    uniq = sorted(set(all_addresses))

    lines = [
        "//! 汇总 runtime/primitives/china 目录下所有制度保留地址",
        "//! （main_address + fee_address + stake_address + NRC_ANQUAN_ADDRESS）。",
        "//! 用于禁止 duoqian-manage 抢注这些机构地址。",
        "//!",
        "//! 派生统一走 `primitives::core_const::DUOQIAN_DOMAIN` + op_tag，由",
        "//! `tools/duoqian.py` 一次性生成，禁止手改。",
        "",
        "use hex_literal::hex;",
        "",
        f"pub const CHINA_RESERVED_MAIN_ADDRESSES: &[[u8; 32]; {len(uniq)}] = &[",
    ]
    for a in uniq:
        lines.append(f'    hex!("{a}"),')
    lines.append("];")
    lines.append("")
    lines.append("/// 检查地址是否属于制度保留地址（静态常量数组二分查找）。")
    lines.append("pub fn is_reserved_main_address(address: &[u8; 32]) -> bool {")
    lines.append("    CHINA_RESERVED_MAIN_ADDRESSES")
    lines.append("        .binary_search(address)")
    lines.append("        .is_ok()")
    lines.append("}")
    lines.append("")

    new_text = "\n".join(lines)
    if dry_run:
        print(f"\n=== china_zb.rs 将包含 {len(uniq)} 个保留地址 ===")
    else:
        zb_path.write_text(new_text, encoding="utf-8")
        print(f"✅ china_zb.rs 已更新：{len(uniq)} 个保留地址")


# ── 主流程 ──────────────────────────────────────────
def main() -> int:
    parser = argparse.ArgumentParser(
        description="统一派生 primitives/china 下的 main/fee/stake/anquan 地址（DUOQIAN_V1 + op_tag）"
    )
    grp = parser.add_mutually_exclusive_group()
    grp.add_argument("--dry-run", action="store_true", default=True, help="仅打印差异（默认）")
    grp.add_argument("--apply", action="store_true", help="写回源码")
    args = parser.parse_args()
    dry_run = not args.apply

    mode = "🔍 干运行" if dry_run else "✏️  写回"
    print(f"{mode} 模式")
    print(f"Domain: {DOMAIN!r}  SS58: {SS58_FORMAT} ({ss58_le().hex()})\n")

    # 汇总用
    all_reserved: list[str] = []

    # ── main_address ──
    main_total = main_changed = 0
    for fn in FILES_WITH_MAIN:
        fp = CHINA_DIR / fn
        if not fp.exists():
            print(f"⚠️  跳过不存在的 {fn}")
            continue
        entries = extract_main(fp)
        changed = [e for e in entries if e.old_hex != e.new_hex]
        main_total += len(entries)
        main_changed += len(changed)
        print(f"📄 [main]  {fn}: {len(entries)} 条，{len(changed)} 变更")
        for e in entries:
            all_reserved.append(e.new_hex)
            if e.old_hex != e.new_hex:
                print(f"   🔄 {e.sfid_id}")
                print(f"      旧: {e.old_hex}")
                print(f"      新: {e.new_hex}")
        if not dry_run and changed:
            rewrite_main(fp, entries)

    # ── fee_address ──
    fee_total = fee_changed = 0
    for fn in FILES_WITH_FEE:
        fp = CHINA_DIR / fn
        if not fp.exists():
            continue
        entries = extract_fee(fp)
        changed = [e for e in entries if e.old_hex != e.new_hex]
        fee_total += len(entries)
        fee_changed += len(changed)
        print(f"📄 [fee]   {fn}: {len(entries)} 条，{len(changed)} 变更")
        for e in entries:
            all_reserved.append(e.new_hex)
            if e.old_hex != e.new_hex:
                print(f"   🔄 {e.shenfen_id}")
                print(f"      旧: {e.old_hex}")
                print(f"      新: {e.new_hex}")
        if not dry_run and changed:
            rewrite_fee(fp, entries)

    # ── stake_address ──
    stake_total = stake_changed = 0
    for fn in FILES_WITH_STAKE:
        fp = CHINA_DIR / fn
        if not fp.exists():
            continue
        entries = extract_stake(fp)
        changed = [e for e in entries if e.old_hex != e.new_hex]
        stake_total += len(entries)
        stake_changed += len(changed)
        print(f"📄 [stake] {fn}: {len(entries)} 条，{len(changed)} 变更")
        for e in entries:
            all_reserved.append(e.new_hex)
            if e.old_hex != e.new_hex:
                print(f"   🔄 citizens_number={e.citizens_number}")
                print(f"      旧: {e.old_hex}")
                print(f"      新: {e.new_hex}")
        if not dry_run and changed:
            rewrite_stake(fp, entries)

    # ── NRC_ANQUAN_ADDRESS ──
    new_anquan = hexstr(derive_anquan())
    all_reserved.append(new_anquan)
    print(f"\n📄 [anquan] NRC_ANQUAN_ADDRESS: {new_anquan}")
    if not dry_run:
        rewrite_anquan(CHINA_DIR / "china_cb.rs", new_anquan)

    # ── china_zb.rs 汇总 ──
    regen_zb(all_reserved, dry_run=dry_run)

    print(
        f"\n==== 统计 ====\n"
        f"main : {main_total} 条，{main_changed} 变更\n"
        f"fee  : {fee_total} 条，{fee_changed} 变更\n"
        f"stake: {stake_total} 条，{stake_changed} 变更\n"
        f"anquan: 1 条\n"
        f"汇总保留地址: {len(set(all_reserved))} 个唯一\n"
    )
    if dry_run:
        print("💡 --apply 写回源码")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
