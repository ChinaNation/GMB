# citizenchain/node/src/offchain/settlement/submitter.rs · Step 2b-ii-β-2-a 技术说明

- **日期**:2026-04-19
- **范围**:扫码支付 Step 2b-ii-β-2-a(substrate extrinsic 构造器,含 placeholder submit)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_II_B_1_SIGNER.md`(KeystoreBatchSigner 真实签名器已就绪)
- **后续**:`STEP2B_II_B_2_B_SERVICE.md`(service.rs 接入 + CLI flag + RPC 合并 + 替换 placeholder submit 为真正 `pool.submit_one`)

---

## 1. 本步范围

Step 2b-ii-β-2 被拆成 a / b 两个子步,本次交付 **a · extrinsic 构造器 + 字段打包 helper**:

- 新建 `offchain/settlement/submitter.rs`:
  - `PoolBatchSubmitter` 结构(持 `Arc<FullClient>` + `Arc<TxPool>` + `Arc<RwLock<Option<SigningKey>>>`)
  - 实现 `packer::BatchSubmitter::submit`
  - 纯函数 `build_signed_extrinsic` 严格对齐 `benchmarking.rs::create_benchmark_extrinsic` 的 `TxExtension` 拼接顺序
  - 纯函数 `decode_batch_items` / `encode_bounded_sig` / `encode_bounded_batch`
- `offchain/mod.rs` 挂载新子模块
- `node/Cargo.toml` 补 3 个依赖:`offchain-transaction-pos` / `frame-support` / `frame-system-rpc-runtime-api`

**明确不做**(β-2-b):
- 真正调 `pool.submit_one(...)`(本步返回 extrinsic blake2b_256 作为占位 hash + log + TODO)
- 查询真实账户 nonce(本步用 0 占位,β-2-b 接 `ProvideRuntimeApi::account_nonce`)
- `cli.rs` flag
- `service.rs` 启动清算行 worker
- `rpc.rs` 合并 `OffchainClearingRpcServer::into_rpc`

---

## 2. 设计要点

### 2.1 类型选择

```rust
pub type TxPool =
    sc_transaction_pool::TransactionPoolHandle<runtime::opaque::Block, FullClient>;
```

**关键**:pool 约束的 Block 必须是 `runtime::opaque::Block`(OpaqueExtrinsic 版),与 `service.rs` 的 `use citizenchain::{opaque::Block};` 保持一致。用 `runtime::Block`(具体 UncheckedExtrinsic 版)会触发大量 trait bounds 不满足(Client 只为 opaque::Block 实现 `ProvideRuntimeApi`/`BlockBackend` 等)。

### 2.2 依赖密钥复用

`signing_key: Arc<RwLock<Option<SigningKey>>>` 与 β-1 的 `KeystoreBatchSigner` 共享同一个 slot:

- β-1 `sign_batch`:签**batch 内部的 `batch_signature`**。2026-04-28 起 runtime
  已严格校验本签名,消息必须与 `GMB_OFFCHAIN_BATCH_V1 || institution || batch_seq || batch_bytes`
  保持逐字节一致。
- β-2-a 外层 `SignedPayload`:签**整个 extrinsic** 的 `TxExtension + call + implicit`,构成 `UncheckedExtrinsic.signature`

两签名共用同一 `sr25519::Pair`,所以对应账户必须是该清算行在 `admins-change::Institutions` 中登记的管理员之一,否则链上 `ensure_signed` 通过后 pallet 内置 `is_admin_of` 检查会拒绝。

### 2.3 extrinsic 构造

复制 `benchmarking.rs::create_benchmark_extrinsic` 的 `TxExtension` 12 元组(顺序与 `runtime::TxExtension` 严格一致):

```
(AuthorizeCall, CheckNonZeroSender, CheckNonStakeSender,
 CheckSpecVersion, CheckTxVersion, CheckGenesis, CheckEra,
 CheckNonce, CheckWeight, ChargeTransactionPayment,
 CheckMetadataHash, WeightReclaim)
