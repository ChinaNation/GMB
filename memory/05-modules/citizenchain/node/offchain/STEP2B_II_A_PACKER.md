# citizenchain/node/src/offchain/settlement/packer.rs · Step 2b-ii-α 技术说明

- **日期**:2026-04-19
- **范围**:扫码支付 Step 2b-ii-α(packer 业务逻辑 + 依赖注入接口)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_I_NODE.md`(ledger.accept_payment / rpc.submitPayment 已就绪)
- **后续**:`STEP2B_II_B_INTEGRATION.md`(接 substrate client + CLI flag + RPC 注册)

---

## 1. 本步范围

Step 2b-ii 被拆成 α / β 两个子步,本次交付 **α · 业务逻辑 + 依赖注入**:
- `settlement/packer.rs` 完整业务逻辑(组 batch → 签名 → 提交 → 失败回滚)
- 两个依赖注入 trait:`BatchSigner` / `BatchSubmitter`
- 两个 noop 占位:`NoopBatchSigner` / `NoopBatchSubmitter`
- `NodeBatchItem` 结构(节点镜像 runtime `OffchainBatchItemV2`)
- `mod.rs` 启动器 `start_clearing_bank_components` 增补 signer/submitter 参数 + `start_clearing_bank_components_with_noop` 便捷版
- 4 个 tokio + 2 个同步单测,覆盖正/负路径与字节编码不变性

**明确不做**(Step 2b-ii-β 接):
- `KeystoreBatchSigner`(用 `offchain::keystore::SigningKey` 签)
- `PoolBatchSubmitter`(拼 `RuntimeCall::OffchainTransaction(submit_offchain_batch_v2 {...})` 外包 `UncheckedExtrinsic` + 调 `TransactionPool`)
- `service.rs` CLI flag `--clearing-bank` + 启动逻辑
- `rpc.rs` 合并 `OffchainClearingRpcServer::into_rpc` 到节点 JSON-RPC

---

## 2. 新增内容

### 2.1 `NodeBatchItem`

```rust
pub struct NodeBatchItem {
    pub tx_id: H256,
    pub payer: AccountId32,
    pub payer_bank: AccountId32,
    pub recipient: AccountId32,
    pub recipient_bank: AccountId32,
    pub transfer_amount: u128,
    pub fee_amount: u128,
    pub payer_sig: [u8; 64],
    pub payer_nonce: u64,
    pub expires_at: u32,
}
```

字段**逐字段对齐** runtime `offchain_transaction::batch_item::OffchainBatchItemV2`,SCALE 编码字节完全一致。单测 `node_batch_item_encodes_deterministically` 断言编码长度 268 字节 + round-trip 成功。

### 2.2 依赖注入 trait

```rust
pub trait BatchSigner: Send + Sync {
    fn sign_batch(&self, message: &[u8]) -> Result<[u8; 64], String>;
}

pub trait BatchSubmitter: Send + Sync {
    fn submit(
        &self,
        institution_main: AccountId32,
        batch_seq: u64,
        batch_bytes: Vec<u8>,
        batch_signature: [u8; 64],
    ) -> Result<H256, String>;
}
```

### 2.3 `batch_signing_message`

```rust
pub fn batch_signing_message(
    institution_main: &AccountId32,
    batch_seq: u64,
    batch_bytes: &[u8],
) -> [u8; 32];
```

= `blake2b_256(b"GMB_OFFCHAIN_BATCH_V1" || institution || batch_seq_le || batch_bytes)`

2026-04-28 补齐:链上 `submit_offchain_batch_v2` 已严格校验 `batch_signature`,
并把成功批次写入 `LastClearingBatchSeq[bank]`。本函数必须继续与 runtime
`batch_item::batch_signing_hash` 逐字节一致。

### 2.4 `OffchainPacker::pack_and_submit`

完整流程:

```
1. ledger.take_pending_for_batch(PACK_TX_THRESHOLD)
   → Vec<PendingPayment>  (空 → 返回 Ok(None))

2. Vec<NodeBatchItem>::from(...)

3. batch_seq = batch_seq_counter.fetch_add(1)  (启动时从 LastClearingBatchSeq 续跑)

4. batch_bytes = batch.encode()
   message = batch_signing_message(institution, batch_seq, &batch_bytes)
   sig = signer.sign_batch(&message)?
     └─ Err → rollback 整批次 pending → return Err

5. submitter.submit(institution, batch_seq, batch_bytes, sig)?
     └─ Err → rollback 整批次 pending → return Err

