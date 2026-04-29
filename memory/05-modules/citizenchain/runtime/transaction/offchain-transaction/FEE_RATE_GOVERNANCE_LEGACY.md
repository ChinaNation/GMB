# 省储行费率治理(LEGACY · 已下线)

> ⚠️ **Step 2b-iv-b(2026-04-20)已彻底删除** ADR-006 宣布退出的"省储行即时清算"
> 体系。本文件描述的 `propose_institution_rate` / `vote_institution_rate` /
> `InstitutionRateBp` / `RateProposalActions` 等 Call/Storage 均已从 runtime
> 物理移除。当前清算行(L2)体系费率治理见:
>
> - `call_index 40 propose_l2_fee_rate` + `41 set_max_l2_fee_rate`(Root / 联合投票)
> - Storage `L2FeeRateBp` / `L2FeeRateProposed` / `MaxL2FeeRateBp`
> - 延迟 7 天生效机制由 `on_initialize` + `fee_config::activate_pending_rates` 落实
> - 技术文档:`STEP2B_IV_B_RUNTIME_CLEANUP.md`(清理记录)与 runtime pallet
>   源码 `src/fee_config.rs`
>
> 下方内容仅作历史参考。

## 概述

各省储行的链下交易费率通过内部投票（InternalVoteEngine）进行治理。省储行管理员（PRB admin）可发起费率变更提案，经内部投票通过后自动生效。

## 费率范围

| 参数 | 值 | 说明 |
|------|------|------|
| OFFCHAIN_RATE_BP_MIN | 1 bp | 0.01% |
| OFFCHAIN_RATE_BP_MAX | 10 bp | 0.1% |
| BP_DENOMINATOR | 10,000 | 基点转换分母 |
| OFFCHAIN_MIN_FEE | 1 分 | 单笔最低手续费 0.01 元 |

费率单位为基点（bp），1 bp = 0.01%。合法范围 1~10 bp，对应 0.01%~0.1%。

## 默认费率

未设置费率的省储行（`InstitutionRateBp` 存储值为 0）按最低费率 `OFFCHAIN_RATE_BP_MIN`（1 bp = 0.01%）执行。由 `ensure_rate_and_institution` 内部处理。

## 存储

```rust
// 各省储行链下清算费率（bp，范围1~10）
pub type InstitutionRateBp<T> =
    StorageMap<_, Blake2_128Concat, InstitutionPalletId, u32, ValueQuery>;

// 费率治理提案动作
pub type RateProposalActions<T: Config> =
    StorageMap<_, Blake2_128Concat, u64, RateProposalAction, OptionQuery>;

pub struct RateProposalAction {
    pub institution: InstitutionPalletId,
    pub new_rate_bp: u32,
}
```

## 提案流程

### 1. 发起提案（propose_institution_rate）

- **调用者**：省储行管理员（PRB admin，通过 `is_prb_admin` 验证）
- **参数**：institution（省储行 PalletId）、new_rate_bp（目标费率）
- **校验**：new_rate_bp 必须在 1~10 范围内
- **操作**：
  1. 通过 InternalVoteEngine 创建内部提案（org=ORG_PRB）
  2. 将 RateProposalAction 写入 RateProposalActions 存储
  3. 触发 InstitutionRateProposed 事件

### 2. 投票（vote_institution_rate）

- **调用者**：同一省储行的其他管理员
- **参数**：proposal_id、approve（赞成/反对）
- **操作**：
  1. 验证 proposal_id 对应的 RateProposalAction 存在
  2. 验证调用者是该省储行管理员
  3. 调用 InternalVoteEngine::cast_internal_vote 记录投票
  4. 若赞成票且提案状态变为 PASSED，立即尝试执行

### 3. 自动执行（try_execute_rate）

投票通过后在同一交易中自动执行，使用 `with_transaction` 保证原子性：

1. 验证提案状态为 PASSED、kind 为 INTERNAL、institution 匹配
2. 将 new_rate_bp 写入 `InstitutionRateBp` 存储
3. 触发 InstitutionRateUpdated 事件
4. 设置提案状态为 EXECUTED

若执行失败，回滚并触发 InternalProposalExecutionFailed 事件。

## 费用计算公式

```
fee = max(amount * rate_bp / 10000, OFFCHAIN_MIN_FEE)
```

node 端 `calc_offchain_fee` 与链上计算逻辑保持一致。

## 提案清理

过期或已执行的提案可通过 `prune_rate_proposal`（call_index=12）清理，移除 RateProposalActions 中的记录。

## 源码位置

- `citizenchain/runtime/transaction/offchain-transaction/src/lib.rs`
  - `propose_institution_rate`（call_index=1）
  - `vote_institution_rate`（call_index=2）
  - `try_execute_rate`（内部方法）
  - `ensure_rate_and_institution`（内部方法）