```

`SignedPayload::from_raw` 携带 implicit 部分:`(spec_version, tx_version, genesis_hash, best_hash)`,签名消息由 `raw_payload.using_encoded(|e| sender.sign(e))` 给出。

### 2.4 β-2-a 的降级提交

`PoolBatchSubmitter::submit` 本步:
1. ✅ decode batch_bytes → Vec<OffchainBatchItemV2>
2. ✅ 打包 BoundedVec<..., MaxBatchSize> 和 BatchSignatureOf<Runtime>
3. ✅ 组 `RuntimeCall::OffchainTransactionPos(submit_offchain_batch_v2 {..})`
4. ✅ 从 signing_key 读 sr25519 pair
5. ⚠️ **nonce 暂用 0 占位**(β-2-b 查真实)
6. ✅ 调 `build_signed_extrinsic` 构造签名 extrinsic
7. ⚠️ **不调 `pool.submit_one`**,返回 `H256::from(blake2b_256(extrinsic.encode()))` 作为占位 hash + log 记录

packer 收到 `Ok(placeholder_hash)` 后:
- pending 保留在本地 ledger(等 `PaymentSettled` 事件清理)
- 实际上链上并没有这笔 extrinsic,所以事件**永远不会触发**
- Step 2b-ii-β-2-b 接入真实 `pool.submit_one` 后,替换 placeholder 逻辑即可

这个降级是**故意的**:本步保证 extrinsic 构造正确(编译 + 字节级对齐 benchmarking.rs),β-2-b 只需替换最后一步的"不提交 → submit_one"。

---

## 3. 单元测试

| 测试 | 覆盖 |
|---|---|
| `batch_bytes_decodes_to_items` | `Vec<OffchainBatchItemV2>` SCALE roundtrip |
| `encode_bounded_sig_respects_limit` | 64 字节签名打包到 `BoundedVec<u8, MaxBatchSignatureLength>` 不超限 |
| `encode_bounded_batch_respects_limit` | 2 条 item 打包到 `BoundedVec<_, MaxBatchSize>` 不超限,字段 roundtrip |
| `decode_batch_items_rejects_invalid_bytes` | 非法字节返回 Err |
| `sign_key_slot_none_returns_err` | signing_key `None` 的锁访问语义 |

> 注:`build_signed_extrinsic` 和 `submit` 的完整端到端测试需要 `FullClient` 实例,留 β-2-b 集成测试做。

## 4. Cargo.toml 新依赖

```toml
offchain-transaction-pos = { path = "../runtime/transaction/offchain-transaction-pos", default-features = true }
frame-support = { workspace = true, default-features = true }
frame-system-rpc-runtime-api = { workspace = true, default-features = true }
```

> `frame-system-rpc-runtime-api` 本步没被 use,但 β-2-b `account_nonce` 需要;提前加避免 β-2-b 再改一次。

## 5. 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node
(offchain/ 子树 grep 0 命中,pool_submitter 无 error/warning;链接阶段 ui/mod.rs:91 tauri proc macro 受 frontend/dist 门禁,与本步无关)
```

## 6. Step 2b-ii-β-2-b 对接清单

β-2-b 要做:

1. **真正 submit**:把 `PoolBatchSubmitter::submit` 的降级 placeholder 换成
   ```rust
   let opaque: OpaqueExtrinsic = extrinsic.into();
   let fut = self.pool.submit_one(best_hash, TransactionSource::Local, opaque);
   let tx_hash = futures::executor::block_on(fut)
       .map_err(|e| format!("submit_one 失败:{e:?}"))?;
   Ok(H256::decode(&mut &tx_hash.encode()[..]).unwrap())
   ```

2. **真实 nonce 查询**:
   ```rust
   use sp_api::ProvideRuntimeApi;
   use frame_system_rpc_runtime_api::AccountNonceApi;
   let api = client.runtime_api();
   let best_hash = client.info().best_hash;
   let nonce = api.account_nonce(best_hash, sender_account.clone()).unwrap_or(0);
   ```

3. **CLI flag**(`cli.rs`):
   ```rust
   #[arg(long, value_name = "MAIN_SS58")]
   pub clearing_bank: Option<String>,

   #[arg(long)]
   pub clearing_bank_password: Option<String>,
   ```

4. **service.rs 启动**:
   ```rust
   if let Some(bank_ss58) = cli.clearing_bank {
       let bank_main = AccountId32::from_ss58check(&bank_ss58)?;
       let password = cli.clearing_bank_password.as_deref().unwrap_or("");
       let keystore = OffchainKeystore::new(&base_path);
       let signing_key_slot = Arc::new(RwLock::new(None));
       if keystore.has_signing_key() {
           *signing_key_slot.write().unwrap() =
               Some(keystore.load_signing_key(password)?);
       }
       let signer = Arc::new(KeystoreBatchSigner::new(signing_key_slot.clone()));
       let submitter = Arc::new(PoolBatchSubmitter::new(
           client.clone(),
           transaction_pool.clone(),
           signing_key_slot,
       ));
       let components = offchain::start_clearing_bank_components(
           &base_path, bank_main, password, signer, submitter,
       )?;
       // 后台 packer worker:
       task_manager.spawn_handle().spawn("offchain-packer", None,
           packer_loop(components.packer.clone(), client.clone()));
       // RPC 注入 rpc::RpcBuilder 时 merge OffchainClearingRpcServer
   }
   ```

5. **rpc.rs**:RPC extension 构建函数增加参数 `clearing_rpc: Option<Arc<OffchainClearingRpcImpl>>`,有值时 `io.merge(...)`。

## 7. 变更记录

- 2026-04-19:Step 2b-ii-β-2-a pool_submitter 落地(extrinsic 构造器 + 占位 submit),零编译错误。
