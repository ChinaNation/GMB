# ADR-016 账户密钥与 sr25519→ML-DSA-65 抗量子迁移

- 状态:Accepted target-state
- 决议日期:2026-06-07
- 关联任务卡:`memory/08-tasks/open/20260607-wallet-pqc-passkey.md`
- 关联设计:`memory/05-modules/citizenchain/runtime/otherpallet/ACCOUNT_KEYS_PQC_TECHNICAL.md`

## 决议

GMB 全系统签名从 sr25519 平滑迁移到后量子签名 NIST **ML-DSA-65**(FIPS 204,Category 3)。迁移遵循"四不变":**不换助记词、不换账户、不换地址、不换余额**。前期账户仅用 sr25519;后期按账户切换到 PQC 唯一签名;终态可继续升级到 ML-DSA-87 乃至换族,仍不换账户。

启动算法标签 `algo = 0x02`(ML-DSA-65)。

## 派生:同一助记词,确定性派生 PQC 钥匙

用户只持有一套助记词。助记词恢复 `AccountRootSeedV1`(32 字节),即现有 `CryptoScheme.miniSecretFromEntropy` 的输出,语义升格、派生结果不变。所有签名钥匙从该根种子派生:

```
AccountRootSeedV1(永不变, 与签名算法无关)
  ├─ sr25519   = HKDF-SHA512(root, "GMB/sr25519/v1")    → 账户地址(永久锚点, 比特级不变)
  ├─ ML-DSA-65 = HKDF-SHA512(root, "GMB/ML-DSA-65/v1")  → 启动即用的 PQC 钥匙
  └─ ML-DSA-87 = HKDF-SHA512(root, "GMB/ML-DSA-87/v1")  → 将来升级, 同根同助记词
```

PQC 私钥由根种子确定性派生,过渡**不需要任何新秘密、新助记词**;纯助记词冷用户重新导入即可派生 PQC 钥匙完成绑定。冷热钱包共用同一派生规则。

## 账户身份:账户抽象,canonical AccountId 永远是 sr25519 派生值

链上 `AccountId = sr25519 公钥原样 32 字节`。ML-DSA 公钥 1952 字节无法塞进 32 字节。因此:

- **不扩展 `MultiSignature`**:给签名枚举加 ML-DSA 变体会让 `IdentifyAccount` 把 PQC 公钥 hash 成一个新的 32 字节 AccountId,等于新地址、第二份余额,违反地址不变。
- **采用账户抽象**:链上记录"canonical AccountId → 绑定的 PQC 公钥",PQC 交易在授权阶段验签后,以原 sr25519 派生的 canonical AccountId 身份执行。sr25519 与 ML-DSA-65 是同一账户主体下的两把签名凭证,不产生第二份余额、第二个地址。

## 账户状态机(每账户独立,无需全网同一天切换)

```
Sr25519Only ── 创建即此态, 前期只用 sr25519
   │  发一笔 hybrid 双签 bind_pqc_key
   ▼
Bound       ── sr25519 与 ML-DSA-65 并存, 两条通道解析到同一 canonical 账户
   │  切换(用户自愿 opt-in 或治理强制)
   ▼
PqcOnly     ── 本账户拒绝 sr25519, 只认 ML-DSA-65
```

三态下 canonical AccountId 恒为原 sr25519 派生值,余额/nonce/历史零变化。

## 绑定:hybrid 双签

`bind_pqc_key` 必须双签:外层由 sr25519 正常签名(证明现账户主人),call 内 `pqc_self_sig` 是 PQC 私钥对 `challenge = blake2_256(canonical_account ++ pqc_pubkey ++ account_key_nonce ++ genesis_hash)` 的签名(证明掌握 PQC 私钥)。两签皆过才写绑定。杜绝把他人公钥绑到自己账户或反向冒绑。

## 算法版本标签:可滚动升级

绑定记录带 `algo: u8` 版本标签;升级 = 从同一根种子派生新算法密钥,发新 `bind_pqc_key` 绑到同一 canonical 账户(当前活跃钥匙授权)。算法不是永久 ABI 锁,而是可滚动升级的凭证版本。

标签空间:`0x01=ML-DSA-44`(保留)、`0x02=ML-DSA-65`(启动)、`0x03=ML-DSA-87`、`0x10+` 预留其它族。ML-DSA-44/65/87 是 FIPS 204 同一算法的三档参数(模块维度 (4,4)/(6,5)/(8,7),NIST Category 2/3/5),`fips204` crate 同时实现,升级是同 crate 换参数,非换实现。

