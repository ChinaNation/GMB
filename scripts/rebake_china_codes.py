#!/usr/bin/env python3
"""china_*.rs 机构码 re-bake:把旧的通用每文件码(GCB/SCH/GZF/GSF/GJC/GLF/GJY)
重写为新 86 码体系下每个机构类型的专属码(NRC/PRC/PRB/PRS/MFA/PGV/...)。

只改 cid_number 的 seg2(码+盈利位+校验位);R5/N9/D4 保留。校验位用后端同一算法
(base36,acc=Σ(idx+1)·pos,payload=R5+code+profit+N9+D4)。所有公权机构 profit=0。

改完后须跑 `python3 scripts/gmb.py --apply` 按新 cid_number 重派生全部账户。

用法:
  python3 scripts/rebake_china_codes.py --scan    # 预览映射,不写
  python3 scripts/rebake_china_codes.py --apply   # 写回 china_*.rs
"""
import argparse
import re
from pathlib import Path

CHINA_DIR = Path(__file__).resolve().parent.parent / "citizenchain/runtime/primitives/cid/china"
ALPHABET = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"

# 国家级单体 name(子串)→ code,按顺序匹配(局/署在前,院/府在后)
ZF_NATIONAL = [
    ("联邦安全局", "FSC"), ("联邦情报局", "FIB"), ("联邦特勤局", "FSS"),
    ("联邦人事局", "FPR"), ("联邦注册局", "FRG"),
    ("外事交流部", "MFA"), ("国家防务部", "MDF"), ("国土安全部", "MHS"),
    ("公民生活保障部", "MCW"), ("住房与城镇建设部", "MHU"), ("农业与农村发展部", "MAG"),
    ("商务与市场贸易部", "MCM"), ("财政与税务部", "MFT"), ("能源与环保发展部", "MEN"),
    ("交通运输部", "MTR"),
    ("总统府", "PRS"),  # 必须在 5 联邦局之后(它们也含"总统府")
]


def checksum_char_mod36(payload: str) -> str:
    total = 0
    for idx, ch in enumerate(payload):
        pos = ALPHABET.find(ch)
        if pos < 0:
            pos = 0
        total += (idx + 1) * pos
    return ALPHABET[total % 36]


def code_for(file_stem: str, name: str, index: int) -> str:
    if file_stem == "china_cb":
        return "NRC" if index == 0 else "PRC"
    if file_stem == "china_ch":
        return "PRB"
    if file_stem == "china_sf":
        return "PJD" if "省" in name else "NJD"
    if file_stem == "china_jc":
        if "廉政署" in name:
            return "FAC"
        if "审计署" in name:
            return "FAU"
        if "调查署" in name:
            return "FIV"
        return "PSP" if "省" in name else "NSP"
    if file_stem == "china_lf":
        return "PLG" if "省" in name else "NLG"
    if file_stem == "china_jy":
        return "NED"
    if file_stem == "china_zf":
        for sub, code in ZF_NATIONAL:
            if sub in name:
                return code
        return "PGV"  # xx省联邦政府
    raise ValueError(f"unknown china file {file_stem}")


def new_cid_number(old: str, code: str) -> str:
    # 旧:R5-SEG2-N9-D4。保留 R5/N9/D4,seg2 = code + '0' + checksum
    parts = old.split("-")
    if len(parts) != 4:
        raise ValueError(f"bad cid_number {old}")
    r5, _seg2, n9, d4 = parts
    payload = f"{r5}{code}0{n9}{d4}"
    c = checksum_char_mod36(payload)
    return f"{r5}-{code}0{c}-{n9}-{d4}"


PAIR_RE = re.compile(
    r'(cid_full_name:\s*")([^"]+)("[^}]*?cid_number:\s*")([^"]+)(")',
    re.DOTALL,
)


def process_file(path: Path, apply: bool):
    stem = path.stem
    text = path.read_text(encoding="utf-8")
    counter = {"i": 0}
    rows = []

    def repl(m):
        idx = counter["i"]
        counter["i"] += 1
        name = m.group(2)
        old_cid = m.group(4)
        code = code_for(stem, name, idx)
        new_cid = new_cid_number(old_cid, code)
        rows.append((name, old_cid, new_cid, code))
        return m.group(1) + name + m.group(3) + new_cid + m.group(5)

    new_text = PAIR_RE.sub(repl, text)
    if apply and new_text != text:
        path.write_text(new_text, encoding="utf-8")
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--scan", action="store_true")
    args = ap.parse_args()
    apply = args.apply and not args.scan

    total = 0
    code_count = {}
    for stem in ("china_cb", "china_ch", "china_zf", "china_sf", "china_jc", "china_lf", "china_jy"):
        path = CHINA_DIR / f"{stem}.rs"
        rows = process_file(path, apply)
        total += len(rows)
        print(f"== {stem} ({len(rows)} 条) ==")
        for name, old, new, code in rows[:3]:
            print(f"  {code:4} {old} -> {new}  [{name}]")
        if len(rows) > 3:
            print(f"  ... 其余 {len(rows) - 3} 条")
        for _, _, _, code in rows:
            code_count[code] = code_count.get(code, 0) + 1
    print(f"\n总计 {total} 条;码分布:", dict(sorted(code_count.items())))
    print("APPLIED" if apply else "SCAN ONLY(加 --apply 写回)")


if __name__ == "__main__":
    main()