6. Ok → 记录 last_pack_block;pending 保留等 PaymentSettled 事件清理
```

### 2.5 `mod.rs` 启动器

新签名:
```rust
pub fn start_clearing_bank_components(
    base_path: &Path,
    bank_main: AccountId32,
    password: &str,
    signer: Arc<dyn BatchSigner>,
    submitter: Arc<dyn BatchSubmitter>,
) -> Result<OffchainComponents, String>
```

便捷版(α 阶段 service.rs 使用):
```rust
pub fn start_clearing_bank_components_with_noop(
    base_path: &Path,
    bank_main: AccountId32,
    password: &str,
) -> Result<OffchainComponents, String>
```

---

## 3. 回滚语义

- `pack_and_submit` 失败(签名或提交)→ `rollback` 遍历 `batch`,对每条 `item.tx_id` 调 `ledger.reject_pending(tx_id)`。
- `ledger.reject_pending` 内部仅当 `cached_nonce == item.nonce` 时回滚 nonce,避免破坏后续已入 pending 的 nonce 链。
- **成功路径**:`pack_and_submit` 返回 `Ok(Some(tx_hash))`,但 ledger 的 pending 仍然保留。pending 真正清理**只能**由 Step 2b-iii 的 `event_listener.on_payment_settled` 在链上 `PaymentSettled` 事件到达时执行。

---

## 4. 单元测试

| 测试 | 覆盖 |
|---|---|
| `should_pack_is_false_when_empty` | 触发条件(Step 1 继承) |
| `noop_signer_triggers_rollback` | Noop 默认实现 → pack_and_submit Err → ledger pending 归 0 |
| `happy_path_submits_and_keeps_pending` | Mock 成功 → ledger pending 不动 + submitter 收到正确参数 |
| `submitter_error_rolls_back` | Mock 提交失败 → pending 全部剔除 |
| `batch_signing_message_is_deterministic` | 消息对 institution / batch_seq / bytes 任一变化敏感 |
| `node_batch_item_encodes_deterministically` | SCALE 编码长度 268 字节 + round-trip |

---

## 5. 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node
(offchain/ 子树零错误;grep offchain 0 命中;desktop.rs tauri macro 受 frontend/dist 门禁,与本步无关)
```

---

## 6. Step 2b-ii-β 对接清单

下一步要实现:

1. **`KeystoreBatchSigner`**
   - 从 `offchain::keystore::SigningKey` 拿到清算行管理员 sr25519 私钥
   - `sign_batch(message) -> [u8; 64]` 调 `sp_core::Pair::sign(&key, message)`
   - 放在 `offchain/settlement/signer.rs`

2. **`PoolBatchSubmitter`**
   - 持有 `Arc<FullClient>` + `Arc<TransactionPool>`
   - `submit` 内:
     - 用 `batch_bytes` decode 回 `Vec<OffchainBatchItemV2>`(或直接透传 SCALE,让 RuntimeCall 构造复用)
     - 构造 `RuntimeCall::OffchainTransaction(submit_offchain_batch_v2 {...})`
     - 外包 `UncheckedExtrinsic`(需要签名,或 unsigned extrinsic 走特殊路径)
     - 调 `pool.submit_one(...)` 拿 `TransactionHash`
   - 放在 `offchain/settlement/submitter.rs`

3. **`service.rs` 接入**
   - CLI 加 `--clearing-bank <MAIN_ACCOUNT_SS58>` 可选参数
   - 启动时若设置:
     - 从 keystore 取 SigningKey
     - 构造 `KeystoreBatchSigner` + `PoolBatchSubmitter`
     - 调 `start_clearing_bank_components(..., signer, submitter)` 拿 `OffchainComponents`
     - 起后台 worker:每 N 块调 `packer.should_pack` → `pack_and_submit`
     - 保存 `Arc<OffchainComponents>` 到 `TaskManager` 或 ctx

4. **`rpc.rs` 接入**
   - extension builder 若检测到 `OffchainComponents` 存在,调
     `io.merge(OffchainClearingRpcServer::into_rpc(components.rpc_impl.clone()))`

---

## 7. 变更记录

- 2026-04-19:Step 2b-ii-α packer 业务逻辑 + 2 个依赖注入 trait + Noop 占位 + 启动器增补 + 6 个单测,零编译错误。
- 2026-04-29:文档引用随目录收口改为 `offchain::keystore::SigningKey`。
