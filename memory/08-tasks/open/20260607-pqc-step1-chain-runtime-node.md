# PQC 抗量子迁移 Step 1:区块链(runtime + node)完整技术方案

- 状态:open(技术方案已定,代码未启动)
- 创建日期:2026-06-07
- 关联决策:`memory/04-decisions/ADR-016-account-key-pqc-migration.md`
- 关联实现蓝图:`memory/05-modules/citizenchain/runtime/otherpallet/ACCOUNT_KEYS_PQC_TECHNICAL.md`(详细伪代码真源)
- 上层关系:范围 A(用户钱包账户签名)分两步实现 —— **本卡 = Step 1 区块链(runtime + node)**;Step 2 = `20260607-pqc-step2-wallet-wuminapp-wumin.md`(wuminapp + wumin)。
- 范围外:PoW 出块 seal 共识签名(范围 C,需全节点硬升级,另议);SFID/CPMS 系统签名域(范围 B,`20260607-sfid-cpms-pqc-signing.md`)。

## 0. 目标与可独立验证性

让链端支持"同一账户主体绑定 ML-DSA-65 凭证、用 PQC 签名发起交易",并让 node 的两个本机热钱包(矿工热钱包 / 清算行结算签名器)用 ML-DSA-65 签名。

**关键:Step 1 自包含、可端到端验证**——node 自带矿工热钱包,能本机产出 ML-DSA-65 签名,因此整条 PQC 链路(派生→绑定→pqc_dispatch→验签→执行)在不依赖 wuminapp/wumin(Step 2)的情况下即可跑通。

## A. 共享底座

### A.1 新建 crate `gmb-pqc`(node 与 Step 2 钱包共用)
- `derive_ml_dsa65(root_seed: [u8;32]) -> (pubkey, secret)` = `HKDF-SHA512(root, "GMB/ML-DSA-65/v1") → 32B seed → fips204 ML-DSA-65 keygen`。
- `sign(secret, msg) -> sig`、`verify(pubkey, msg, sig) -> bool`。
- 依赖:`fips204`、`hkdf`(0.12.4 已在依赖树)、`sha2`。
- ⚠️ 待验:`fips204` 是否暴露 seed-based 确定性 keygen;若否,用 HKDF 输出喂确定性 RNG(必须确定性,保证同种子同密钥)。

### A.2 `primitives` 新增 `pqc.rs`(链端)
- 算法标签常量:`ALGO_ML_DSA_65 = 0x02`(标签空间见 ADR-016)。
- HKDF domain / challenge domain 常量(与 `gmb-pqc` 共享同一字面口径,避免链/节点不一致)。
- `BatchSignatureVerifier` trait(供 offchain-transaction 注入,见 §C)。
- `verify_ml_dsa_65(pubkey, msg, sig) -> bool`:`fips204` 纯 Rust、`#[no_std]`、WASM 内验签,封装风格照 `sfid-system::verify_sr25519`(`lib.rs:732-736`)。

## B. runtime:新 pallet `account-keys`(pallet_index = 27)

目录照搬 `sfid-system`。完整 storage / extrinsic 伪代码见实现蓝图 §2、§3。要点:

### B.1 storage
- `BoundPqcKey: StorageMap<AccountId, PqcKeyRecord>`(只存公钥 hash;`PqcKeyRecord{algo, pubkey_hash:[u8;32], state:Active/PqcOnly/Revoked, bound_at}`)。
- `AccountKeyNonce: StorageMap<AccountId, u32>`(PQC 交易防重放,general-tx 不走 `CheckNonce`)。

### B.2 `bind_pqc_key`(call_index 0,hybrid 双签)
外层 sr25519 签(现账户主人)+ 内层 ML-DSA-65 对 `challenge = blake2_256(who ++ pqc_pubkey ++ nonce ++ genesis)` 自签;两签过 → 写 `BoundPqcKey(state=Active)` + bump nonce。算法升级(65→87)走同一 extrinsic 换 `algo` 重绑。

