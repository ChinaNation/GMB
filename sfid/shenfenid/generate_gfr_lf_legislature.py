#!/usr/bin/env python3
"""按公法人(GFR)+立法院(LF)生成国家立法院与各省立法院身份码与账户哈希。"""

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


def make_record(sc, org_name: str, province_name: str, province_code: str, city_name: str, city_code: str, date: str, secret: str, seq: int):
    r5 = f"{province_code}{city_code}"
    a3 = "GFR"
    t2 = "LF"
    p1 = "0"
    d = date
    sc.validate_fields(a3, r5, t2, p1, d)
    bucket = f"{a3}|{r5}|{t2}|{p1}|{d}"
    n9 = sc.perturb_n9(seq, bucket, secret)
    c1 = sc.checksum(f"{a3}{r5}{t2}{p1}{n9}{d}")
    parsed = sc.ParsedCode(a3, r5, t2, p1, c1, n9, d)
    return {
        "org_name": org_name,
        "province_name": province_name,
        "province_code": province_code,
        "city_name": city_name,
        "city_code": city_code,
        "seq": seq,
        "code": sc.fmt(parsed),
        "account": sc.account(parsed),
    }


def build_records(sc, date: str, secret: str):
    records = []
    city_name, city_code = sc.CITY_OPTIONS_BY_PROVINCE["ZS"][0]
    # 国家与中枢省同属 ZS001；用不同 seq 避免身份码冲突。
    records.append(make_record(sc, "国家立法院", "中枢省", "ZS", city_name, city_code, date, secret, seq=0))

    for province_name, province_code in sc.PROVINCE_OPTIONS:
        city_opts = sc.CITY_OPTIONS_BY_PROVINCE.get(province_code, [])
        if not city_opts:
            continue
        city_name, city_code = city_opts[0]
        seq = 1 if province_code == "ZS" else 0
        records.append(
            make_record(
                sc,
                f"{province_name}立法院",
                province_name,
                province_code,
                city_name,
                city_code,
                date,
                secret,
                seq=seq,
            )
        )
    return records


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate national+provincial legislature codes by GFR+LF")
    parser.add_argument("--date", default="", help="D* date (Y...YMMDD); default uses shenfen_code.today()")
    parser.add_argument("--secret", default="", help="N9 secret; default follows shenfen_code generate behavior")
    parser.add_argument("--json", action="store_true", help="print JSON")
    args = parser.parse_args()

    sc = load_shenfen_code_module()
    date = args.date or sc.today()
    secret = args.secret or "GMB-IDENTITY-DEFAULT-SECRET"
    records = build_records(sc, date=date, secret=secret)

    if args.json:
        print(json.dumps(records, ensure_ascii=False, indent=2))
        return 0

    print("org\tprovince\tcity\tseq\tcode\taccount")
    for r in records:
        print(
            f"{r['org_name']}\t{r['province_name']}\t{r['city_name']}\t{r['seq']}\t{r['code']}\t{r['account']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
