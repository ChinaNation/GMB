#!/usr/bin/env python3
"""检查 raw chainspec 是否具备公民宪法创世冻结条件。

本脚本只读 raw chainspec,不连接节点、不依赖 runtime metadata。
检查项:
1. `:code` 存在,可选校验其字节等于指定 CI WASM。
2. `LegislationYuan::Laws[0]` 是宪法、全国 scope、v1 生效、无待生效版。
3. `LegislationYuan::LawVersions[0][1]` 存在且包含不可修改条款。
4. `ConstitutionImmutableManifest` 清单与 v1 条文摘要逐字匹配。
5. `LawsByScope[Constitution][0] == [0]`, `NextLawId == 1`。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path

import xxhash

PALLET = b"LegislationYuan"
CONSTITUTION_LAW_ID = 0
GENESIS_VERSION = 1
TIER_CONSTITUTION = 0
LAW_STATUS_EFFECTIVE = 1
IMMUTABLE_ARTICLES = [1, 2, 3, 17, 19, 24, 34, 42]
CODE_KEY = "0x3a636f6465"


def twox_128(data: bytes) -> bytes:
    return (
        xxhash.xxh64(data, seed=0).intdigest().to_bytes(8, "little")
        + xxhash.xxh64(data, seed=1).intdigest().to_bytes(8, "little")
    )


def blake2_128(data: bytes) -> bytes:
    return hashlib.blake2b(data, digest_size=16).digest()


def blake2_256(data: bytes) -> bytes:
    return hashlib.blake2b(data, digest_size=32).digest()


def u32(v: int) -> bytes:
    return v.to_bytes(4, "little")


def u64(v: int) -> bytes:
    return v.to_bytes(8, "little")


def map_prefix(storage: bytes) -> bytes:
    return twox_128(PALLET) + twox_128(storage)


def blake2_128_concat(encoded: bytes) -> bytes:
    return blake2_128(encoded) + encoded


def storage_value(storage: bytes) -> str:
    return "0x" + map_prefix(storage).hex()


def storage_map(storage: bytes, key: bytes) -> str:
    return "0x" + (map_prefix(storage) + blake2_128_concat(key)).hex()


def storage_double_map(storage: bytes, key1: bytes, key2: bytes) -> str:
    return "0x" + (map_prefix(storage) + blake2_128_concat(key1) + blake2_128_concat(key2)).hex()


class Scale:
    def __init__(self, data: bytes) -> None:
        self.data = data
        self.i = 0

    def _need(self, n: int) -> None:
        if self.i + n > len(self.data):
            raise ValueError("SCALE 数据长度不足")

    def u8(self) -> int:
        self._need(1)
        v = self.data[self.i]
        self.i += 1
        return v

    def u32(self) -> int:
        self._need(4)
        v = int.from_bytes(self.data[self.i : self.i + 4], "little")
        self.i += 4
        return v

    def u64(self) -> int:
        self._need(8)
        v = int.from_bytes(self.data[self.i : self.i + 8], "little")
        self.i += 8
        return v

    def raw(self, n: int) -> bytes:
        self._need(n)
        v = self.data[self.i : self.i + n]
        self.i += n
        return v

    def compact(self) -> int:
        first = self.u8()
        mode = first & 0x03
        if mode == 0:
            return first >> 2
        if mode == 1:
            second = self.u8()
            return ((second << 8) | first) >> 2
        if mode == 2:
            rest = self.raw(3)
            return int.from_bytes(bytes([first]) + rest, "little") >> 2
        length = (first >> 2) + 4
        return int.from_bytes(self.raw(length), "little")

    def vec_bytes(self) -> bytes:
        return self.raw(self.compact())

    def opt_bytes(self) -> bytes | None:
        tag = self.u8()
        if tag == 0:
            return None
        if tag != 1:
            raise ValueError(f"非法 Option tag: {tag}")
        return self.vec_bytes()

    def opt_u32(self) -> int | None:
        tag = self.u8()
        if tag == 0:
            return None
        if tag != 1:
            raise ValueError(f"非法 Option tag: {tag}")
        return self.u32()


@dataclass
class Law:
    law_id: int
    tier: int
    scope_code: int
    effective_version: int | None
    latest_version: int
    pending_version: int | None
    status: int


@dataclass
class Version:
    law_id: int
    version: int
    articles: dict[int, bytes]
    published_at: int
    effective_at: int


def parse_law(raw: bytes) -> Law:
    s = Scale(raw)
    law_id = s.u64()
    tier = s.u8()
    scope_code = s.u32()
    houses_len = s.compact()
    s.raw(houses_len * 36)
    effective_version = s.opt_u32()
    latest_version = s.u32()
    pending_version = s.opt_u32()
    status = s.u8()
    return Law(law_id, tier, scope_code, effective_version, latest_version, pending_version, status)


def skip_clause(s: Scale) -> None:
    s.u32()
    s.vec_bytes()
    s.opt_bytes()


def parse_article(s: Scale) -> tuple[int, bytes]:
    start = s.i
    number = s.u32()
    s.vec_bytes()
    s.opt_bytes()
    s.vec_bytes()
    s.opt_bytes()
    for _ in range(s.compact()):
        skip_clause(s)
    return number, s.data[start : s.i]


def parse_section(s: Scale, articles: dict[int, bytes]) -> None:
    s.u32()
    s.vec_bytes()
    s.opt_bytes()
    for _ in range(s.compact()):
        number, raw_article = parse_article(s)
        articles[number] = raw_article


def parse_chapter(s: Scale, articles: dict[int, bytes]) -> None:
    s.u32()
    s.vec_bytes()
    s.opt_bytes()
    for _ in range(s.compact()):
        parse_section(s, articles)


def parse_version(raw: bytes) -> Version:
    s = Scale(raw)
    law_id = s.u64()
    version = s.u32()
    s.vec_bytes()
    s.opt_bytes()
    articles: dict[int, bytes] = {}
    for _ in range(s.compact()):
        parse_chapter(s, articles)
    s.raw(32)
    s.u8()
    s.u64()
    published_at = s.u64()
    effective_at = s.u64()
    return Version(law_id, version, articles, published_at, effective_at)


def parse_vec_u64(raw: bytes) -> list[int]:
    s = Scale(raw)
    return [s.u64() for _ in range(s.compact())]


def parse_manifest(raw: bytes) -> tuple[list[int], list[bytes]]:
    s = Scale(raw)
    numbers = [s.u32() for _ in range(s.compact())]
    hashes = [s.raw(32) for _ in range(s.compact())]
    return numbers, hashes


class RpcTop:
    """--rpc 模式:以 state_getStorage(key, at) 透明替代 raw.top 字典。

    plain chainspec(ADR-031 D5)不再物化 GB 级 raw state,检查改为
    对临时节点的创世块按键查询,键与断言逻辑与文件模式完全一致。
    """

    def __init__(self, url: str, at: str | None) -> None:
        self.url = url
        self.at = at

    def get(self, key: str) -> str | None:
        import urllib.request

        if not key.startswith("0x"):
            key = "0x" + key
        params = [key] + ([self.at] if self.at else [])
        body = json.dumps(
            {"jsonrpc": "2.0", "id": 1, "method": "state_getStorage", "params": params}
        ).encode()
        req = urllib.request.Request(
            self.url, data=body, headers={"content-type": "application/json"}
        )
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read()).get("result")


def top_value(top, key: str, label: str) -> bytes:
    value = top.get(key.lower()) or top.get(key)
    if value is None:
        raise AssertionError(f"缺少 {label}: {key}")
    raw = value[2:] if value.startswith("0x") else value
    return bytes.fromhex(raw)


def check(path: Path | None, expect_code_file: Path | None, rpc_top=None) -> None:
    if rpc_top is not None:
        top = rpc_top
    else:
        spec = json.loads(path.read_text())
        top = spec.get("genesis", {}).get("raw", {}).get("top", {})
        if not isinstance(top, dict):
            raise AssertionError("chainspec 缺 genesis.raw.top")

    code = top_value(top, CODE_KEY, ":code")
    if not code:
        raise AssertionError(":code 为空")
    if expect_code_file is not None:
        expected = expect_code_file.read_bytes()
        if code != expected:
            raise AssertionError(
                f":code 与 WASM 文件不一致: chainspec={len(code)} bytes, wasm={len(expected)} bytes"
            )

    law = parse_law(top_value(top, storage_map(b"Laws", u64(0)), "Laws[0]"))
    assert law.law_id == CONSTITUTION_LAW_ID, f"Laws[0].law_id 异常: {law.law_id}"
    assert law.tier == TIER_CONSTITUTION, f"Laws[0].tier 不是 Constitution: {law.tier}"
    assert law.scope_code == 0, f"Laws[0].scope_code 不是全国 0: {law.scope_code}"
    assert law.effective_version == GENESIS_VERSION, f"宪法创世生效版本应为 v1: {law}"
    assert law.latest_version == GENESIS_VERSION, f"宪法创世最新版本应为 v1: {law}"
    assert law.pending_version is None, f"宪法创世不得有待生效版本: {law}"
    assert law.status == LAW_STATUS_EFFECTIVE, f"宪法创世状态应为 Effective: {law.status}"

    version = parse_version(
        top_value(top, storage_double_map(b"LawVersions", u64(0), u32(1)), "LawVersions[0][1]")
    )
    assert version.law_id == CONSTITUTION_LAW_ID, f"LawVersion law_id 异常: {version.law_id}"
    assert version.version == GENESIS_VERSION, f"LawVersion version 异常: {version.version}"

    missing = [n for n in IMMUTABLE_ARTICLES if n not in version.articles]
    if missing:
        raise AssertionError(f"宪法 v1 缺不可修改条款: {missing}")

    numbers, hashes = parse_manifest(
        top_value(top, storage_value(b"ConstitutionImmutableManifest"), "ConstitutionImmutableManifest")
    )
    assert numbers == IMMUTABLE_ARTICLES, f"manifest 清单异常: {numbers}"
    assert len(hashes) == len(numbers), "manifest 条号与摘要数量不一致"
    for number, digest in zip(numbers, hashes):
        actual = blake2_256(version.articles[number])
        if actual != digest:
            raise AssertionError(f"manifest 第 {number} 条摘要与宪法 v1 条文不一致")

    scope = parse_vec_u64(
        top_value(
            top,
            storage_double_map(b"LawsByScope", bytes([TIER_CONSTITUTION]), u32(0)),
            "LawsByScope[Constitution][0]",
        )
    )
    assert scope == [CONSTITUTION_LAW_ID], f"宪法层级唯一性异常: {scope}"

    next_law_id = Scale(top_value(top, storage_value(b"NextLawId"), "NextLawId")).u64()
    assert next_law_id == 1, f"NextLawId 应为 1: {next_law_id}"

    print("constitution genesis check ok")
    print(f"  spec: {path}")
    print(f"  :code bytes: {len(code)}")
    print("  law_id=0 tier=Constitution effective_version=1 latest_version=1 pending=None")
    print("  immutable articles:", ",".join(str(n) for n in numbers))


def main() -> int:
    parser = argparse.ArgumentParser(description="检查公民宪法创世冻结条件(raw 文件或 --rpc 临时节点)")
    parser.add_argument("chainspec", type=Path, nargs="?")
    parser.add_argument("--expect-code-file", type=Path)
    parser.add_argument("--rpc", help="临时节点 RPC 地址,如 http://127.0.0.1:19944")
    parser.add_argument("--at", help="创世块哈希(--rpc 模式钉块查询)")
    args = parser.parse_args()

    if args.rpc is None and args.chainspec is None:
        parser.error("必须提供 raw chainspec 文件或 --rpc")

    try:
        check(
            args.chainspec,
            args.expect_code_file,
            rpc_top=RpcTop(args.rpc, args.at) if args.rpc else None,
        )
    except Exception as exc:  # noqa: BLE001
        print(f"constitution genesis check failed: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
