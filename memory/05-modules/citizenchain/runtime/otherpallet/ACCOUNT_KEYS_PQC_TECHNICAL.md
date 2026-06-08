# account-keys / PQC 迁移技术设计

- 状态:设计 / 待实现(本文件为 ADR-016 的实现蓝图,代码尚未落地)
- 关联决策:`memory/04-decisions/ADR-016-account-key-pqc-migration.md`
- 关联任务卡:`memory/08-tasks/open/20260607-wallet-pqc-passkey.md`

## 0. 功能需求

- 让 sr25519 账户在**不换助记词、不换账户、不换地址、不换余额**的前提下,平滑迁移到后量子签名 ML-DSA-65。
- 链上同一账户主体可绑定多把签名凭证(sr25519 + ML-DSA-65),由账户状态机控制"前期 sr25519 → hybrid 并行 → PQC 唯一签名"。
- 验签完全可由链上状态重建,不依赖链下缓存或在线状态。
- runtime 改动遵循"chainspec 创世冻结、只走链上 setCode"运维模型,默认不引入 host function。

## 1. 模块定位与代码事实

新增 FRAME pallet `account-keys`(`pallet_index = 27`,下一个空闲位),目录照搬 `sfid-system`:
```text
citizenchain/runtime/otherpallet/account-keys/
  Cargo.toml
  src/{lib.rs, benchmarks.rs, weights.rs, tests/{mod.rs, cases.rs}}
```

复刻范式:`citizenchain/runtime/otherpallet/sfid-system/src/lib.rs` 的
`ShengSigningPubkey`(存公钥)、`verify_sr25519`(`lib.rs:732-736`)、`BindCredential` + 双向映射 + `UsedBindNonce`(nonce 防重)。

依赖的代码事实(本文件不改这些,只引用):
- `AccountId = sr25519 公钥原样 32 字节`:`citizenchain/runtime/src/lib.rs:130-134`。
- general-transaction 授权通道已就绪(`frame_system::AuthorizeCall`):`lib.rs:164-177`;`UncheckedExtrinsic`:`lib.rs:237-238`。
- 自定义 TransactionExtension 模板 `CheckNonStakeSender`:`lib.rs:179-234`。
- `#[pallet::authorize]` 机制(SDK fork):`authorize_call.rs:59-113`(authorize 可读 storage、成功后 origin 设 `Authorized`)。
- pallet 注册宏块:`lib.rs:284-390`;`spec_version`:`lib.rs:83`。
- 唯一硬编码 sr25519 批签:`offchain-transaction/src/lib.rs:46-47, 635-663`;`MaxBatchSignatureLength=ConstU32<128>`:`configs/mod.rs:1216`。

## 2. 关键数据结构

```rust
// PqcKeyRecord:每账户一条 PQC 绑定记录(只存公钥 hash,见 ADR-016)
pub struct PqcKeyRecord<BlockNumber> {
    pub algo: u8,            // 0x02 = ML-DSA-65(标签空间见 ADR-016)
    pub pubkey_hash: [u8; 32],
    pub state: KeyState,     // Active(已绑定可并行) / PqcOnly(拒 sr25519) / Revoked
    pub bound_at: BlockNumber,
}

#[pallet::storage]                       // canonical AccountId → PQC 凭证
pub type BoundPqcKey<T> = StorageMap<_, Blake2_128Concat, T::AccountId, PqcKeyRecord<BlockNumberFor<T>>, OptionQuery>;

#[pallet::storage]                       // PQC 交易防重放(general-tx 不走 CheckNonce,自管)
pub type AccountKeyNonce<T> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;
```

Config 常量(照 sfid-system `MaxCredentialSignatureLength` 写法):
`MaxPqcPubkeyLen = ConstU32<2048>`(容 ML-DSA-65 公钥 1952B)、`MaxPqcSigLen = ConstU32<4736>`(容签名 3309B 并预留 ML-DSA-87 4627B)。

## 3. extrinsic 与验证路径

### 3.1 `bind_pqc_key`(call_index 0,hybrid 双签)

外层由 sr25519 正常签名(证明现账户主人=第一签),参数携带 `pqc_pubkey, algo, pqc_self_sig`(第二签)。

```rust
let who = ensure_signed(origin)?;
let nonce = AccountKeyNonce::<T>::get(&who);
let challenge = blake2_256(&[who.encode(), pqc_pubkey.to_vec(), nonce.encode(), genesis_hash()].concat());
ensure!(verify_ml_dsa_65(&pqc_pubkey, &challenge, &pqc_self_sig), Error::<T>::InvalidPqcSelfSig);
BoundPqcKey::<T>::insert(&who, PqcKeyRecord { algo, pubkey_hash: blake2_256(&pqc_pubkey), state: Active, bound_at: now });
AccountKeyNonce::<T>::mutate(&who, |n| *n += 1);
```

`verify_ml_dsa_65` 封装照 sfid-system `verify_sr25519`(`lib.rs:732-736`)同款,内部调 `fips204::ml_dsa_65::verify`。**升级(65→87)走同一 extrinsic 换 algo 重绑**。

### 3.2 `pqc_dispatch`(call_index 1,让任意 call 被 PQC 授权 —— PQC 唯一签名的执行点)

PQC 交易 = 一笔 general-transaction(无外层 sr25519 签名):

