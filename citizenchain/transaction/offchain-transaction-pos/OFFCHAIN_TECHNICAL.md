# OFFCHAIN_TECHNICAL

模块：`offchain-transaction-pos`  
范围：省储行链下清算批次的上链验证、队列重试、治理配置与清理

## 1. 目标与边界
- 本模块不负责链下“确认终态”，仅负责批次上链执行与治理控制。
- 链下系统负责业务撮合和不可变确认；本模块负责链上可审计落账。
- 上链手续费由机构手续费账户承担。

## 2. 核心业务口径
1. 收款账户需先绑定清算省储行。
2. 机构费率由治理维护，范围 `1..=10 bp`。
3. 链下已确认交易可按批次上链（直接提交或先入队后处理）。
4. 每条批次项包含：
   - `tx_id`
   - `payer`
   - `recipient`
   - `transfer_amount`
   - `offchain_fee_amount`
5. 要求：
   - `transfer_amount > 0`
   - `payer != recipient`
   - `tx_id` 在批次内唯一
   - `recipient` 已绑定且绑定机构一致

## 3. 签名与验证密钥
- 批次消息为 `blake2_256("GMB_OFFCHAIN_BATCH_V1", institution, batch_seq, batch)`。
- 入队路径必须验签（当前机构生效密钥）。
- 出队处理路径不重复验签；依赖入队验签结果与密钥纪元机制防止旧签名继续执行。

### 3.1 密钥轮换（普通）
- 普通治理轮换在通过后进入 `PendingVerifyKeys`，延迟生效。
- `on_initialize` 到激活高度时切换到新 key，并递增 `VerifyKeyEpoch`。
- 若机构已有 pending 轮换，再次通过新轮换会被拒绝（防覆盖）。

### 3.2 密钥轮换（紧急）
- `emergency_rotate_verify_key` 采用双管理员确认（至少 `EMERGENCY_ROTATE_MIN_ADMINS=2`）。
- 支持 `cancel_emergency_rotation_approval` 撤回审批。
- 紧急轮换成功后：
  - 立即替换当前 key
  - 清空 `PendingVerifyKeys`
  - 从 `PendingRotationInstitutions` 主动移除机构
  - 递增 `VerifyKeyEpoch`
  - 清空该机构下所有紧急审批键空间（`clear_prefix`）

## 4. 防重放与队列
- 已执行交易防重：`ProcessedOffchainTx + ProcessedOffchainTxAt`（带保留窗口）。
- 待处理队列防重：`QueuedTxIndex` 防止跨入队批次重复 `tx_id`。
- 批次顺序：
  - 执行序号：`LastBatchSeq`
  - 入队序号：`NextEnqueueBatchSeq`
- 队列状态：`Pending | Processed | Failed | Cancelled`。

### 4.1 密钥纪元失效
- 入队时记录 `verify_key_epoch_snapshot`。
- 处理时若与当前 `VerifyKeyEpoch` 不一致，批次会自动标记 `Cancelled` 并释放 `QueuedTxIndex`，不会执行资金转移。

## 5. 执行与重试
- 直接路径：`submit_offchain_batch`（事务包裹执行）。
- 队列路径：
  - `enqueue_offchain_batch`（持久化）
  - `process_queued_batch`（重试执行）
- 失败策略：
  - 可重试错误保留队列并累计 `retry_count`
  - 超过 `MAX_QUEUE_RETRY_COUNT` 变为 `Failed`
  - 管理员可 `skip_failed_batch` 推进序号
  - 管理员可 `cancel_queued_batch` 取消队头 pending 批次
  - 管理员可 `cancel_stale_queued_batches` 批量取消过期 pending（按队头可推进序列原则）

## 6. 清理与存储治理
- 手动清理入口：
  - `prune_queued_batch`
  - `prune_batch_summary`
  - `prune_processed_tx`
  - `prune_expired_proposal_action`
- 自动清理入口：
  - `on_idle` 有界清理 `processed/queued/summary`
- 过期 pending 队列：
  - `auto_prune_one_queued_batch` 对过期 pending 会直接清理存储与 `QueuedTxIndex`。
  - `prune_queued_batch` 对 pending 也可用 `enqueued_at` 判定过期后清理。

## 7. 治理动作
- 费率治理：`propose/vote_institution_rate`。
- 验签密钥治理：`propose/vote_verify_key`。
- 机构资金归集治理：`propose/vote_sweep_to_main`。
- relay 白名单治理：`propose/vote_relay_submitters`。
- 对“已通过但执行失败”的提案：`retry_execute_proposal`。

## 8. 权重与完整性约束
- 关键权重按最坏项上界声明（含 `MaxBatchSize` 影响）。
- 关键常量完整性在 `integrity_test` 中断言：
  - 费率上下界
  - 紧急管理员阈值
  - 分母与阈值非零
  - `MaxBatchSize > 0`

## 9. 存储版本
- pallet 已声明 `#[pallet::storage_version]`，当前版本为 `1`，用于未来安全迁移判定。

## 10. 关键事件（观测建议）
- 批次：`Submitted/Queued/Processed/Failed/Cancelled/Pruned`
- 紧急换钥：`Approval`、`ApprovalCancelled`、`EmergencyRotated`
- 密钥普通轮换：`RotationScheduled`、`VerifyKeyRotated`
- 治理执行：`InternalProposalExecutionFailed`、`ProposalExecutionRetried`

## 11. 当前测试覆盖（摘要）
- 紧急换钥双管理员门限、审批撤回、单审批不生效。
- 密钥纪元失效导致已入队批次自动取消。
- pending key 不覆盖保护。
- retry_execute_proposal 成功/失败分支。
- on_idle 自动清理（含 stale pending）与 on_initialize 轮换 epoch 递增。
- 批量取消 stale pending、序列推进与队列行为一致性。
