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
    "prbzss01",
    "prblns02",
    "prbgds03",
    "prbgxs04",
    "prbfjs05",
    "prbhns06",
    "prbyns07",
    "prbgzs08",
    "prbhns09",
    "prbjxs10",
    "prbzjs11",
    "prbjss12",
    "prbsds13",
    "prbsxs14",
    "prbhns15",    
    "prbhbs16",
    "prbhbs17",
    "prbsxs18",
    "prbcqs19",
    "prbscs20",    
    "prbgss21",
    "prbbps22",
    "prbbhs23",
    "prbsjs24",
    "prbljs25",    
    "prbjls26",
    "prblns27",
    "prbnxs28",
    "prbqhs29",
    "prbahs30",
    "prbtws31",
    "prbxzs32",
    "prbxjs33",
    "prbxks34",
    "prbals35",
    "prbcls36",
    "prbtss37",
    "prbhxs38",
    "prbkls39",
    "prbhts40",    
    "prbrhs41",
    "prbxas42",
    "prbhjs43",
]


data = batch_calc(ids)


print(f"{'PalletID':<12} | AccountId")
print("-" * 80)

for item in data:
    print(f"{item['pallet_id']:<12} | {item['account_id']}")