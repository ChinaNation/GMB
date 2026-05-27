# CPMS ARCHIVE 钱包账户

- 最后更新:2026-05-26
- 任务卡:`memory/08-tasks/open/20260526-cpms-wallet-address-only.md`

## 1. 目标

CPMS 是线下实名档案系统，只负责确认用户现场出示的钱包账户地址，并把该钱包账户写入公民真实档案。
CPMS 不生成钱包签名 challenge，不保存钱包签名，不验证用户钱包私钥控制权。

钱包私钥控制权验证统一放在 SFID 绑定阶段：SFID 扫描 CPMS 出具的档案码后，要求 wuminapp 对 SFID
绑定 challenge 签名，并校验签名公钥等于档案码中的 `wallet_pubkey`。

## 2. 档案字段

`archives` 增加:

| 字段 | 含义 |
|---|---|
| `wallet_address` | 用户钱包 SS58 地址 |
| `wallet_pubkey` | 从 SS58 地址解析出的 32 字节公钥,`0x` hex |
| `wallet_sig_alg` | 固定 `sr25519` |
| `wallet_bound_at` | 钱包地址保存/更新时间 |
| `wallet_bound_by` | 执行保存的钱包管理员 user_id |

## 3. CPMS 保存流程

```text
创建档案
→ 用户在 wuminapp 电子护照页选择钱包
→ wuminapp 展示钱包地址二维码（WUMIN_QR_V1 / user_contact）
→ CPMS 档案详情页点击“钱包账户”后的绑定/更新按钮
→ CPMS 扫描钱包地址二维码
→ CPMS 从 SS58 地址解析 wallet_pubkey
→ CPMS 保存 wallet_address / wallet_pubkey
→ 生成 ARCHIVE 档案码
```

CPMS 只接受能解析出 AccountId32 的钱包地址；解析失败返回 `invalid wallet_address`。

## 4. ARCHIVE 字段

`ARCHIVE` 在原字段基础上增加:

```json
{
  "wallet_address": "5...",
  "wallet_pubkey": "0x...",
  "wallet_sig_alg": "sr25519"
}
```

CPMS 签名原文:

```text
sfid-cpms-v1|archive|{ano}|{cs}|{valid_from}|{valid_until}|{cpms_pubkey}|{geo_seal_hash}|{wallet_address}|{wallet_pubkey}
```

无钱包地址时，CPMS 不允许生成 ARCHIVE。
