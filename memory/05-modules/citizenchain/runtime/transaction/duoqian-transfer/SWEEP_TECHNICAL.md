# 手续费划转 (Fee Sweep) 技术文档

## 概述

`duoqian-transfer` 模块中的 sweep 子功能，将机构手续费账户 (`fee_address`) 余额划转至机构主账户 (`main_address`)。仅限国储会 (NRC) 和省储行 (PRB) 使用，注册多签机构不支持。

## 数据结构

```rust
pub struct SweepAction<Balance> {
    pub institution: InstitutionPalletId,  // 机构标识 (48 字节)
    pub amount: Balance,                    // 划转金额 (分)
    pub proposer: AccountId,                 // 提案人
}
```

存储：`SweepProposalActions<T>` — 独立 `StorageMap<u64, SweepAction>`；投票引擎 `ProposalOwner` 仍绑定 `duoqian-transfer` 的 `MODULE_TAG`。

## 常量

| 常量 | 值 | 含义 |
|------|-----|------|
| `FEE_ADDRESS_MIN_RESERVE_FEN` | 111,111 | 手续费账户最低保留 1111.11 元 |
| `FEE_SWEEP_MAX_PERCENT` | 80 | 单次划转上限：可用余额的 80% |

## 权限控制

- **发起者**：NRC 或 PRB 管理员（通过 `InternalAdminProvider::is_internal_admin` 校验）
- **org 判断**：`resolve_sweep_org` 仅识别 NRC（CHINA_CB 首项）和 PRB（CHINA_CH 全部），返回对应 org 常量
- 注册多签机构 (ORG_DUOQIAN) 不在 sweep 范围内

## 账户解析

- `resolve_fee_account`：NRC 取 `CHINA_CB[0].fee_address`，PRB 取对应 `CHINA_CH` 节点的 `fee_address`
- `resolve_main_account`：通过 `institution_pallet_address` 查 `main_address`

## 提案/投票/执行流程

### 1. propose_sweep_to_main (call_index=2)

1. 校验调用者为对应机构管理员
2. 通过 `InternalVoteEngine::create_internal_proposal_with_data` 创建提案，并绑定 owner/data/meta（获取 proposal_id）
3. 写入 `SweepProposalActions` 存储
4. 触发 `SweepToMainProposed` 事件

### 2. 投票

管理员统一调用 `VotingEngine::internal_vote(proposal_id, approve)`。投票引擎使用创建时的管理员快照和固定阈值判定，达阈值后回调本模块自动执行 sweep。

### 3. try_execute_sweep（内部方法）

1. 校验提案状态为 `STATUS_PASSED`
2. `InstitutionAsset::can_spend` 检查（action = `OffchainFeeSweepExecute`）
3. 计算手续费：`calculate_onchain_fee(amount)` — 费率 0.1%，有最低值
4. **余额检查**：`fee_balance >= amount + tx_fee + reserve (111,111 fen)`
5. **Cap 检查**：`amount <= (fee_balance - reserve) * 80 / 100`
6. 执行 `Currency::transfer` 从 fee_account 到 main_account（KeepAlive）
7. 执行 `Currency::withdraw` 扣取手续费
8. 手续费通过 `FeeRouter`（即 `TransferFeeRouter` -> `OnchainFeeRouter`）按 80/10/10 分账
9. 触发 `SweepToMainExecuted` 事件（含 `reserve_left` 余额）
10. 返回 `ProposalExecutionOutcome::Executed`，由投票引擎设置提案状态为 `STATUS_EXECUTED`

## 手续费分账路径

`TransferFeeRouter` 将旧 `NegativeImbalance` 转换为新 `Credit`，传递给 `OnchainFeeRouter`：
- 80% -> 当前区块矿工（PoW 全节点）
- 10% -> 国储会费用账户
- 10% -> 国储会安全基金账户

## 错误码

| 错误 | 触发条件 |
|------|----------|
| `InvalidSweepAmount` | 金额为 0 |
| `InvalidInstitution` | 机构非 NRC/PRB |
| `UnauthorizedAdmin` | 调用者非管理员 |
| `SweepProposalNotFound` | proposal_id 无对应记录 |
| `SweepProposalNotPassed` | 提案未通过 |
| `InsufficientFeeReserve` | 余额不足以覆盖划转+手续费+保留 |
| `SweepAmountExceedsCap` | 超过可用余额 80% 上限 |
| `InstitutionSpendNotAllowed` | 资产保护检查未通过 |
