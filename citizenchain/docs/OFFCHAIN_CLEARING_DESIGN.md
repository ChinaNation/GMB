# 省储行链下清算系统设计（定稿）

## 1. 目标
- 用户发起链下交易后，由收款方绑定的省储行执行清算。
- 省储行确认后即时生效：
  - 付款方余额实时减少 `amount + fee`
  - 收款方余额实时增加 `amount`
  - 省储行手续费账户实时增加 `fee`
- 一旦确认，交易结果绝对不可修改。
- 已确认交易必须最终被打包上链（持久化重试，直到成功）。

## 2. 业务规则（硬约束）
- 清算省储行选择权在收款方。
- 收款方每 `87600` 区块最多更换 1 次清算省储行。
- 不允许冲正交易；错单由用户与商户线下解决，系统不提供撤销/改写。
- 账户统一，不区分链上/链下账户。
- 不做限额、不做冻结。
- 手续费规则沿用现有制度参数。
- 打包阈值沿用当前实现。

## 3. 核心对象
- `MerchantClearingBinding`
  - `merchant_account`
  - `institution`（省储行 institution id）
  - `last_switch_block`
  - `version`

- `OffchainLedgerTx`
  - `tx_id`（建议：`hash(T2 || institution || merchant_order_id || payer || payee || amount || fee || ts)`）
  - `t2`（从 shenfen_id 提取）
  - `institution`
  - `payer`
  - `payee`
  - `amount`
  - `fee`
  - `confirmed_at`
  - `status`：`CONFIRMED | PACKED`

- `PackOutbox`
  - `id`
  - `institution`
  - `batch_seq`
  - `tx_ids[]`
  - `next_retry_at`
  - `retry_count`
  - `status`：`PENDING | SUBMITTED | FINALIZED`

## 4. 状态机
- 交易状态机：
  - `INIT -> CONFIRMED -> PACKED`
- 约束：
  - `CONFIRMED` 写入后不可改，不允许回退。
  - 同一 `(t2, tx_id)` 幂等，重复请求直接返回已确认结果。

## 5. 清算流程（链下）
1. 校验收款方绑定的省储行。
2. 计算手续费（沿制度参数）。
3. 原子记账（单事务）：
   - 借记 payer：`amount + fee`
   - 贷记 payee：`amount`
   - 贷记省储行 fee 账户：`fee`
   - 写入 `OffchainLedgerTx(status=CONFIRMED)`
4. 实时返回确认结果（不可改）。
5. 异步进入 `PackOutbox`，等待按阈值打包。

## 6. 打包与上链（必须成功）
- 使用持久化重试队列（Outbox Pattern）：
  - 达到阈值生成批次。
  - 失败不回滚已确认交易，只增加 `retry_count` 并指数退避重试。
  - 直到链上成功确认后，将相关 `OffchainLedgerTx` 标记为 `PACKED`。
- 幂等键：`(institution, batch_seq)` 与 `(t2, tx_id)` 双重保护。

## 7. T2 提取与使用
- `T2` 来源：`shenfen_id` 的第三段前两位（示例 `SFR-ZS001-CH1Z-...` 中 `CH`）。
- 在节点软件中由运行时内置常量提供，运行时按 institution 查到对应 shenfen_id 后提取。
- 去重维度采用 `(T2, tx_id)`，避免跨省误伤。

## 8. 接口草案
- `bind_clearing_institution(merchant, institution)`
  - 校验：距上次切换 >= 87600 blocks。
- `submit_offchain_trade(payer, payee, merchant_order_id, amount, ts)`
  - 直接执行链下确认并返回。
- `query_offchain_trade(tx_id)`
  - 返回确认信息与打包状态。
- `run_pack_worker(institution)`
  - 后台任务，持续处理 `PackOutbox`。

## 9. 一致性与审计
- 链下账本必须 append-only（写后不可改）。
- 所有确认事件都落审计日志（含 operator、trace id、签名摘要）。
- 对账任务：按批次核对 `CONFIRMED` 与链上 `PACKED` 数量和金额一致性。

## 10. 实施顺序
1. 商户清算绑定与年度切换限制。
2. 链下清算原子记账与不可改事件存储。
3. `(T2, tx_id)` 幂等防重。
4. Outbox 持久化队列与重试。
5. 打包确认回写 `PACKED`。
6. 对账与审计报表。
