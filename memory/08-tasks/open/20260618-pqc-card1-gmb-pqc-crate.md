# PQC card1:gmb-pqc 共享 crate + fips204 WASM spike

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§1 §4)
状态:open(**先行 spike,后续 card 依赖本卡**)

任务需求:
建立全系统单一抗量子原语基座 `gmb-pqc`,被 runtime(no_std WASM)+ sfid/cpms/node 后端 + 钱包 FFI 共用,杜绝第二套实现:
1. **HKDF-SHA512 派生规则单一源**:domain 标签表(`GMB/sr25519/v1` / `GMB/ML-DSA-65/v1` / `GMB/ML-KEM-768/v1` …),从 `AccountSeedV1` 派生各算法私钥种子(fips204 无 seed keygen → HKDF 输出喂确定性 RNG)。
2. **algo 常量**:`0x01` sr25519 / `0x02` ML-DSA-65 / `0x03` ML-DSA-87(预留)。
3. **domain 常量强制 `[u8;N]` 数组**(铁律 `feedback_scale_domain_must_be_array`):把现有违规的 `L3_PAY_SIGNING_DOMAIN`/`BATCH_SIGNING_DOMAIN`(`offchain-transaction/src/batch_item.rs:39/42` 等 `&[u8]`)迁入并改数组类型。
4. **`verify_by_algo` trait**:链端 ML-DSA 验签封装(fips204)。
5. **`fips204` WASM spike(必做)**:验证 no_std + 编进 runtime WASM + 体积 + 验签权重;若占区块预算显著比例,评估是否需 host function(fork `ChinaNation/ss58-2027-fix` 分支,**非纯 setCode**)。出 benchmark 真实 `WeightInfo`,禁用猜测值。

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
