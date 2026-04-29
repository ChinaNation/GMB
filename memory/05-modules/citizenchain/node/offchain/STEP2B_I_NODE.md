# citizenchain/node/src/offchain · Step 2b-i 节点层落地

- **日期**:2026-04-19
- **范围**:扫码支付清算体系 Step 2b-i(节点业务逻辑)
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **前置**:`STEP2A_RUNTIME.md`(runtime 重写层已就绪)
- **后续**:Step 2b-ii packer 真正接 `TransactionPool`,Step 2b-iii gossip 接 libp2p,Step 2b-iv 删除旧 3 个 offchain_*.rs

---

## 0. 2026-04-29 目录真源

清算行在 node 层统一归入 `offchain` 功能域,前后端目录保持同名:

| 职责 | 后端路径 | 前端路径 |
|---|---|---|
| 功能入口/状态机 | `citizenchain/node/src/offchain/commands.rs` | `citizenchain/node/frontend/offchain/section.tsx` |
| SFID 查询 | `citizenchain/node/src/offchain/sfid.rs` | `citizenchain/node/frontend/offchain/sfid.tsx` |
| 清算行节点声明 | `citizenchain/node/src/offchain/signing.rs` | `citizenchain/node/frontend/offchain/register.tsx` |
| 链上节点查询 | `citizenchain/node/src/offchain/chain.rs` | `citizenchain/node/frontend/offchain/detail.tsx` |
| 管理员解密 | `citizenchain/node/src/offchain/decrypt.rs` | `citizenchain/node/frontend/offchain/admin.tsx` |
| 节点端点信息 | `citizenchain/node/src/offchain/health.rs` | `citizenchain/node/frontend/offchain/node.tsx` |
| DTO 类型 | `citizenchain/node/src/offchain/types.rs` | `citizenchain/node/frontend/offchain/types.ts` |
| 清算行命令 API | `citizenchain/node/src/offchain/commands.rs` | `citizenchain/node/frontend/offchain/api.ts` |
| 清算行页面样式 | 暂无后端 | `citizenchain/node/frontend/offchain/styles.css` |
| 清算行密钥容器 | `citizenchain/node/src/offchain/keystore.rs` | 暂无桌面前端页面 |
| 清算行启动接线 | `citizenchain/node/src/offchain/bootstrap.rs` | 暂无桌面前端页面 |
| 结算引擎 | `citizenchain/node/src/offchain/settlement/*` | 暂无桌面前端页面 |

历史分散的清算行业务目录已删除,后续只使用上表中的 `offchain` 路径。

## 1. 本步范围

Step 2b 拆成 4 个子步,本次交付 **2b-i · 业务逻辑**:
- `offchain/ledger.rs` 补 **核心扫码支付业务**
- `offchain/rpc.rs` 补 **submitPayment RPC 入口**
- `offchain/mod.rs` 补 **组件聚合启动器 `OffchainComponents`**
- `offchain/commands.rs` 补 **Tauri 清算行管理命令入口**

**明确不做**(后续子步):
- `packer::pack_and_submit` 真正构造 extrinsic 提交(Step 2b-ii)
- `gossip.rs` 接 libp2p `NotificationService`(Step 2b-iii)
- `event_listener` 真正订阅 sc-client-api 事件(Step 2b-iii)
- `service.rs` / `rpc.rs` 接入启动路径(Step 2b-ii 或 iii,与 packer/event_listener 一并接)
- 删除旧 `offchain_ledger.rs` / `offchain_packer.rs` / `offchain_gossip.rs`(Step 2b-iv)

---

## 2. 新增/扩展

### 2.1 `ledger.rs`

**新增**:
- `NodePaymentIntent` 结构:与 runtime `PaymentIntent` 字段逐字段对齐的节点层镜像,独立于 pallet crate,避免循环依赖。
- `L3_PAY_SIGNING_DOMAIN = b"GMB_L3_PAY_V1"` 常量,与 runtime 端严格一致。
- `NodePaymentIntent::signing_hash()`:重算签名哈希,用于 `sr25519_verify`。
- `OffchainLedger::accept_payment(intent, sig, current_block, l2_ack_sig)`:完整扫码支付入账(验签 + nonce + 余额 + pending 入账)。
- `OffchainLedger::reject_pending(tx_id)`:packer 失败回滚。
- `OffchainLedger::take_pending_for_batch(max)`:按 `accepted_at` 升序取 pending,供 packer 上链。

**核心不变式**:
- `cached_nonce[payer] + 1 == intent.nonce`(严格单调)
- `confirmed - pending_debit >= amount + fee`(本地可用余额充足)
- `!accepted_tx_ids.contains(tx_id)`(节点级防重)

### 2.2 `rpc.rs`

**新增**:
- `offchain_submitPayment(intent_hex, payer_sig_hex) -> SubmitPaymentResp`
- `SubmitPaymentResp { tx_id, l2_ack_sig, accepted_at }`(Serialize/Deserialize)
- 本地工具 `decode_hex` / `encode_hex`(沿用 wuminapp 端 hex 风格,支持 `0x` 前缀)

### 2.3 `mod.rs` / `commands.rs`

