# OFFCHAIN_TECHNICAL(LEGACY · 已下线)

> ⚠️ **Step 2b-iv-b(2026-04-20)已彻底删除** 本文描述的省储行清算体系:
> `submit_offchain_batch` / `enqueue_offchain_batch` / `process_queued_batch` /
> `bind_clearing_institution` / `propose_institution_rate` /
> `vote_institution_rate` / 相关清理 Calls 全部从 runtime 物理移除;
> `InstitutionRateBp` / `RecipientClearingInstitution` / `QueuedBatches` /
> `RateProposalActions` 等 Storage 同步删除。
>
> **当前清算行(L2)体系**技术文档:
> - `memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`(决策)
> - `memory/05-modules/citizenchain/runtime/STEP2B_IV_B_RUNTIME_CLEANUP.md`(清理记录)
> - `memory/05-modules/citizenchain/runtime/STEP2_D_LAYER_B_PALLET_INTEGRATION.md`(集成测试)
> - `memory/05-modules/citizenchain/runtime/transaction/offchain-transaction-pos/STEP2A_RUNTIME.md`(Step 2a 实现)
> - `memory/05-modules/citizenchain/runtime/transaction/offchain-transaction-pos/STEP1_TECHNICAL.md`(Step 1 骨架)
>
> 下方内容仅作历史参考,**不反映当前 runtime 行为**。

模块：`offchain-transaction-pos`(LEGACY)
范围：省储行链下清算批次的上链验证、队列重试、治理配置与清理

## 0. 功能需求

### 0.1 核心职责
- 为省储行链下清算批次提供链上验证、入队、出队执行和审计留痕能力。
- 保证批次执行严格按机构内 `batch_seq` 单调推进，不允许跨序号乱序落账。
- 将链下手续费独立结转到机构 `fee_address`，并支持后续治理归集。

### 0.2 机构与账户模型需求
- 每个收款账户必须先绑定所属清算省储行，未绑定不得作为链下批次收款方。
- 每个机构必须具备独立费率、验签密钥、relay 提交白名单和手续费账户。
- 机构主账户只负责初始化默认验签密钥和 relay 白名单；后续调整必须走内部治理。

### 0.3 批次校验需求
- 批次中的每条交易必须包含 `tx_id`、`payer`、`recipient`、`transfer_amount`、`offchain_fee_amount`。
- `transfer_amount` 必须大于 0，`payer != recipient`，且付款源不能是制度保护地址。
- `tx_id` 必须在批次内唯一，且不能命中已处理窗口或待处理队列索引。
- `recipient` 必须已绑定清算机构，且绑定机构必须与批次机构一致。
- `offchain_fee_amount` 必须严格等于按当前制度费率计算出的链下手续费。

### 0.4 提交与队列需求
- 直接提交路径 `submit_offchain_batch` 与入队路径 `enqueue_offchain_batch` 都必须校验 relay 白名单。
- 直接提交必须在不存在待处理 backlog 的前提下执行。
- 入队时必须验证批次签名、锁定费率快照、锁定验签密钥纪元，并建立 `QueuedTxIndex` 防重。
- 出队重试必须只处理 `Pending` 批次；处理成功、失败、取消都必须留下可观测状态。
- 批次因换钥失效时，只有当前队头批次允许推进执行序号，不能跨序号跳过更早批次。

### 0.5 签名与换钥需求
- 批次签名消息必须固定为 `blake2_256("GMB_OFFCHAIN_BATCH_V1", institution, batch_seq, batch)`。
- 入队和直接提交必须使用当前生效验签密钥完成验签。
- 普通换钥必须走内部投票并延迟生效；紧急换钥必须要求至少两名管理员确认。
- 已入队批次必须记录 `verify_key_epoch_snapshot`；当纪元落后于当前值时，批次必须作废且不得继续执行。

### 0.6 资金与治理需求
- 批次执行时，主金额必须从 `payer` 转给 `recipient`，链下手续费必须从 `payer` 转给机构 `fee_address`。
- `fee_address` 余额只能通过内部治理提案划转到机构主账户，且划转后必须保留最低储备并受单次比例上限约束。
- 费率、验签密钥、relay 白名单、手续费归集都必须通过内部治理提案驱动并留下事件。
- 若 `payer` 或 `fee_address` 命中制度账户白名单边界，还必须通过 `institution-asset-guard` 的资金动作检查。

### 0.7 存储治理与可运维需求
- 模块必须保存 processed tx、防重窗口、队列记录、批次摘要和提案动作映射，支持手动与自动清理。
- `on_initialize` 必须负责普通换钥激活，`on_idle` 必须在剩余权重内做有界清理。
- 清理 pending 队列或 stale 队列时，必须遵守“只能推进队头序列”的约束，避免破坏执行顺序。

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

## 7.1 治理提案执行与 STATUS_EXECUTED

四类治理动作（rate、verify_key、sweep、relay_submitters）在投票通过后由投票引擎回调本模块执行。执行成功后，本模块调用 `voting_engine::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)` 将投票引擎侧的提案状态标记为已执行，防止同一提案被重复执行。

各动作执行函数：
- `try_execute_rate`：执行成功后调用 `set_status_and_emit(proposal_id, STATUS_EXECUTED)`
- `try_execute_verify_key`：执行成功后调用 `set_status_and_emit(proposal_id, STATUS_EXECUTED)`
- `try_execute_sweep`：执行成功后调用 `set_status_and_emit(proposal_id, STATUS_EXECUTED)`
- `try_execute_relay_submitters`：执行成功后调用 `set_status_and_emit(proposal_id, STATUS_EXECUTED)`

提案状态流转：`VOTING → PASSED → EXECUTED`

行为变更：
- 执行成功后不再立即删除 `ProposalActions` 中的动作数据，而是保留原始数据用于审计。
- 动作数据由投票引擎的 90 天延迟清理机制统一回收，避免执行后立即丢失提案上下文。

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
