# PQC card1:gmb-pqc 共享 crate + fips204 WASM spike

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§1 §4)
状态:open(**先行 spike,后续 card 依赖本卡**)

任务需求:
建立全系统单一抗量子原语基座 `gmb-pqc`,被 runtime(no_std WASM)+ sfid/cpms/node 后端 + 钱包 FFI 共用,杜绝第二套实现:
1. **HKDF-SHA512 派生规则单一源(精确口径,钉 golden vector)**:`PRK=HKDF-Extract(salt=空, IKM=AccountSeedV1[32B])`,域分离全靠 `info`(ASCII 无 null,`GMB/account/ml-dsa-65/v1` / `GMB/account/ml-kem-768/v1`);`HKDF-Expand` 输出 ML-DSA L=32(ξ→FIPS204 KeyGen_internal)、ML-KEM L=64(d‖z→FIPS203 KeyGen_internal);fips204 无 ξ-API 时用此 32B 种确定性 RNG。**sr25519 地址锚点不在此表**:保持现有 `fromSeed(AccountSeedV1)` 直接派生,**绝不套 HKDF**(否则 `HKDF(seed)≠seed`→地址变)。**golden vector**:固定助记词→{sr25519 地址, ML-DSA-65 公钥, ML-KEM-768 公钥} 作冷热/链/后端跨端基准。
2. **algo 常量**:`0x01` sr25519 / `0x02` ML-DSA-65 / `0x03` ML-DSA-87(预留)。
3. **domain 常量强制 `[u8;N]` 数组**(铁律 `feedback_scale_domain_must_be_array`):把现有违规的 `L3_PAY_SIGNING_DOMAIN`/`BATCH_SIGNING_DOMAIN`(`offchain-transaction/src/batch_item.rs:39/42` 等 `&[u8]`)迁入并改数组类型。
4. **`verify_by_algo` trait**:链端 ML-DSA 验签封装(fips204)。
5. **`fips204` WASM spike(必做)**:验证 no_std + 编进 runtime WASM + 体积 + 验签权重;若占区块预算显著比例,评估是否需 host function(fork `ChinaNation/ss58-2027-fix` 分支,**非纯 setCode**)。出 benchmark 真实 `WeightInfo`,禁用猜测值。
6. 🔴 **(硬闸门 spike,card2 前置)验证路线 A `GmbPqcAuth` 机制(路线已定 A,不二选一)**:① `GmbPqcAuth.validate` 把 General Transaction origin 转 `Signed(account)`,使其后 `CheckNonce`/`ChargeTransactionPayment` 正常(`AuthorizeCall` 默认给 `Authorized` 非 `Signed`,`lib.rs:164`),`prepare` 不负责改 origin;② **嵌套 tuple `(GmbPqcAuth, AuthorizeCall)` 作第一项能编译且按序执行**(outer 仍 12 项);③ 绑定写 `post_dispatch` 可行;④ txpool `provides=(account,nonce)`;⑤ PQC 签名消息绑定实际 `following_extensions_hash`,proof 内 `nonce`/`tip`/`era` 与实际后续扩展不一致时拒绝;⑥ `extra=Pqc|Bootstrap` 只允许 General Transaction 起点,sr25519 signed extrinsic 携带 PQC proof 直接拒绝。确认当前 SDK 可行;若 origin 转换受限,退而 `GmbPqcAuth` 内自管 nonce/扣费,**仍单一扩展授权,不回退 pallet call 主路径**。

所属模块:Blockchain(runtime/primitives + 共享 crate)

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md(协议登记)
- 链端模块完成标准 citizenchain.md

必须遵守:
- 派生规则冷热 / 前后端逐字一致(domain 标签 / 派生顺序),否则地址锚点漂移。
- domain 常量必须 `[u8;N]`,不得 `&[u8]`。
- 验签技术选型 fips204(no_std / 常量时间);不用 RustCrypto ml-dsa。
- 不破坏 chainspec 冻结模型(默认不加 host function,由 spike 数据驱动)。

输出物:
- `gmb-pqc` crate + 中文注释 + 单测(派生确定性 / algo 路由 / 跨端 fixture)+ benchmark + 文档(更新 unified-protocols 登记)

验收标准:
- 同 `AccountSeedV1` 在 Rust / Dart-FFI 派生出逐字节一致的三套密钥。
- `verify_by_algo` 单测通过;domain 常量全 `[u8;N]`,旧 `&[u8]` 残留清零。
- fips204 WASM spike 出结论文档(WASM 内 vs host function)+ 真实验签 weight。
