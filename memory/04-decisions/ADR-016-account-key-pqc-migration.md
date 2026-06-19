# ADR-016 账户密钥与 sr25519→ML-DSA-65 抗量子迁移

- 状态:**Superseded(2026-06-18)by [ADR-022](ADR-022-unified-pqc-crypto.md)**

## 作废说明

本 ADR(单独的"钱包签名域 PQC 迁移",带 sr25519→PQC 账户状态机 / hybrid 双签迁移机器)已被 **ADR-022 GMB 统一抗量子加密方案** 取代:

- 全系统无数据、上线前可重新创世 → **无迁移 / 无兼容 / 无双轨**,迁移机器(状态机 / hybrid / PqcOnly 收紧 / storage 迁移)全部删除,账户出生即 PQC-native。
- ADR-016 只管钱包签名,ADR-022 统一**签名(ML-DSA-65)+ 机密性(ML-KEM-768 混合)+ 对称(AES-256-GCM)+ KDF(HKDF/Argon2id)** 四原语,覆盖钱包 / 链 / SFID / CPMS / node / IM / 传输全域。
- 仍保留的正确结论(同源派生、AccountId=sr25519 锚点、四不变、pqc_dispatch 不碰 MultiSignature、fips204 选型)已并入 ADR-022。

**实现以 ADR-022 + `memory/08-tasks/open/20260618-pqc-card0..6` 为准。** 本文件仅留指针,勿据此实现。