```rust
#[pallet::call_index(1)]
#[pallet::authorize(|_src, account, call, nonce, pubkey, sig| {
    let rec = BoundPqcKey::<T>::get(account).ok_or(InvalidTransaction::Call)?;   // authorize 可读 storage
    ensure!(rec.state != Revoked && blake2_256(pubkey) == rec.pubkey_hash);
    ensure!(*nonce == AccountKeyNonce::<T>::get(account));
    let msg = blake2_256(&[call.encode(), nonce.encode(), genesis_hash()].concat());
    ensure!(verify_ml_dsa_65(pubkey, &msg, sig));                                // WASM 内 fips204 验签
    Ok((valid_transaction(), AUTH_WEIGHT))
})]
pub fn pqc_dispatch(origin, account, call: Box<RuntimeCall>, nonce: u32, algo: u8, pubkey, sig) -> DispatchResult {
    ensure_authorized(origin)?;                                                  // authorize 已设 Authorized origin
    AccountKeyNonce::<T>::mutate(&account, |n| *n += 1);
    call.dispatch(RawOrigin::Signed(account).into())                            // 以 canonical 账户派发内层 call
}
```

要点:
- **不改全局 `TxExtension` 元组**,纯加 pallet;绑定期对现有交易 0 影响。
- 手续费:general-tx 的 `ChargeTransactionPayment` 仍在 `TxExtension`(`lib.rs:174`),需向 canonical `account` 计费——authorize/validate 返回付费账户,或 `pqc_dispatch` 内显式扣费。**实现期定**。
- `RuntimeCall` 编码进 `msg` 做域隔离,防跨 call 重放;nonce 防同 call 重放。

### 3.3 `RejectSr25519WhenPqcOnly`(Phase 3 收紧)

新增自定义 TransactionExtension(照 `CheckNonStakeSender`,`lib.rs:179-234`),插入 `TxExtension`(`lib.rs:164-177`):

```rust
if let Some(who) = origin.as_system_origin_signer() {        // sr25519 签名通道
    if matches!(BoundPqcKey::<T>::get(who).map(|r| r.state), Some(KeyState::PqcOnly)) {
        return Err(InvalidTransaction::Call.into());
    }
}
```

## 4. offchain-transaction 批签参数化

`offchain-transaction/src/lib.rs:635-663` 的 `verify_batch_signature`:读 `batch_signature` 首字节 algo tag,`sr25519` 走旧 `sr25519_verify`、`0x02` 走 `verify_ml_dsa_65`(提交者公钥从 `AccountKeys::BoundPqcKey` 取,替代 `:655-663` 的"account 即公钥"假设)。
- `configs/mod.rs:1216`:`MaxBatchSignatureLength` 128 → 4736。
- 删 `:46-47` 的 `sp_core::sr25519` 硬编码 import,改为按标签分发。

## 5. 体积与权重评估

- **公钥**:ML-DSA-65 公钥 1952B,但链上只存 32B hash → 100 万账户 ≈ 32MB state(存完整公钥则约 2GB)。
- **签名**:3309B 进 extrinsic body(sr25519 仅 64B)。最小转账 extrinsic 从 ~150B 跳到 ~3.5KB,压力在 `BlockLength` / `proof_size` 与 length fee,需相应调参或对 PQC call 设更高 length fee。
- **验签 weight**:fips204 ML-DSA-65 WASM 内验签常量时间、无堆;**必须跑 `runtime-benchmarks` 出真实 `WeightInfo`,禁止用猜测值**。若验签 weight 占区块预算显著比例(>5–10%),Phase 3 再评估 fork `sp-io` 加 `ml_dsa_verify` host function(代价:全节点二进制同步升级)。

## 6. runtime 接线清单

- `lib.rs:284-390` 注册:`#[runtime::pallet_index(27)] pub type AccountKeys = account_keys;`。
- `configs/mod.rs` 加 `impl account_keys::Config for Runtime`(照 sfid-system `:961-973`);`offchain_transaction::Config` 接 `BatchSigVerifier`;`:1216` 改 `MaxBatchSignatureLength`。
- `Cargo.toml` 加 `account-keys` 依赖 + `fips204 = { default-features=false }`;接入 std / runtime-benchmarks / try-runtime 三处 feature(照 `sfid-system` 行)。
- `primitives` 新增 `pqc.rs`:algo 标签常量、`BatchSignatureVerifier` trait、HKDF & challenge domain 常量(与钱包 `gmb-pqc` 共享口径)。

## 7. 分阶段(各阶段单独建任务卡、bump spec_version、走 setCode)

| Phase | 内容 | 链上行为 |
|---|---|---|
| 0 | `primitives::pqc` + 共享 crate `gmb-pqc` + `account-keys` 骨架(storage + `bind_pqc_key` + benchmark)+ 钱包 FFI + 单测 | **0 行为变化**(新 pallet 不被现有路径触达) |
| 1 | `pqc_dispatch` + `#[pallet::authorize]`,hybrid 并行 | 新增 PQC general-tx 通道,sr25519 并存 |
| 2 | offchain 批签参数化 + `MaxBatchSignatureLength`→4736 + QR 协议扩展 | 批签支持 ML-DSA-65 |
| 3 | `RejectSr25519WhenPqcOnly` 接入 `TxExtension` + 可选 host function | PQC-only 收紧 |

## 8. 测试基线

- `account-keys` 单测入 `src/tests/{mod.rs,cases.rs}`(对齐 2026-05-07 pallet 测试重构):bind 双签成功/失败、nonce 防重放、algo 升级重绑、pqc_dispatch 授权成功/拒绝、PqcOnly 拒 sr25519。
- `cargo test` 全 pallet 绿;主 crate 需 `WASM_FILE`。
- benchmark 出 `verify_ml_dsa_65` 真实 weight。
