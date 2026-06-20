# PQC card2:链端 account-keys + pqc_dispatch + bootstrap 绑定 + 验签器 + seal

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§3/§4/§5/§6)
状态:open(依赖 card1 `gmb-pqc`)

任务需求:
**活链 runtime 升级**在位支持 PQC 签名,账户地址/余额/权限/治理身份不变:
1. **新增 `account-keys` pallet(idx=27)**:
   - storage `AccountPqcKey[AccountId] = {alg:0x02, key_version:u32, pubkey:BoundedVec(~1952B), bound_at, bootstrap_mode}`(存完整公钥)。
   - **`pqc_dispatch`**(general-tx + `#[pallet::authorize]`):从 `AccountPqcKey` 按 AccountId 读 ML-DSA 公钥 → `verify_ml_dsa_65` → 以 `RawOrigin::Signed(account)` 派发内层 call;**费用从 account 扣;nonce 用 `frame_system::AccountInfo.nonce`**(不另造)。
   - **`bootstrap_pqc_dispatch`**(未绑定账户首次,无感桥):验 ① sr25519 bootstrap challenge(证旧地址主人)+ ② ML-DSA 交易签名 → 写 `AccountPqcKey` → **立即派发内层 call**;已绑定**拒绝**再次 sr25519 覆盖(first-bind-wins)。
   - **绝不扩 `MultiSignature`**(C1)。
2. **payload 反重放域**:`GMB_PQC_TX_V1`(genesis/spec_version/tx_version/ss58/account/nonce/era/tip/call_hash/sig_alg/key_version/auth_mode)、`GMB_PQC_BOOTSTRAP_V1`(+ spec_version 防跨升级重放、pqc_pubkey_hash);txpool `provides=(account,nonce)`;`validate_unsigned`/authorize 轻量无副作用,写 storage+派发在执行阶段。
3. **阶段策略 A/B/C/D**:B 预埋(sr25519+PQC 并存)→ C(已绑定只收 ML-DSA,未绑定 bootstrap 后转)→ D 收紧(长期未绑定治理处理 + 已绑定拒 sr25519)。🔴 **bootstrap 窗口必须在 sr25519 被量子破前关闭**(治理硬截止;bootstrap 强度=sr25519)。
4. **5 个 SFID/机构验签器 algo-tag 路由**:`configs/mod.rs:781/890/961/1037` + organization-manage → `verify_by_algo`。
5. 省级签名库 `ShengSigningPubkey`/`ShengAdmins` `[u8;32]`→`BoundedVec`。
6. offchain L3/批量(`settlement.rs:172`/`lib.rs:649`):废弃 `sr25519_pubkey_from_account` 改查 account-keys;`MaxBatchSignatureLength` 放宽。
7. **seal 共识签名 ML-DSA-65**(`service.rs:276-287`/`93-143`/`162-166`)随 PQC 二进制本链落地;`blake2_256` PoW 不动。

所属模块:Blockchain(runtime + node 共识)

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md(pqc_dispatch / bootstrap / AccountPqcKey 协议登记)
- citizenchain 模块完成标准

必须遵守:
- 绝不扩 MultiSignature;AccountId 永远 sr25519 锚点(四不变)。
- bootstrap 只用于未绑定账户首次进 PQC,**非长期双轨**。
- **spike 前置**:authorize 须在收费前产 signed origin;`frame_system::AccountInfo.nonce` 在 general-tx authorize 复用是否可原子读/写且与 txpool 一致——不过就回退专用 nonce。
- DB/storage 列宽核对 ML-DSA 大密钥。

输出物:
- account-keys pallet(pqc_dispatch + bootstrap_pqc_dispatch)+ 验签器改造 + node seal + 中文注释 + 单测(`src/tests/{mod,cases}.rs`:bootstrap 双签成功/拒绝、已绑定拒覆盖、pqc_dispatch 授权、nonce 防重放、5 验签器分流、seal)+ benchmark + 文档

验收标准:
- 未绑定账户首次 **bootstrap + execute** 成功;已绑定后续 ML-DSA 成功;已绑定普通 sr25519 用户交易按策略被拒。
- 5 验签器 algo-tag 分流;`ShengSigningPubkey` 容纳 ML-DSA;seal=ML-DSA-65 出块/验块通过。
- 残留 sr25519 验签/`sr25519_pubkey_from_account` 假设清零;全 pallet 单测绿 + benchmark;真实运行态出块验收。
