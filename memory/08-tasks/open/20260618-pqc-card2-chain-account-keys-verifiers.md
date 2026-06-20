# PQC card2:链端 GmbPqcAuth 扩展授权 + account-keys(存储/策略) + bootstrap + 验签器 + seal

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§3/§4/§5/§6)
状态:open(依赖 card1 gmb-pqc + spike 确认 GmbPqcAuth 机制)

任务需求:
**活链 runtime 升级**在位支持 PQC 签名(地址/余额/权限/治理身份不变),路线 A 定稿:
1. **链上授权 = General Transaction + 自定义 `GmbPqcAuth` TransactionExtension(无 pqc_dispatch pallet call 主路径)**:
   - `GmbPqcAuth` 放扩展流水线**最前**:验 ML-DSA-65 → 由 `validate` 返回 `Signed(account)` origin → 后续 `CheckNonce`/`ChargeTransactionPayment` 走系统标准逻辑(不自管 nonce/扣费);`prepare` 只做复核/预处理,不负责改 origin。
   - 🔴 **tuple 12 上限**:`TxExtension` 已 12 项(`lib.rs:164-176`,SDK 上限 12)→ 用**嵌套 `(GmbPqcAuth, AuthorizeCall)` 占第一项槽位**,**不加第 13 项**。
   - **PQC proof 放扩展 `extra`**(非 call):ML-DSA 签名/公钥(bootstrap)/`auth_mode`/`key_version`;**签名 preimage 排除签名字节**;校验 `call_hash==blake2_256(body.call)`,bootstrap 再 `pqc_pubkey_hash==blake2_256(body.pqc_pubkey)`。
   - 🔴 **PQC 签名消息必须绑定实际后续扩展**:按当前 SDK `inherited_implication` 口径生成 `following_extensions_hash = blake2_256(SCALE(extension_version, call, following_extensions, following_implicits))`;proof 内 `nonce`/`tip`/`era` 若重复出现,必须逐字段等于实际 `CheckNonce`/`ChargeTransactionPayment`/`CheckEra`。必须证明**嵌套 tuple 下钱包按 metadata 重建的 hash 与链端 `GmbPqcAuth` 重算值逐字节一致**。
   - 🔴 **权重按 `extra` 分支**:`None` 近零并透传;`Pqc` 计 `AccountPqcKey` 读取 + ML-DSA 验签 + hash 校验;`Bootstrap` 计 hash + sr25519 bootstrap 验签 + ML-DSA 验签 + `post_dispatch` 写入上限。
   - 🔴 **validate 廉价检查先于 ML-DSA**:先查 `extra/origin` 模式、`PqcPolicy`、`AccountPqcKey`、`alg/key_version/pubkey`、`call_hash`、`following_extensions_hash`、显式 `nonce/tip/era` 一致性、nonce 基本窗口,再执行 ML-DSA 验签;node/txpool 可按 `(account, source)` 做限速。
   - **`GmbPqcAuth` 同时负责"已绑定拒 sr25519"**(读 `AccountPqcKey`+`PqcPolicy.reject_sr25519_when_bound`),**不单独加扩展**(同扩展不误伤自己授权的 PQC 交易)。
   - 🔴 **`extra` 必有 `None` 变体**(`{None|Pqc|Bootstrap}`):Phase A/B 普通 sr25519 交易也过此扩展;`extra=None` 时**透明放行**给 `AuthorizeCall`(非 PQC 的 authorized general call 不被误伤),仅对 `extra=None` 的 sr25519 signed origin 才判 `reject_sr25519_when_bound`。`extra=Pqc|Bootstrap` 只允许 General Transaction 起点;sr25519 signed extrinsic 携带 PQC proof 直接拒绝。🔴 注:加 `GmbPqcAuth` 改了 signed-extra/metadata 格式,未适配新 metadata 的旧客户端本就发不了交易(非误拒);"不误拒"指仍用 sr25519 但已适配新 extra(`None`)的客户端。
2. **`account-keys` pallet(idx=27)只承载 存储/策略/查询/事件/轮换,不承载主交易派发**:
   - `AccountPqcKey[AccountId]{alg:0x02,key_version,pubkey:BoundedVec(完整~1952B),bound_at,bootstrap_mode}`。
   - `PqcPolicy{phase,bootstrap_deadline:Option<BlockNumber>,reject_sr25519_when_bound,allow_bootstrap_unbound}`,治理写、客户端读;🔴 **首次升级默认安全**:`phase=B`/`reject_sr25519_when_bound=false`/`allow_bootstrap_unbound=true`/`deadline=None`。
   - 密钥轮换 call:当前有效 PQC 私钥授权 + `key_version++`。
