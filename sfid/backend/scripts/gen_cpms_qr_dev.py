#!/usr/bin/env python3
"""
Generate CPMS QR payloads for SFID local/dev integration.

This script follows the current dev verification logic in backend/src/main.rs:
- Citizen/status QR signature: hex(blake2b_256("{pubkey}|{canonical_text}"))
- Register checksum: hex(blake2b_256(register_canonical_text))
"""

import argparse
import json
import sys
import time
import uuid


def blake2b_256_hex(text: str) -> str:
    import hashlib
    return hashlib.blake2b(text.encode("utf-8"), digest_size=32).hexdigest()


def canonical_citizen(payload: dict) -> str:
    base = (
        f"ver={payload['ver']}&issuer_id={payload['issuer_id']}&site_sfid={payload['site_sfid']}"
        f"&archive_no={payload['archive_no']}&issued_at={payload['issued_at']}"
        f"&expire_at={payload['expire_at']}&qr_id={payload['qr_id']}&sig_alg={payload['sig_alg']}"
    )
    status = payload.get("status")
    if status:
        base += f"&status={status}"
    return base


def canonical_status(payload: dict) -> str:
    return (
        f"ver={payload['ver']}&issuer_id={payload['issuer_id']}&site_sfid={payload['site_sfid']}"
        f"&archive_no={payload['archive_no']}&status={payload['status']}"
        f"&issued_at={payload['issued_at']}&expire_at={payload['expire_at']}"
        f"&qr_id={payload['qr_id']}&sig_alg={payload['sig_alg']}"
    )


def canonical_register(payload: dict) -> str:
    return (
        f"site_sfid={payload['site_sfid']}&pubkey_1={payload['pubkey_1']}"
        f"&pubkey_2={payload['pubkey_2']}&pubkey_3={payload['pubkey_3']}"
        f"&issued_at={payload['issued_at']}"
    )


def print_payload(payload: dict, canonical: str) -> None:
    print(json.dumps(payload, ensure_ascii=False))
    print("\n# canonical_text")
    print(canonical)


def cmd_citizen(args: argparse.Namespace) -> int:
    now = int(time.time())
    payload = {
        "ver": "1",
        "issuer_id": "cpms",
        "site_sfid": args.site_sfid,
        "archive_no": args.archive_no,
        "issued_at": args.issued_at or now,
        "expire_at": args.expire_at or (now + args.ttl),
        "qr_id": args.qr_id or str(uuid.uuid4()),
        "sig_alg": "sr25519",
        "status": args.status.upper(),
    }
    canonical = canonical_citizen(payload)
    payload["signature"] = blake2b_256_hex(f"{args.sign_pubkey}|{canonical}")
    print_payload(payload, canonical)
    return 0


def cmd_status(args: argparse.Namespace) -> int:
    now = int(time.time())
    payload = {
        "ver": "1",
        "issuer_id": "cpms",
        "site_sfid": args.site_sfid,
        "archive_no": args.archive_no,
        "status": args.status.upper(),
        "issued_at": args.issued_at or now,
        "expire_at": args.expire_at or (now + args.ttl),
        "qr_id": args.qr_id or str(uuid.uuid4()),
        "sig_alg": "sr25519",
    }
    canonical = canonical_status(payload)
    payload["signature"] = blake2b_256_hex(f"{args.sign_pubkey}|{canonical}")
    print_payload(payload, canonical)
    return 0


def cmd_register(args: argparse.Namespace) -> int:
    now = int(time.time())
    payload = {
        "site_sfid": args.site_sfid,
        "pubkey_1": args.pubkey_1,
        "pubkey_2": args.pubkey_2,
        "pubkey_3": args.pubkey_3,
        "issued_at": args.issued_at or now,
    }
    canonical = canonical_register(payload)
    payload["checksum_or_signature"] = blake2b_256_hex(canonical)
    print_payload(payload, canonical)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Generate CPMS QR payload JSON for SFID dev")
    sub = parser.add_subparsers(dest="command", required=True)

    p_citizen = sub.add_parser("citizen", help="Generate citizen bind QR payload")
    p_citizen.add_argument("--site-sfid", required=True)
    p_citizen.add_argument("--archive-no", required=True)
    p_citizen.add_argument("--sign-pubkey", required=True)
    p_citizen.add_argument("--status", required=True, choices=["NORMAL", "ABNORMAL"])
    p_citizen.add_argument("--issued-at", type=int)
    p_citizen.add_argument("--expire-at", type=int)
    p_citizen.add_argument("--ttl", type=int, default=600)
    p_citizen.add_argument("--qr-id")
    p_citizen.set_defaults(func=cmd_citizen)

    p_status = sub.add_parser("status", help="Generate status-change QR payload")
    p_status.add_argument("--site-sfid", required=True)
    p_status.add_argument("--archive-no", required=True)
    p_status.add_argument("--status", required=True, choices=["NORMAL", "ABNORMAL"])
    p_status.add_argument("--sign-pubkey", required=True)
    p_status.add_argument("--issued-at", type=int)
    p_status.add_argument("--expire-at", type=int)
    p_status.add_argument("--ttl", type=int, default=600)
    p_status.add_argument("--qr-id")
    p_status.set_defaults(func=cmd_status)

    p_register = sub.add_parser("register", help="Generate CPMS key-register QR payload")
    p_register.add_argument("--site-sfid", required=True)
    p_register.add_argument("--pubkey-1", required=True)
    p_register.add_argument("--pubkey-2", required=True)
    p_register.add_argument("--pubkey-3", required=True)
    p_register.add_argument("--issued-at", type=int)
    p_register.set_defaults(func=cmd_register)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return args.func(args)
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
