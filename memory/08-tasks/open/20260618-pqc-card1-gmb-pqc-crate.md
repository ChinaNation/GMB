# PQC card1:gmb-pqc 共享 crate + 全部 spike 闸门

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§2/§5/§11)
状态:open(**先行 spike,card2/card3 依赖本卡;spike 绿了才动**)

任务需求:
建立全系统单一抗量子原语基座 `gmb-pqc`(runtime no_std WASM + sfid/cpms/node 后端 + 钱包 FFI 共用):
1. **HKDF-SHA512 账户派生单一源(精确口径,钉 golden vector)**:`PRK=HKDF-Extract(salt=空, IKM=AccountSeedV1[32B])`,域分离全靠 `info`(ASCII 无 null、**含长度域**):**仅** `GMB/account/ml-dsa-65/seed32/v1` → `HKDF-Expand L=32` → 32B ξ → FIPS204 `KeyGen_internal(ξ)`。🔴 **账户不派生 ML-KEM**(决策3:机密性 KEM 在 IM/TLS 层,不进账户体系)。🔴 **sr25519 锚点不在此表**(现有 `fromSeed(AccountSeedV1)` 直接派生,绝不套 HKDF)。
2. 🔴 **(B8)锁库**:强制选用暴露 `KeyGen_internal(ξ)` seed-API 的 fips204 锁定版本,**删一切 DRBG fallback**(不暴露 ξ-API 则换库);库名+版本+API 名钉进本卡产出。
3. **algo 常量**:`0x01` sr25519 / `0x02` ML-DSA-65 / `0x03` ML-DSA-87(预留)。
4. **domain 常量强制 `[u8;N]`**(铁律 `feedback_scale_domain_must_be_array`):`DOMAIN_TX=b"GMB_PQC_TX_MLDSA65_V1"`、`DOMAIN_BOOTSTRAP=b"GMB_PQC_BOOTSTRAP_MLDSA65_V1"`(算法标识编进域字面量);并迁 `batch_item.rs:39/42` 的 `L3_PAY/BATCH_SIGNING_DOMAIN` `&[u8]`→`[u8;N]`。
5. **`verify_by_algo` trait**:链端 ML-DSA 验签封装(fips204),校验 payload `alg` 与 `AccountPqcKey.alg` 一致后路由。
6. **golden vector**:固定助记词 → AccountSeedV1 →(中间量 **ξ**)→ {sr25519 SS58, ML-DSA-65 公钥};版本化文件,Rust/wuminapp-FFI/wumin-FFI 三端对拍,库升级须重跑。

**🔴 spike 闸门(全绿才进 card2/card3):**
- S1 fips204 no_std + 编进 runtime WASM + **iOS/Android 移动端编译** + 体积 + 单次验签 weight;一个区块最多容多少次 PQC 验签。
- S2 **(B2)客户端 General Transaction 编码**:polkadart 0.7.1 只编 legacy 0x84、无 v5/extension_version/自定义 extra → fork/patch polkadart 或自写 v5 SCALE 编码器;验"Dart 产出的 v5 交易被链端 GmbPqcAuth 接受"。
- S3 **(B3)链端 GmbPqcAuth 机制**:`validate` 转 `Signed(account)`;嵌套 `(GmbPqcAuth, AuthorizeCall)` 编译+按序;**嵌套下 `inherited_implication` 真含 outer 全部后续扩展 implicit、Dart 重建 `following_extensions_hash` 与链端 `inherited_implication.encode()` 逐字节一致**(最高风险点,参照 mod.rs:712-869);post_dispatch 写绑定可行且幂等、绝不 Err。
- S4 **(H14)枚举当前 runtime 所有 `#[pallet::authorize]` call**,确认不成为已绑定账户绕过 PQC 强制的旁路。
- S5 QR 分片真机稳定(最坏 bootstrap ~10KB+)。
- S6 seal=ML-DSA 评估 → 归 **card7**(本卡只确认它是节点二进制非 setCode)。

所属模块:Blockchain(runtime/primitives + 共享 crate)

输入文档:ADR-022 / unified-protocols(协议登记)/ citizenchain 完成标准

必须遵守:
- 派生规则冷热/前后端逐字一致;domain 常量 `[u8;N]`;fips204 no_std 常量时间,不用 RustCrypto ml-dsa;不破坏 chainspec 冻结(默认不加 host function,由 spike 数据驱动)。

输出物:
- `gmb-pqc` crate + 锁库版本记录 + golden vector(含ξ)+ 中文注释 + 单测(派生确定性/algo 路由/跨端 fixture)+ benchmark + spike 结论文档

验收标准:
- 同 AccountSeedV1 在 Rust/Dart-FFI 派生逐字节一致的 sr25519+ML-DSA(golden vector 含 ξ 对拍)。
- S1-S5 全部出结论且通过(尤其 S2/S3:Dart 能产被链端接受的 v5 交易、嵌套 following_extensions_hash 逐字节一致);domain 常量全 `[u8;N]` 旧 `&[u8]` 清零。