3. **bootstrap 首笔(无感)**:`GmbPqcAuth.validate` 验序 ① `blake2_256(body.pqc_pubkey)==payload.pqc_pubkey_hash` → ② sr25519 bootstrap challenge(证旧地址主人)→ ③ ML-DSA 交易签名 → 返回 `Signed(account)` origin → nonce/扣费/业务 dispatch;🔴 **绑定写入 `AccountPqcKey` 放 `post_dispatch`**(nonce/扣费已跑),**即使内层业务 call 失败绑定仍保留、内层失败照常收费**;写入前必须检查仍未绑定/同值幂等,不同公钥或版本不得覆盖(first-bind-wins)。
4. **payload 反重放**:`GMB_PQC_TX_V1` / `GMB_PQC_BOOTSTRAP_V1`(+spec_version + `following_extensions_hash`);🔴 **hash 全 `blake2_256`**;🔴 **era 默认 immortal**(链域靠 genesis_hash,不带 checkpoint;若 mortal 则必含 checkpoint block hash,二选一不混用);显式 `nonce/tip/era/genesis/spec` 与 `following_extensions_hash` 是有意冗余,用于钱包展示/离线确认/跨端 fixture,必须逐字段一致;txpool `provides=(account,nonce)`。
5. **5 个 SFID/机构验签器 algo-tag 路由**(`configs/mod.rs:781/890/961/1037` + organization-manage)→ `verify_by_algo`。
6. 省级签名库 `ShengSigningPubkey`/`ShengAdmins` `[u8;32]`→`BoundedVec`。
7. offchain L3/批量(`settlement.rs:172`/`lib.rs:649`):废弃 `sr25519_pubkey_from_account` 改查 account-keys;`MaxBatchSignatureLength` 放宽。
8. **seal 共识签名 ML-DSA-65**(`service.rs:276-287`/`93-143`/`162-166`)随 PQC 二进制本链落地;`blake2_256` PoW 不动。

所属模块:Blockchain(runtime + node 共识)

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md(GmbPqcAuth / bootstrap / AccountPqcKey / PqcPolicy 登记)
- citizenchain 模块完成标准

必须遵守:
- 绝不扩 MultiSignature;AccountId 永远 sr25519 锚点(四不变)。
- 授权唯一走 `GmbPqcAuth` 扩展;account-keys **不承载主交易派发**。
- bootstrap 只用于未绑定账户首次进 PQC,非长期双轨;绑定写 `post_dispatch`。
- 嵌套 tuple 不超 12 项;hash 全 blake2_256;PqcPolicy 安全默认。
- DB/storage 列宽核对 ML-DSA 大密钥。
- **依赖 card1 spike 确认 GmbPqcAuth 机制**(origin 转换 + 嵌套 tuple + post_dispatch)在当前 SDK 可行。

输出物:
- `GmbPqcAuth` 扩展 + account-keys(storage/policy/rotation)+ 验签器改造 + node seal + 中文注释 + 单测(bootstrap 验序成功/拒绝、已绑定拒覆盖、拒 sr25519 不误伤 PQC、origin 转换+nonce/扣费、嵌套 tuple 下 following_extensions_hash 与钱包 metadata 重建逐字节一致、weight 分支、廉价检查先于 ML-DSA、following_extensions_hash 不一致拒绝、sr25519 signed 携带 PQC proof 拒绝、post_dispatch 幂等绑定、5 验签器、seal)+ benchmark + 文档

验收标准:
- 未绑定首次 bootstrap+execute 成功(绑定 `post_dispatch`,内层失败绑定仍留);已绑定后续 PQC 成功;已绑定 sr25519 用户交易被 `GmbPqcAuth` 拒、PQC 交易不误伤。
- 嵌套 tuple `(GmbPqcAuth, AuthorizeCall)` 编译+按序执行;`CheckNonce`/`ChargeTransactionPayment` 真实生效。
- 5 验签器 algo-tag 分流;`ShengSigningPubkey` 容纳 ML-DSA;seal=ML-DSA-65 出块/验块通过。
- 残留 sr25519 验签/`sr25519_pubkey_from_account` 假设清零;全 pallet 单测绿+benchmark;真实运行态出块验收。