## 链上只存公钥 hash

`BoundPqcKey` 只存 `[u8; 32]` 公钥 hash + algo/state/nonce;日常 PQC 交易携带完整公钥(1952 字节)+ 签名(3309 字节),授权时 `blake2_256(pubkey) == stored_hash` 再验签。更大的 PQC 公钥只增加临时交易体积,不增加永久 state。

## 终态语义

PQC-only 后,那 32 字节地址不再是"活的签名公钥",退化为**永久账户标识符**(数值上等于旧 sr25519 公钥),实际授权全靠绑定的 PQC 钥匙。攻击者即便将来算出 sr25519 私钥也无效——链不再认该账户的 sr25519 签名。地址从头到尾不变,钱一直在原账户。PqcOnly 必须赶在量子计算机具备实战能力之前完成(harvest-now-decrypt-later 窗口)。

## 安全原则

- 助记词是最终恢复根;`AccountRootSeedV1` 不上传服务器。
- SFID 不保存钱包助记词、Root Seed、私钥或 PQC 私钥。
- 冷钱包必须保持离线签名能力。
- 验签技术选型:`fips204` crate(纯 Rust、`#[no_std]`、无堆、常量时间);RustCrypto `ml-dsa` 未审计、有过验签漏洞,不用。
- 默认不 fork `sp-io` 加 host function(host function 要求全节点二进制同步升级,与"chainspec 创世冻结、只 setCode"运维模型冲突);WASM 内验签,留待 benchmark 数据驱动决定。

## 模块契约

- 新增 `account-keys` pallet(pallet_index = 27),承载 `BoundPqcKey` / `AccountKeyNonce` storage、`bind_pqc_key`、`pqc_dispatch`。
- PQC 交易走 general-transaction + `#[pallet::authorize]`,不改全局 `TxExtension` 元组。
- PQC-only 收紧由 `RejectSr25519WhenPqcOnly` TransactionExtension 承担(Phase 3 接入)。
- `offchain-transaction` 批签由 algo 标签分流,`MaxBatchSignatureLength` 扩到 4736。
- QR 协议 `sig_alg` 由写死 `sr25519` 扩为枚举 `sr25519 | ml-dsa-65`。
- 冷热钱包共用 `gmb-pqc` Rust crate 做派生/签名/验签,派生规则强制一致。

## 备选方案(否决)

- **扩展 `MultiSignature` 加 ML-DSA 变体**:破坏地址不变(PQC 公钥 hash 成新 AccountId),类型级不可调和,否决。
- **Polkadot FRI-SNARK proof-of-seed 迁移**:对 GMB 过度设计(SNARK 工程量极大,证明体积 ~100KB),否决。
- **Falcon**:签名更小(666 字节),但无标准常量时间实现、依赖浮点,钱包侧侧信道风险高,Rust no_std 生态不成熟,否决。
- **ML-DSA-44**:更小(签名 2420 字节)但仅 Category 2;启动不选,标签 `0x01` 保留可回退。
- **ML-DSA-87**:Category 5,体积最大(签名 4627 字节);本期不选,标签 `0x03` 预留升级。
- **链上存完整 PQC 公钥**:永久 state 膨胀约 40 倍,否决,改存公钥 hash。

## 后续动作

- 实现分 Phase 0(链上 0 行为变化,新 pallet 骨架 + 共享 crate + 钱包 FFI)→ Phase 1(hybrid `pqc_dispatch`)→ Phase 2(offchain 批签 + QR 协议)→ Phase 3(PQC-only 收紧)。各阶段单独建任务卡、bump `spec_version`、走链上 setCode。
- **Passkey 本轮不纳入**,作为独立后续立项(热钱包本机解锁/高危确认层,非抗量子手段,不抢助记词根地位;含 WebAuthn PRF、恢复流程评估)。
- 待验:`fips204` 是否暴露 seed-based 确定性 keygen;若否,用 HKDF 输出喂确定性 RNG。
- 待定(实现期):general-tx 手续费向 canonical 账户计费的具体落点;热钱包是否只保存加密后的 `AccountRootSeedV1`;切 PqcOnly 的全网治理截止策略。
