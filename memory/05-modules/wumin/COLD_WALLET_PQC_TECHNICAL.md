# wumin 冷钱包 PQC 抗量子迁移技术设计

- 状态:设计 / 待实现(目标态,代码尚未落地)
- 关联决策:`memory/04-decisions/ADR-016-account-key-pqc-migration.md`
- 关联实现蓝图:`memory/05-modules/citizenchain/runtime/otherpallet/ACCOUNT_KEYS_PQC_TECHNICAL.md`
- 角色边界:`memory/05-modules/wuminapp-vs-wumin.md`

## 0. 冷钱包定位回顾

`wumin` 是**完全离线**的软件硬件钱包,与 `wuminapp`(热钱包)无任何 Dart 包依赖,只通过二维码对扫交互。冷钱包负责离线签名(`login_receipt` / `sign_response` 生成)。PQC 迁移后,冷钱包必须仍能**纯离线**完成 ML-DSA-65 签名,不依赖任何在线服务。

## 1. 同源派生(与热钱包强制一致)

冷热钱包必须用**同一套派生规则**(铁律:`memory/...feedback_*` 冷热同源)。两端共用 Rust 共享 crate `gmb-pqc`:

```
助记词
  → AccountRootSeedV1(= 现有 mini-secret, 不变 → sr25519 地址不变)
  ├─ sr25519   = HKDF-SHA512(root, "GMB/sr25519/v1")
  └─ ML-DSA-65 = HKDF-SHA512(root, "GMB/ML-DSA-65/v1")
```

- `gmb-pqc` 提供 `derive_ml_dsa65(root_seed) / sign / verify`,经原生 C FFI 暴露给 wumin 的 Flutter 层。
- 派生口径(HKDF domain label、challenge 拼装格式)与链上 `primitives::pqc` 共享常量,保证三端(链/热/冷)一致。

## 2. 纯助记词冷用户的自然过渡

ML-DSA-65 私钥由 `AccountRootSeedV1` 确定性派生,**过渡无需任何新秘密**。即便用户当初只抄了助记词(冷钱包典型场景):

1. 重新导入助记词 → 派生出 `AccountRootSeedV1` → 派生出 ML-DSA-65 钥匙。
2. 离线生成绑定签名(对 `bind_pqc_key` 的 challenge 做 ML-DSA-65 自签)。
3. 通过二维码把绑定/交易签名交给热端广播。

无需冷钱包提前预设 PQC 账户,无需更换助记词或账户。

## 3. 离线签名流程

- `sign_request`(接收):热端构造的交易(含 `pqc_dispatch` 的 payload)展示给冷钱包扫;冷钱包解码、展示中文摘要、用 ML-DSA-65 离线签名。
- `sign_response`(生成):签完生成二维码给热端扫回广播。
- 绑定 `bind_pqc_key`:外层 sr25519 + 内层 ML-DSA-65 自签,两签都可在冷钱包离线完成。
- 冷钱包不依赖在线 Passkey(Passkey 本轮不纳入,且永不作为冷钱包签名前置条件)。

## 4. 二维码协议扩展

- `wumin/lib/signer/` 的解码/签名路径扩展 `sig_alg` 为枚举 `sr25519 | ml-dsa-65`(对应热端 `qr/bodies/*` 放开硬编码)。
- `payload_decoder` 增加 `AccountKeys.bind_pqc_key` / `AccountKeys.pqc_dispatch` 的解码分支,展示验真字段必须与链上 call 字面一致(遵循 `unified-protocols.md` P-QR-002 规则)。
- 协议一致性通过 `memory/01-architecture/qr/qr-protocol-spec.md` + fixtures 强制对齐,冷热两端各自实现、不共享 Dart 代码。

## 5. 安全边界

- 冷钱包离线持有助记词与派生密钥,`AccountRootSeedV1` 与 PQC 私钥不出本机、不进二维码。
- 二维码只承载待签 payload 与已签名,不承载任何私钥材料。
- 签名 payload 必须带链域隔离(`genesis_hash`),防跨链/跨 call 重放。

## 6. 待实现清单

- wumin Rust FFI 接入 `gmb-pqc`(`ml_dsa65_pubkey_from_seed / ml_dsa65_sign`)。
- `signer` 派生/签名/解码路径支持 ML-DSA-65。
- QR fixtures 增加 ML-DSA-65 的 `sign_request/sign_response/bind_pqc_key` 样例,冷热两端对齐测试。
