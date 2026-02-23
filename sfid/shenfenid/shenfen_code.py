#!/usr/bin/env python3
"""
身份识别码系统
A3+R5+T2P1C1+N9+D*=A3分类码、R5省市码、T2组织码、P1盈利属性码、C1校验码、N9桶扰码、D*日期码
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import os
import re
import sqlite3
from dataclasses import dataclass
from pathlib import Path

try:
    from tools.shenfenid.city_codes import CITY_OPTIONS_BY_PROVINCE, PROVINCE_OPTIONS
except ModuleNotFoundError:
    from city_codes import CITY_OPTIONS_BY_PROVINCE, PROVINCE_OPTIONS

A3_OPTIONS = [
    ("公民人", "GMR"), ("自然人", "ZRR"), ("智能人", "ZNR"),
    ("公法人", "GFR"), ("私法人", "SFR"), ("非法人", "FFR"),
]
A3_SET = {c for _, c in A3_OPTIONS}

T2_OPTIONS = [
    ("中国", "ZG"), ("政府", "ZF"), ("立法院", "LF"),("司法院", "SF"), ("监察院", "JC"),
    ("公民教育委员会", "JY"), ("公民储备委员会", "CB"), ("公民储备银行", "CH"),("他国", "TG"), 
]
T2_SET = {c for _, c in T2_OPTIONS}

P1_OPTIONS = [("非盈利", "0"), ("盈利", "1")]
P1_SET = {c for _, c in P1_OPTIONS}

PROVINCE_CODES = {code for _, code in PROVINCE_OPTIONS}
PROVINCE_NAME_BY_CODE = {code: name for name, code in PROVINCE_OPTIONS}

ALPHABET = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
SEGMENTED_RE = re.compile(r"^([A-Z]{3})-([A-Z0-9]{5})-([A-Z]{2})([01])([A-Z0-9])-([0-9]{9})-([0-9]+)$")
UNSEGMENTED_RE = re.compile(r"^([A-Z]{3})([A-Z0-9]{5})([A-Z]{2})([01])([A-Z0-9])([0-9]{9})([0-9]+)$")


@dataclass
class ParsedCode:
    a3: str
    r5: str
    t2: str
    p1: str
    c1: str
    n9: str
    d: str


def to_val(ch: str) -> int:
    return ALPHABET.index(ch)


def checksum(payload: str) -> str:
    total = 0
    for i, ch in enumerate(payload, start=1):
        total = (total + i * to_val(ch)) % 36
    return ALPHABET[total]


def is_leap(year: int) -> bool:
    return year % 4 == 0 and (year % 100 != 0 or year % 400 == 0)


def validate_date(d: str) -> None:
    if not d.isdigit() or len(d) < 5:
        raise ValueError("D* must be Y...YMMDD")
    y = int(d[:-4])
    m = int(d[-4:-2])
    day = int(d[-2:])
    if y < 1:
        raise ValueError("year must be >= 1")
    if m < 1 or m > 12:
        raise ValueError("month must be 01..12")
    md = [31, 29 if is_leap(y) else 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m - 1]
    if day < 1 or day > md:
        raise ValueError("day out of range")


def validate_fields(a3: str, r5: str, t2: str, p1: str, d: str) -> None:
    if a3 not in A3_SET:
        raise ValueError("invalid A3")
    if not re.fullmatch(r"[A-Z0-9]{2}[0-9]{3}", r5):
        raise ValueError("R5 must be 2 alnum chars + 3 digits")
    if r5[:2] not in PROVINCE_CODES:
        raise ValueError(f"province code '{r5[:2]}' not configured")
    if t2 not in T2_SET:
        raise ValueError("invalid T2")
    if p1 not in P1_SET:
        raise ValueError("invalid P1")
    validate_date(d)

    if a3 in {"GMR", "ZRR", "ZNR", "SFR"} and t2 != "ZG":
        raise ValueError(f"{a3} requires T2=ZG")
    if a3 == "GFR":
        if t2 == "ZG":
            raise ValueError("GFR cannot use T2=ZG")
        if p1 != "0":
            raise ValueError("GFR requires P1=0")
    if a3 in {"GMR", "ZRR"} and p1 != "1":
        raise ValueError(f"{a3} requires P1=1")


def parse_code(raw: str) -> ParsedCode:
    txt = raw.strip().upper()
    m = SEGMENTED_RE.match(txt) or UNSEGMENTED_RE.match(txt)
    if not m:
        raise ValueError("code format invalid")
    p = ParsedCode(*m.groups())
    validate_fields(p.a3, p.r5, p.t2, p.p1, p.d)
    return p


def payload(p: ParsedCode) -> str:
    return f"{p.a3}{p.r5}{p.t2}{p.p1}{p.n9}{p.d}"


def fmt(p: ParsedCode) -> str:
    return f"{p.a3}-{p.r5}-{p.t2}{p.p1}{p.c1}-{p.n9}-{p.d}"


def account(p: ParsedCode) -> str:
    raw = f"{p.a3}{p.r5}{p.t2}{p.p1}{p.c1}{p.n9}{p.d}".encode()
    return "0x" + hashlib.blake2b(raw, digest_size=32).hexdigest()


def ensure_db(conn: sqlite3.Connection) -> None:
    conn.execute("CREATE TABLE IF NOT EXISTS n9_sequence(bucket TEXT PRIMARY KEY, next_seq INTEGER NOT NULL)")


def next_seq(conn: sqlite3.Connection, bucket: str) -> int:
    conn.execute("BEGIN IMMEDIATE")
    row = conn.execute("SELECT next_seq FROM n9_sequence WHERE bucket=?", (bucket,)).fetchone()
    if row is None:
        seq = 0
        conn.execute("INSERT INTO n9_sequence(bucket,next_seq) VALUES(?,?)", (bucket, 1))
    else:
        seq = row[0]
        if seq >= 1_000_000_000:
            conn.rollback()
            raise ValueError("N9 exhausted")
        conn.execute("UPDATE n9_sequence SET next_seq=? WHERE bucket=?", (seq + 1, bucket))
    conn.commit()
    return seq


def perturb_n9(seq: int, bucket: str, secret: str) -> str:
    d = hashlib.blake2b(f"{bucket}|{secret}".encode(), digest_size=16).digest()
    a = int.from_bytes(d[:8], "big") % 1_000_000_000
    while a % 2 == 0 or a % 5 == 0:
        a = (a + 1) % 1_000_000_000
        if a == 0:
            a = 1
    b = int.from_bytes(d[8:], "big") % 1_000_000_000
    return f"{(a * seq + b) % 1_000_000_000:09d}"


def pick(title: str, opts: list[tuple[str, str]]) -> str:
    print(f"\n{title}")
    for i, (label, code) in enumerate(opts, 1):
        print(f"{i:2d}. {label} ({code})")
    while True:
        s = input("请输入序号: ").strip()
        if s.isdigit() and 1 <= int(s) <= len(opts):
            return opts[int(s)-1][1]
        print("输入无效，请重试")


def today() -> str:
    return datetime.date.today().strftime("%Y%m%d")


def input_date() -> str:
    t = today()
    while True:
        s = input(f"请输入日期 D*（Y...YMMDD，回车默认 {t}）: ").strip()
        if s == "":
            return t
        try:
            validate_date(s)
            return s
        except ValueError as e:
            print(f"日期无效: {e}")


def cmd_generate(args: argparse.Namespace) -> int:
    db = Path(args.db).expanduser().resolve()
    db.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db))
    try:
        ensure_db(conn)
        a3 = pick("请选择 A3（主体属性）", A3_OPTIONS)
        prov = pick("请选择省代码（R5前两位）", PROVINCE_OPTIONS)
        city_opts = CITY_OPTIONS_BY_PROVINCE.get(prov, [])
        if not city_opts:
            raise ValueError(f"{prov} 暂无县级数据，请先确认映射")
        city = pick("请选择市代码（R5后三位）", city_opts)
        r5 = f"{prov}{city}"

        if a3 in {"GMR", "ZRR", "ZNR", "SFR"}:
            t2 = "ZG"
            print("T2 自动设为 ZG")
        elif a3 == "GFR":
            t2 = pick("请选择 T2（公法人不可选 ZG）", [x for x in T2_OPTIONS if x[1] != "ZG"])
        else:
            t2 = pick("请选择 T2", T2_OPTIONS)

        if a3 in {"GMR", "ZRR"}:
            p1 = "1"
            print("P1 自动设为 1")
        elif a3 == "GFR":
            p1 = "0"
            print("P1 自动设为 0")
        else:
            p1 = pick("请选择 P1", P1_OPTIONS)

        d = input_date() if a3 in {"GMR", "ZRR"} else today()
        if a3 not in {"GMR", "ZRR"}:
            print(f"D* 自动设为当天日期：{d}")

        validate_fields(a3, r5, t2, p1, d)
        bucket = f"{a3}|{r5}|{t2}|{p1}|{d}"
        secret = args.secret or os.environ.get("IDENTITY_N9_SECRET", "GMB-IDENTITY-DEFAULT-SECRET")
        n9 = perturb_n9(next_seq(conn, bucket), bucket, secret)
    finally:
        conn.close()

    p = ParsedCode(a3, r5, t2, p1, checksum(f"{a3}{r5}{t2}{p1}{n9}{d}"), n9, d)
    print(f"\n省份: {PROVINCE_NAME_BY_CODE.get(prov, prov)}")
    print(f"R5: {r5}")
    print(f"D*: {d}")
    print(f"CODE: {fmt(p)}")
    print(f"ACCOUNT: {account(p)}")
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    try:
        p = parse_code(args.code)
    except ValueError as e:
        print(f"INVALID: {e}")
        return 1
    exp = checksum(payload(p))
    if exp == p.c1:
        print("VALID")
        return 0
    print(f"INVALID: checksum mismatch, expected {exp}, got {p.c1}")
    return 1


def cmd_parse(args: argparse.Namespace) -> int:
    try:
        p = parse_code(args.code)
    except ValueError as e:
        print(f"INVALID: {e}")
        return 1
    exp = checksum(payload(p))
    print(f"code:      {fmt(p)}")
    print(f"A3:        {p.a3}")
    print(f"R5:        {p.r5}")
    print(f"T2:        {p.t2}")
    print(f"P1:        {p.p1}")
    print(f"C1:        {p.c1}")
    print(f"N9:        {p.n9}")
    print(f"D*:        {p.d}")
    print(f"account:   {account(p)}")
    print(f"checksum:  {'OK' if exp == p.c1 else f'FAIL (expected {exp})'}")
    return 0 if exp == p.c1 else 1


def cmd_account(args: argparse.Namespace) -> int:
    try:
        p = parse_code(args.code)
    except ValueError as e:
        print(f"INVALID: {e}")
        return 1
    print(account(p))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Shenfen code tool")
    sub = parser.add_subparsers(dest="cmd", required=True)

    g = sub.add_parser("generate", help="交互生成")
    g.add_argument("--db", default="tools/shenfenid/.identity_code_seq.db")
    g.add_argument("--secret", default="")
    g.set_defaults(func=cmd_generate)

    v = sub.add_parser("verify", help="校验")
    v.add_argument("--code", required=True)
    v.set_defaults(func=cmd_verify)

    p = sub.add_parser("parse", help="解析")
    p.add_argument("--code", required=True)
    p.set_defaults(func=cmd_parse)

    a = sub.add_parser("account", help="派生账户")
    a.add_argument("--code", required=True)
    a.set_defaults(func=cmd_account)

    return parser


if __name__ == "__main__":
    parser = build_parser()
    args = parser.parse_args()
    raise SystemExit(args.func(args))