**新增**:
- `OffchainComponents` 聚合 `Arc<Ledger>` / `Arc<Packer>` / `Arc<EventListener>` / `Arc<RpcImpl>`
- `start_clearing_bank_components(base_path, bank_main, password)`:一次性组装 + 磁盘恢复
- `commands.rs`:清算行页面的 SFID 查询、节点查询、端点自测、扫码签名、管理员解密命令。

Step 2b-ii 之后由 `offchain/bootstrap.rs` 统一处理 CLI 参数、密钥解锁、
`start_clearing_bank_components` 调用和三个后台 worker spawn；`service.rs`
只保留节点通用启动接线,并把 `rpc_impl` 注册到 JSON-RPC。

---

## 3. 签名验证流程(Step 2b-i 端到端)

```
wuminapp(Dart)                   清算行节点(Rust,本步)
──────────────                   ─────────────────────────
1. 构造 PaymentIntent
2. SCALE 编码 → intent_bytes
3. msg = blake2_256(DOMAIN || intent_bytes)
4. sig = sr25519_sign(priv, msg)
5. POST offchain_submitPayment(
      intent_hex = hex(intent_bytes),
      payer_sig_hex = hex(sig))
                                 → rpc::submit_payment
                                 → NodePaymentIntent::decode
                                 → intent.signing_hash() 重算
                                 → sr25519_verify(sig, hash, intent.payer)
                                 → query UserBank[payer/recipient] + L2FeeRateBp
                                 → signer.sign_batch(L2_ACK_MESSAGE)
                                 → accept_payment(intent, sig, Some(block), l2_ack, accepted_at)
                                   │ nonce / 余额 / 防重
                                   └ push pending
                                 ← SubmitPaymentResp { tx_id, l2_ack, accepted_at }
```

2026-04-28 补齐: `l2_ack_sig` 已不再是 `[0u8; 64]` 占位。RPC 入口会先拒绝错路由
`recipient_bank`、`UserBank` 绑定漂移和手续费不一致,再用清算行管理员密钥对
`GMB_L2_ACK_V1 || bank_main || SCALE(intent) || payer_sig || accepted_at` 的哈希签名。

## 4. 编译验证

```
$ WASM_FILE=/tmp/dummy_wasm.wasm cargo check -p node
(中间需要先把 offchain-transaction 的 OffchainBatchItemV2 加 DecodeWithMemTracking 派生,本步已修)
```

- `offchain-transaction`:**零 warning / 零 error**
- `node` 的 `offchain/` 子树:**零 warning / 零 error**(grep offchain 0 命中)
- 链接阶段在 `desktop.rs` tauri proc macro(frontend/dist 缺失)受 CI 门禁拦截,与本步无关

## 5. 关键兼容性

| 层 | 约束 |
|---|---|
| 签名哈希 | `blake2_256(b"GMB_L3_PAY_V1" || SCALE(intent))` 逐字节一致:wuminapp Dart / node Rust / runtime pallet |
| `NodePaymentIntent` 字段顺序 | 与 runtime `PaymentIntent` 严格一致(tx_id / payer / payer_bank / recipient / recipient_bank / amount / fee / nonce / expires_at) |
| `PaymentIntent` Hash 语义 | 本步 `intent.tx_id: H256` 由 wuminapp 本地生成(如 `blake2_256(payer||nonce||...)`),runtime 端 `execute_clearing_bank_batch` 通过 `T::Hash::decode(&intent.tx_id.as_bytes())` 兼容 |

## 6. 后续对接清单

Step 2b-ii 要做:
1. `service.rs` 加 CLI flag `--clearing-bank <MAIN_ACCOUNT_SS58>`(或从 chainspec 读);启动时委托 `offchain/bootstrap.rs` 调 `start_clearing_bank_components`,把组件存到 `AppCtx`。
2. `rpc.rs` 构造 RPC io 时 `io.merge(OffchainClearingRpcServer::into_rpc(ctx.rpc_impl.clone()))`。
3. `packer::pack_and_submit` 实现:
   - `ledger.take_pending_for_batch(MaxBatchSize)` → 构造 `OffchainBatchItemV2` 列表
   - 清算行管理员私钥(从 `offchain::keystore::SigningKey`)对 batch 签名
   - 构造 `offchain_transaction::Call::submit_offchain_batch_v2` extrinsic
   - 通过 `TransactionPool` 提交
   - 成功 → ledger 删除已上链 tx;失败 → ledger.reject_pending

Step 2b-iii:
1. `gossip.rs` 新建 libp2p `NotificationService` + `ProtocolName = "/gmb/offchain/1"`
2. `event_listener` 真正订阅 `sc-client-api::BlockchainEvents`
3. `reserve_monitor` 定期 `available_balance vs total_deposits` 对账告警

Step 2b-iv:
1. 删除 `citizenchain/node/src/offchain_{ledger,packer,gossip}.rs`
2. `main.rs` 移除 mod 声明
3. grep 检查残留引用

## 7. 变更记录

- 2026-04-19:Step 2b-i 节点业务逻辑层落地,ledger `accept_payment` / rpc `submitPayment` / mod 启动器就绪,零编译错误。
- 2026-04-29:二次目录收口。`offchain_keystore.rs` 迁入 `offchain/keystore.rs`,
  清算行启动逻辑迁入 `offchain/bootstrap.rs`,前端清算行 API 与样式迁入
  `frontend/offchain/api.ts` / `styles.css`。