### B.3 `pqc_dispatch`(call_index 1)+ `#[pallet::authorize]`
PQC 交易 = general-transaction(无外层 sr25519 签名)。authorize 读 `BoundPqcKey`、校验 `blake2_256(pubkey)==pubkey_hash`、`nonce` 匹配、`verify_ml_dsa_65(pubkey, blake2_256(call.encode()++nonce++genesis), sig)`;通过后以 `RawOrigin::Signed(account)` 派发内层 `call`,bump nonce。机制依据 SDK `authorize_call.rs:59-113`。

### B.4 手续费(关键风险,Phase 1 必须落定)
general-tx 无外层签名者,但 `ChargeTransactionPayment` 仍在 `TxExtension`(`lib.rs:174`),需向 canonical `account` 计费。两个候选:
- (a) `pqc_dispatch` 的 authorize/validate 把 origin 设为 `Signed(account)`(而非仅 `Authorized`),让标准 `ChargeTransactionPayment` 自动向该账户收费;
- (b) 自定义支付逻辑/扩展,按 authorize 得到的 account 计费。
落定前先做 spike 验证本 fork 的 general-tx + ChargeTransactionPayment 组合行为。

### B.5 接线
- `lib.rs:284-390` 注册 `#[runtime::pallet_index(27)] pub type AccountKeys = account_keys;`。
- `configs/mod.rs` 加 `impl account_keys::Config for Runtime`(照 sfid-system `:961-973`)。
- `runtime/Cargo.toml` 加 `account-keys` + `fips204`(`default-features=false`)依赖,接入 std / runtime-benchmarks / try-runtime。
- bump `spec_version`(`lib.rs:83`)。

### B.6 weights / benchmarks / tests
- benchmark 出 `verify_ml_dsa_65` 真实 weight(禁猜测值);若占区块预算显著(>5–10%)再评估范围 C 式 host function。
- 单测入 `src/tests/{mod.rs,cases.rs}`:bind 双签成功/失败、nonce 防重放、algo 升级重绑、pqc_dispatch 授权成功/拒绝、PqcOnly 拒 sr25519。

## C. runtime:offchain-transaction 批签参数化
- `offchain-transaction/src/lib.rs:635-663` 按 `batch_signature` 首字节 algo tag 分流:`sr25519` 旧路、`0x02` 走 `verify_ml_dsa_65`(提交者公钥从 `AccountKeys::BoundPqcKey` 取,替代 `:655-663` 的"account 即公钥")。
- `Config` 加 `BatchSigVerifier`(primitives trait);`configs/mod.rs:1216` `MaxBatchSignatureLength` 128 → 4736;删 `:46-47` 硬编码 import。

## D. runtime:`RejectSr25519WhenPqcOnly`(PqcOnly 收紧)
新增自定义 TransactionExtension(照 `CheckNonStakeSender` `lib.rs:179-234`),插入 `TxExtension`(`lib.rs:164-177`):origin 为 sr25519 签名且账户 `state==PqcOnly` → `InvalidTransaction`。

## E. node 实现(两个本机热钱包,不动 PoW seal)

### E.1 ML-DSA-65 密钥来源(决策点)
- **方案 A(推荐):同源派生** —— 从矿工 `powr` keystore 的 BIP39 seed、清算行 `signing_key.enc` 的 seed,经 `gmb-pqc` 派生 ML-DSA-65。一份备份恢复 sr25519+ML-DSA 两支。需从 substrate keystore 读取原始 seed(node 已直接读 keystore 文件名/加密 seed,可行)。
- 方案 B(兜底):独立生成 ML-DSA-65 密钥存 keystore,用 sr25519(powr/清算行)做 `bind_pqc_key` 外层签名绑定。更简单但备份多一份、且 keystore 丢失即不可恢复。
- ⚠️ 两钱包密钥**保持独立、不合并**(决策 2026-06-07)。

### E.2 矿工热钱包(最高优先级)
- `submit_powr_signed_tx`(`core/rpc.rs`)与 `submit_miner_transfer`(`onchain_transaction/mod.rs:354-431`)、`reward_bindWallet`:构造内层 `Balances::transfer_keep_alive` → ML-DSA-65 签 `blake2_256(call++nonce++genesis)` → 包成 `AccountKeys.pqc_dispatch` → 用 general-transaction 提交。
- 首次需 `bind_pqc_key`(外层 powr sr25519 + 内层 ML-DSA 自签)把矿工账户的 ML-DSA 公钥绑上链。
- UI 不变(首页/设置-手续费收款/交易-钱包管理"矿工热钱包"),仅底层签名算法切换。

