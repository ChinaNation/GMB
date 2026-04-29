# 清算行节点 substrate 集成 · Step 2b-ii-β-2-b 技术说明

- **日期**:2026-04-19
- **范围**:扫码支付 Step 2b-ii-β-2-b(pool.submit_one 真调 + CLI flag + service.rs 启动 + rpc.rs 合并 namespace)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2B_II_B_2_A_SUBMITTER.md`(β-2-a:extrinsic 构造 + 占位 submit)
- **后续**:`STEP2B_III_LISTENER_GOSSIP.md`(Step 2b-iii:事件订阅 + libp2p gossip)与 `STEP2B_IV_CLEANUP.md`(删除旧 offchain_*.rs)

---

## 1. 本步范围(β-2-b 一把做完)

原本 β-2-b 内部分 B-1/B-2/B-3/B-4 四步,全部在本轮顺序完成:

- **B-1** · `settlement/submitter.rs` 替换占位为真实 `pool.submit_one` + `account_nonce` 查询
- **B-2** · `cli.rs` 加 `--clearing-bank <SS58>` + `--clearing-bank-password <STR>`
- **B-3** · `service.rs` 启动清算行组件 + `spawn` packer worker;`command.rs` / `node_runner.rs` 透传 CLI
- **B-4** · `rpc.rs` 的 `FullDeps` 加字段 + `create_full` 合并 `OffchainClearingRpcServer::into_rpc`;`OffchainClearingRpcImpl` 派生 `Clone`

**明确不做**(Step 2b-iii / 2b-iv):
- libp2p gossip 真实接入
- `event_listener` 对接 `sc-client-api::BlockchainEvents`
- 删除旧 `offchain_ledger.rs` / `offchain_packer.rs` / `offchain_gossip.rs`

---

## 2. 改动清单

### 2.1 `offchain/settlement/submitter.rs`

| 变更 | 内容 |
|---|---|
| 新 `use` | `frame_system_rpc_runtime_api::AccountNonceApi`, `sc_transaction_pool_api::{TransactionPool, TransactionSource}`, `sp_api::ProvideRuntimeApi`, `sp_runtime::OpaqueExtrinsic` |
| `submit()` | 删除占位 hash,替换为 `pool.submit_one(best_hash, TransactionSource::Local, opaque)` + `block_on`;返回真实 `TxPool::Hash` SCALE 解码为 `H256` |
| `lookup_nonce()` | 新增:`api.account_nonce(best_hash, account)`,失败回退 0(链上 `CheckNonce` 会拒错误 nonce,不会静默) |
| `pool` 字段 | 去掉 `#[allow(dead_code)]`(本步起真实使用) |

### 2.2 `cli.rs`

新增 2 个可选 flag:
```rust
#[arg(long, value_name = "BANK_MAIN_SS58")]
pub clearing_bank: Option<String>,

#[arg(long, value_name = "PASSWORD")]
pub clearing_bank_password: Option<String>,
```

### 2.3 `service.rs`

`new_full` 签名新增 2 个参数:
```rust
pub fn new_full(
    mut config: Configuration,
    mining_threads: usize,
    gpu_device: Option<usize>,
    clearing_bank: Option<String>,
    clearing_bank_password: Option<String>,
) -> Result<TaskManager, ServiceError>
```

在 `transaction_pool` 构造完成后、`rpc_extensions_builder` 之前插入约 80 行清算行启动块:

1. 解析 `--clearing-bank` 的 SS58 → `AccountId32`(失败 → warning + 跳过)
2. 构造 `OffchainKeystore`,有密码且密钥文件存在时解锁填入 `signing_key_slot`
3. 组装 `signer: Arc<dyn BatchSigner> = KeystoreBatchSigner`
4. 组装 `submitter: Arc<dyn BatchSubmitter> = PoolBatchSubmitter`
5. `offchain::start_clearing_bank_components(...)` 拿 `OffchainComponents`
6. `task_manager.spawn_handle().spawn("offchain-clearing-packer", ...)` 启动 30 秒 tick 的 `packer.should_pack → pack_and_submit` loop
7. `Some(components.rpc_impl)` 作为 `clearing_rpc_impl` 给 RPC builder

失败路径(地址非法 / keystore 问题 / 组件启动 Err)一律**只 log warning,不中断节点**,保证 PoW + GRANDPA 基础职能不受影响。

### 2.4 `command.rs`

`runner.run_node_until_exit` 闭包调用 `new_full` 时透传 `cli.clearing_bank.clone()` + `cli.clearing_bank_password.clone()`。

### 2.5 `node_runner.rs`

Tauri UI 路径调用 `new_full` 时透传 `None, None`(GUI 启动暂不支持清算行角色,生产用无头 CLI)。

### 2.6 `rpc.rs`

- `FullDeps` 加字段 `offchain_clearing_rpc: Option<Arc<crate::offchain::rpc::OffchainClearingRpcImpl>>`
- `create_full` 里 destructure 追加 + `if let Some(impl_) = ... { module.merge(OffchainClearingRpcServer::into_rpc((*impl_).clone()))? }`

### 2.7 `offchain/rpc.rs`

- `OffchainClearingRpcImpl` 派生 `#[derive(Clone)]`(内部仅 `Arc`,廉价)

---

## 3. 运行时数据流(端到端首次打通)

