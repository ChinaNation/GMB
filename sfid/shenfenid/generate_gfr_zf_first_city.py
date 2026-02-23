#!/usr/bin/env python3
"""按公法人(GFR)+政府(ZF)生成每省第1个市的身份码与账户哈希。"""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path


def load_shenfen_code_module():
    here = Path(__file__).resolve().parent
    target = here / "shenfen_code.py"
    spec = importlib.util.spec_from_file_location("shenfen_code_runtime", target)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {target}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


def build_records(sc, date: str, secret: str, seq: int):
    records = []
    for province_name, province_code in sc.PROVINCE_OPTIONS:
        city_opts = sc.CITY_OPTIONS_BY_PROVINCE.get(province_code, [])
        if not city_opts:
            continue
        city_name, city_code = city_opts[0]
        r5 = f"{province_code}{city_code}"
        a3 = "GFR"
        t2 = "ZF"
        p1 = "0"
        d = date
        sc.validate_fields(a3, r5, t2, p1, d)
        bucket = f"{a3}|{r5}|{t2}|{p1}|{d}"
        n9 = sc.perturb_n9(seq, bucket, secret)
        c1 = sc.checksum(f"{a3}{r5}{t2}{p1}{n9}{d}")
        parsed = sc.ParsedCode(a3, r5, t2, p1, c1, n9, d)
        records.append(
            {
                "province_name": province_name,
                "province_code": province_code,
                "city_name": city_name,
                "city_code": city_code,
                "code": sc.fmt(parsed),
                "account": sc.account(parsed),
            }
        )
    return records


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate first-city GFR+ZF codes by province")
    parser.add_argument("--date", default="", help="D* date (Y...YMMDD); default uses shenfen_code.today()")
    parser.add_argument("--secret", default="", help="N9 secret; default follows shenfen_code generate behavior")
    parser.add_argument("--seq", type=int, default=0, help="N9 seq for each province bucket; default 0 (first code)")
    parser.add_argument("--json", action="store_true", help="print JSON")
    args = parser.parse_args()

    sc = load_shenfen_code_module()
    date = args.date or sc.today()
    secret = args.secret or "GMB-IDENTITY-DEFAULT-SECRET"

    records = build_records(sc, date=date, secret=secret, seq=args.seq)
    if args.json:
        print(json.dumps(records, ensure_ascii=False, indent=2))
        return 0

    print("province\tcity\tcode\taccount")
    for r in records:
        print(f"{r['province_name']}\t{r['city_name']}\t{r['code']}\t{r['account']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
