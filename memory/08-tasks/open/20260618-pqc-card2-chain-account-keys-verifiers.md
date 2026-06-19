# PQC card2:链端 account-keys pallet + 验签器 ML-DSA + seal

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§2 §4 §5)
状态:open(依赖 card1 `gmb-pqc`)

任务需求:
链端落地 PQC-native 目标态(**无迁移机器,创世直接定义目标类型**):
1. **新增 `account-keys` pallet(idx=27)**:存 `BoundPqcKey`(ML-DSA-65 + ML-KEM-768 公钥绑定 sr25519 锚点 AccountId)、`AccountKeyNonce`;`pqc_dispatch`(general-tx + `#[pallet::authorize]` ML-DSA 验签,公钥从 state 读)。**无状态机、无 hybrid 双签、无 RejectSr25519**(账户出生即 PQC-native)。注册 `lib.rs:280-383`,`spec_version`(本链带 PQC 的创世起)。
2. **5 个 SFID/机构验签器 algo-tag 路由**:`configs/mod.rs:781/890/961/1037` + organization-manage 机构注册器,`sr25519_verify` → `gmb-pqc::verify_by_algo`(0x02 ML-DSA-65)。
3. **省级签名 storage 创世即 BoundedVec**:`sfid-system/src/lib.rs:245-271` `ShengSigningPubkey`/`ShengAdmins` 由 `[u8;32]`/`[u8;64]` 直接定义为 `BoundedVec`(容 ML-DSA 公钥 ~1952B / 签名 ~3309B)。**无 migration**(无数据)。
4. **offchain L3/批量验签**:`offchain-transaction settlement.rs:172` / `lib.rs:649` 改 ML-DSA;废弃 `sr25519_pubkey_from_account`(`lib.rs:655-663`)改查 account-keys;`MaxBatchSignatureLength`(`configs/mod.rs:1236` ConstU32<128>)放宽。
5. **seal 共识签名 ML-DSA-65**:`node/src/core/service.rs:276-287`(签)+ `93-143`(`SimplePow::verify`)+ `162-166`(pre_digest 公钥)改 ML-DSA;跟 PQC 二进制走、在**本链**带 PQC 的那次创世落地(开发期无数据、本就全网重装二进制,无硬分叉、非新链)。`blake2_256` PoW 工作量不动。

所属模块:Blockchain(runtime + node 共识)

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md(P-TX-009 pqc_dispatch / P-STORAGE-005 BoundPqcKey 登记)
- citizenchain 模块完成标准

必须遵守:
- **绝不扩 MultiSignature**(C1):账户 PQC 只走 pqc_dispatch general-tx。
- AccountId 永远 = sr25519 锚点 32B,不改类型(四不变)。
- 无迁移 / 无兼容 / 无双轨;创世直接目标态。
- domain 常量 `[u8;N]`;签名长度守卫按 ML-DSA 尺寸。
- DB / storage 列宽核对 ML-DSA 大密钥。

输出物:
- account-keys pallet + 验签器改造 + node seal + 中文注释 + 单测(`src/tests/{mod,cases}.rs`:authorize 成功/拒绝、nonce 防重放、5 验签器分流、seal 验签)+ benchmark + 文档

验收标准:
- pqc_dispatch ML-DSA 授权通过;5 验签器按 algo tag 正确分流;`ShengSigningPubkey` BoundedVec 容纳 ML-DSA。
- seal=ML-DSA-65 出块 / 验块通过(本链带 PQC 创世后)。
- 残留 sr25519 验签 / `sr25519_pubkey_from_account` 假设清零。
- 全 pallet 单测绿 + benchmark weight;真实运行态出块验收。
