# PQC card3:钱包同源派生(sr25519 直接) + ML-DSA 签名 + bootstrap + QR 分片

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§2/§7/§8)
状态:open(依赖 card1 gmb-pqc、card2 链端 pqc_dispatch/bootstrap)

任务需求:
冷热钱包(wuminapp/wumin)在位升级 PQC 签名,**地址逐字节不变**:
1. **同源派生(关键:sr25519 不套 HKDF)**:`AccountSeedV1` = 现有 `miniSecretFromEntropy` 输出(`wallet_manager.dart:541`(热)/`:398`(冷))。
   - **sr25519 地址锚点**:沿用现有 `sr25519.fromSeed(AccountSeedV1)` **直接派生,不经 HKDF**(否则 `HKDF(seed)≠seed`→地址变,破"不换地址")。
   - **ML-DSA-65 签名**:`HKDF-SHA512(AccountSeedV1, "GMB/account/ml-dsa-65/v1")` → 确定性 keygen。
   - **ML-KEM-768 加密**:`HKDF-SHA512(AccountSeedV1, "GMB/account/ml-kem-768/v1")`。
   - 冷热**逐字一致**;`wallet_manager.dart:541-555`(热)/`:398-417`(冷)。
2. **Rust FFI(gmb-pqc)**:`ml_dsa65_public_from_seed`、`ml_dsa65_sign`。
3. **查 runtime 签名策略分流**:`Sr25519Only`→普通 sr25519 extrinsic;`PqcPrepared/PqcPrimary`→`pqc_dispatch`;**账户未绑定**→`bootstrap_pqc_dispatch`(同一确认动作同时出 sr25519 bootstrap 签名 + ML-DSA 交易签名)。
4. extrinsic 走 pqc_dispatch:`signed_extrinsic_builder.dart:103/186`(原 `SignatureType.sr25519`,PQC 路径不再走 MultiSignature)。
5. **QR 升级**:`sig_alg(sr25519|ml-dsa-65)` + `auth_mode(normal|pqc|bootstrap-pqc)` + `key_version` + `payload_hash` + **`chunk_index/chunk_total` 分片**。🔴 最坏体积按 bootstrap(sr25519 64B + ML-DSA ~3.3KB)实测,不能假设单张 QR 稳定可扫;改 `sign_request_body.dart:40` + `sign_response_body.dart:36` + `qr_signer.dart:118/:134-151` + wuminapp 镜像(四处+,漏一处冷热口径裂)。
6. UI **不展示** PQC 公钥/绑定过程/换账户;交易记录按原地址归集;同助记词恢复同地址 + 同 PQC 密钥(确定性)。

所属模块:Mobile(wuminapp 热钱包 + wumin 冷钱包)

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md(QR sig_alg/auth_mode/分片 + pqc_dispatch/bootstrap)
- wuminapp/wumin 模块完成标准

必须遵守:
- `AccountSeedV1` 不变;**sr25519 分支绝不套 HKDF**(除非 golden vector 证地址逐字节一致)。
- 冷热派生逐字一致;QR sig_alg/auth_mode 全处同步零遗漏。
- 冷钱包保持离线签名(含 bootstrap 离线扫码)。
- UI 不暴露多公钥/多算法概念。

输出物:
- 双端派生(sr25519 直接 + ML-DSA/ML-KEM HKDF)/ FFI / pqc_dispatch+bootstrap 构造 / QR 分片 + 中文注释 + 测试(同源派生 golden vector / bootstrap 往返 / QR 分片)+ 文档

验收标准:
- 同助记词冷热恢复同一 AccountId,sr25519 地址**逐字节不变**;ML-DSA 签名被链端 pqc_dispatch 验通过。
- 未绑定账户首次 bootstrap 扫码(含分片)成功;后续 ML-DSA 交易成功。
- QR `sig_alg`/`auth_mode`/分片冷热往返 OK,真机扫码验收;残留 sr25519 硬编码按域收敛。