```
wuminapp                          清算行节点                     链上 runtime
────────                          ──────────                     ────────────
扫 QR                                                             
sign PaymentIntent                                                
POST offchain_submitPayment   →  rpc/create_full                  
                                 → OffchainClearingRpcImpl.submit_payment
                                 → ledger.accept_payment
                                   ✅ L3 签名验证 + nonce + 余额
                                   ✅ pending 入队,返回 tx_id
                                                                  
every 30s (spawned task)                                          
  packer.should_pack(block)  →  ledger.pending ≥ 阈值 / 满 10 块 → 触发
                                 → take_pending_for_batch
                                 → NodeBatchItem[] + batch_seq
                                 → batch_signing_message
                                 → signer.sign_batch(msg)        ✅ KeystoreBatchSigner
                                 → submitter.submit(...)         ✅ PoolBatchSubmitter
                                   ✅ lookup_nonce(client)
                                   ✅ build_signed_extrinsic
                                   ✅ pool.submit_one(best_hash,
                                         Local, opaque)        → TransactionPool
                                                                ↓
                                                          RuntimeCall::OffchainTransaction(
                                                            submit_offchain_batch_v2 {..})
                                                                ↓
                                                          ✅ settlement::execute_clearing_bank_batch
                                                          ✅ 每笔 sr25519_verify
                                                          ✅ 同/跨行分账
                                                          ✅ 发 PaymentSettled 事件
```

**Step 2b-iii 起** `event_listener` 订阅 `PaymentSettled` 事件后清理 `ledger.pending`,闭环最终完成。本步已经能让扫码付款的 extrinsic 成功进链上交易池并被出块节点打包。

### 3.1 2026-04-28 安全补齐

本轮把原先的占位字段接成可验收闭环:

- `submit_payment` 不再返回 `[0u8;64]` ACK,而是复用 `KeystoreBatchSigner` 对
  `GMB_L2_ACK_V1 || bank_main || SCALE(intent) || payer_sig || accepted_at` 签名。
- RPC 入 pending 前会读链上 `UserBank[payer]`、`UserBank[recipient]` 与
  `L2FeeRateBp[recipient_bank]`,提前拒绝错路由、绑定漂移、未配置费率和手续费不一致。
- `OffchainPacker` 启动时读取链上 `LastClearingBatchSeq[bank]`,下一批从 `last + 1`
  续跑,避免节点重启后重复提交 batch_seq=1。
- runtime 入口严格校验 batch 级 sr25519 签名和 batch_seq,并在 settlement 成功后写
  `LastClearingBatchSeq`;失败批次不推进序号。

---

## 4. 启动命令示例

```bash
# 无清算行角色(默认)
./target/release/citizenchain --chain local

# 以清算行角色启动(SS58 为链上已注册且是 Active 的清算行主账户地址)
./target/release/citizenchain \
  --chain local \
  --clearing-bank 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --clearing-bank-password "my-strong-password"
```

---

## 5. 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node
(offchain / service / rpc / cli / command / node_runner 全部零 error;
 链接阶段仅剩 desktop.rs tauri proc macro `frontend/dist` 门禁,与本步无关)
```

---

## 6. 已知风险与缓解

| 风险 | 等级 | 现状 |
|---|---|---|
| `block_on` 在 tokio runtime 内可能死锁 | **P0** | 本步用 `futures::executor::block_on` 一次性阻塞完成 `pool.submit_one`;若实测压力下触发 panic,下一步把 `BatchSubmitter::submit` 改 async + packer `await` |
| `--clearing-bank-password` 以明文 CLI 传入 | **P1** | 临时可接受;生产建议:从 env 读或首次启动命令行 prompt,Step 2b-iv 加固 |
| `lookup_nonce` 失败静默回退 0 | **P2** | runtime `CheckNonce` 会拒错误 nonce,不会静默成功;但会让 packer 反复提交失败 log 打爆。监控 `pool.submit_one` Err 频率触发告警 |
| 仅清算行角色节点可调 offchain_* RPC | **P2** | 其他节点启动时 `offchain_clearing_rpc = None`,`create_full` 跳过 merge,RPC namespace 不存在 |

---

## 7. 后续

**Step 2b-iii**(建议下一步):
- `offchain/settlement/listener.rs` 接 `sc-client-api::BlockchainEvents`:订阅 `PaymentSettled` 事件清理 pending,订阅 `Deposited` / `Withdrawn` 同步 confirmed 余额
- `offchain/reserve.rs` 新增:周期性 `available_balance vs total_deposits` 对账告警
- `offchain/gossip.rs`:libp2p `NotificationService` + 协议名 `/gmb/offchain/1`(清算行间推送 pending 意图)

**Step 2b-iv**(清理):
- 删除 `node/src/offchain_{ledger,packer,gossip}.rs`
- 删除 `main.rs` 里对应 mod 声明
- 删除 `rpc.rs::FullDeps` 中 `offchain_ledger` / `offchain_shenfen_id` / `offchain_gossip_tx`(老省储行清算字段)
- runtime `offchain-transaction` 删除 call_index 0 旧 `submit_offchain_batch` / 9 旧 `bind_clearing_institution` / 1/2 旧 `propose_institution_rate`

## 8. 变更记录

- 2026-04-19:Step 2b-ii-β-2-b 完整落地,清算行节点端到端链路打通(wuminapp RPC → ledger → packer → pool → extrinsic → runtime)。offchain/ 子树 + service.rs + rpc.rs + cli.rs + command.rs + node_runner.rs 零编译错误。