### E.3 清算行结算签名器
- `KeystoreBatchSigner::sign_batch`(`settlement/signer.rs:41-51`)改 ML-DSA-65(对齐 §C algo tag);批次提交(`PoolBatchSubmitter::submit`)从"sr25519 签名 extrinsic"改为 `pqc_dispatch` general-tx。
- 同样先 `bind_pqc_key` 绑定清算行账户的 ML-DSA 公钥。

### E.4 general-transaction 构造
node 当前用 `UncheckedExtrinsic::new_signed`;PQC 交易改用 general 构造(`new_transaction(call, tx_ext)`,call = `AccountKeys.pqc_dispatch{...}`),AuthorizeCall 负责授权。需在 node 交易构造层新增 general-tx 路径。

### E.5 PoW seal 明确不动
矿工 `powr` 的出块 seal 签名(`service.rs:93-143,275-286`)是共识规则,**本卡不改**,归范围 C。

## F. Step 1 内部分阶段(各阶段单独 setCode + bump spec_version)
- **S1-P0**(链上 0 行为变化):`gmb-pqc` + `primitives::pqc` + `account-keys` 骨架(storage + `bind_pqc_key` + benchmark)+ node 接 `gmb-pqc` 能派生/绑定。新 pallet 不被现有路径触达。
- **S1-P1**(hybrid):`pqc_dispatch` + authorize + 手续费落定;node 矿工热钱包转账走 pqc_dispatch。端到端可验证。
- **S1-P2**:offchain 批签参数化 + `MaxBatchSignatureLength` + node 清算行 ML-DSA + pqc_dispatch 提交。
- **S1-P3**:`RejectSr25519WhenPqcOnly` 接入 TxExtension;node 矿工/清算行账户 opt-in 切 PqcOnly。

## G. 验证(端到端,不依赖 Step 2)
1. node 启动生成 powr 矿工账户,派生并 `bind_pqc_key`,链上 `BoundPqcKey` 出现该账户(Active)。
2. 在交易-钱包管理用"矿工热钱包"转账 → node 构造 `pqc_dispatch` general-tx → 链上 authorize 验 ML-DSA-65 通过 → 转账成功,余额变化正确,**地址不变**。
3. nonce 防重放:重放同一 pqc_dispatch 被拒。
4. 清算行批次用 ML-DSA-65 提交并被 `verify_batch_signature` 接受。
5. 矿工账户切 PqcOnly 后,其 sr25519 通道被 `RejectSr25519WhenPqcOnly` 拒绝;ML-DSA 通道仍通。
6. `cargo test` 全绿(主 crate 需 `WASM_FILE`);benchmark 出 weight。

## H. 关键风险 / 待定
- general-tx + `ChargeTransactionPayment` 计费机制(§B.4)——Phase 1 前做 spike。
- `fips204` seed-based 确定性 keygen 是否可用(§A.1)。
- 从 substrate keystore 提取 seed 做同源派生 vs 独立 ML-DSA 密钥(§E.1)。
- ML-DSA-65 验签 WASM weight 与签名 3309B 进 extrinsic 的 block length 压力(可能需调 `BlockLength` / length fee)。
- general-tx 在本 PoW fork 的可用性确认(`AuthorizeCall` 已在 TxExtension,理论支持,需实测)。

## 验收标准
- 链端 `account-keys` pallet 上线,`bind_pqc_key` / `pqc_dispatch` 按设计工作,spec_version 已 bump,纯 setCode 升级。
- node 矿工热钱包与清算行结算签名器均用 ML-DSA-65 签名并被链端接受;**地址/余额不变**。
- offchain 批签支持 ML-DSA-65;`MaxBatchSignatureLength=4736`。
- PqcOnly 收紧生效;PoW seal 未改动(范围 C)。
- 文档回写、中文注释、残留 sr25519 硬编码清理;单测/benchmark 通过。
