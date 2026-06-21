# citizenwallet 公民钱包 PQC 抗量子签名升级技术设计

- 状态:设计 / 待实现。**真源 = `memory/04-decisions/ADR-022-unified-pqc-crypto.md`**(取代旧 PQC 迁移方案)。
- 任务卡:`memory/08-tasks/open/20260618-pqc-card3-wallet-derivation-signing.md`。

## 要点(以 ADR-022 为准)

- `citizenwallet` 完全离线,PQC 后仍须纯离线完成 ML-DSA-65 签名,不依赖在线服务。
- **同源派生(sr25519 不套 HKDF)**:`AccountSeedV1` = 现有 mini-secret;sr25519 地址锚点沿用现有 `sr25519.fromSeed(AccountSeedV1)` **直接派生**(不经 HKDF,否则地址变);ML-DSA-65/ML-KEM-768 才用 `HKDF-SHA512(AccountSeedV1, "GMB/account/ml-dsa-65/v1" | ".../ml-kem-768/v1")`。冷热共用 `gmb-pqc` crate(原生 C FFI)。
- **无感 bootstrap**:未绑定账户首次扫码,**同一确认**同时出 sr25519 bootstrap 签名(证旧地址主人)+ ML-DSA 交易签名;后续只签 ML-DSA。**无单独绑定步骤、无账户状态切换**。
- **QR 扩展**:`sig_alg(sr25519|ml-dsa-65)` + `auth_mode(normal|pqc|bootstrap-pqc)` + `key_version` + `chunk_index/chunk_total`(ML-DSA ~3.3KB,最坏体积按 bootstrap 实测分片)。
- **安全**:`AccountSeedV1`/私钥不出本机、不进二维码;payload 带 `genesis_hash`+`spec_version` 防跨链/跨升级重放;bootstrap 强度=sr25519,窗口须在量子破 sr25519 前关闭。

> 实现以本文 + ADR-022 为准,旧路线不再适用。
