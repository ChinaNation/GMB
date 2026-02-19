def pallet_to_account(pallet_id: str) -> str:
    """
    Substrate 官方 PalletId → AccountId 转换
    规则：
    AccountId = b"modl" + pallet_id(8 bytes) + 20 bytes 0 padding
    长度固定 32 bytes
    无哈希
    """

    pid_bytes = pallet_id.encode("utf-8")

    if len(pid_bytes) != 8:
        raise ValueError(f"{pallet_id} 必须正好 8 字节")

    account = b"modl" + pid_bytes + b"\x00" * 20
    return "0x" + account.hex()


def batch_calc(pallet_ids: list[str]):
    results = []

    for pid in pallet_ids:
        addr = pallet_to_account(pid)
        results.append({
            "pallet_id": pid,
            "account_id": addr
        })

    return results


# ===== 在这里填你所有ID =====
ids = [
    "nrcgch01",
    "prczss01",
    "prclns02",
    "prcgds03",
    "prcgxs04",
    "prcfjs05",
    "prchns06",
    "prcyns07",
    "prcgzs08",
    "prchns09",
    "prcjxs10",
    "prczjs11",
    "prcjss12",
    "prcsds13",
    "prcsxs14",    
    "prchns15",
    "prchbs16",
    "prchbs17",
    "prcsxs18",
    "prccqs19",    
    "prcscs20",
    "prcgss21",
    "prcbps22",
    "prcbhs23",
    "prcsjs24",    
    "prcljs25",
    "prcjls26",
    "prclns27",
    "prcnxs28",
    "prcqhs29",
    "prcahs30",
    "prctws31",
    "prcxzs32",
    "prcxjs33",
    "prcxks34",
    "prcals35",
    "prccls36",
    "prctss37",
    "prchxs38",
    "prckls39",    
    "prchts40",
    "prcrhs41",
    "prcxas42",
    "prchjs43",
]


data = batch_calc(ids)


print(f"{'PalletID':<12} | AccountId")
print("-" * 80)

for item in data:
    print(f"{item['pallet_id']:<12} | {item['account_id']}")