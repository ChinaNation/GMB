# 手续费划转 (Fee Sweep) 技术文档

## 概述

`multisig-transfer` 模块中的 sweep 子功能，将机构手续费账户 (`fee_account`) 余额划转至机构主账户 (`main_account`)。仅限国家储委会 (NRC) 和省储行 (PRB) 使用，注册多签机构不支持。

## 数据结构

```rust
pub struct SweepAction<AccountId, Balance> {
    pub actor_cid_number: CidNumber,       // 机构唯一主键
    pub institution_account: AccountId,    // 实际转出的费用账户
    pub amount: Balance,                   // 划转金额（分）
    pub proposer: AccountId,               // 签名管理员
}
```

存储：`SweepProposalActions<T>` — 独立 `StorageMap<u64, SweepAction>`；投票引擎 `ProposalOwner` 仍绑定 `multisig-transfer` 的 `MODULE_TAG`。

## 常量

| 常量 | 值 | 含义 |
|------|-----|------|
| `FEE_SWEEP_MAX_PERCENT` | 80 | 单次划转上限：可用余额的 80% |

费用账户没有固定预存金额；支出后只要求余额不低于链上 `ED`。

## 权限控制

- **发起者**：显式 `actor_cid_number` 对应的 NRC 或 PRB 当前管理员（通过 `InternalAdminProvider::is_institution_admin` 校验）
- **机构码判断**：`resolve_sweep_org(actor_cid_number)` 从 CID 解析机构码，仅允许 NRC 和 PRB
- 注册账户（个人多签码 PMUL，`is_personal_code`）不在 sweep 范围内

## 账户解析

- `resolve_fee_account(actor_cid_number)`：用 `AccountKind::InstitutionFee` 从 CID 确定性派生费用账户
- `resolve_main_account(actor_cid_number)`：用 `AccountKind::InstitutionMain` 从同一 CID 确定性派生主账户
- 外部参数 `institution_account` 必须等于派生费用账户；主账户只是划转目标，二者都不能替代 CID 作为机构身份

## 提案/投票/执行流程

### 1. propose_sweep_to_main (call_index=2)

1. 接收 `actor_cid_number + institution_account + amount`，校验 CID 属于 NRC/PRB、账户等于该 CID 的费用账户、调用者为该 CID 的管理员
2. 通过 `InternalVoteEngine::create_institution_proposal_with_data` 创建提案，并绑定 CID、执行账户、owner/data/meta（获取 proposal_id）
3. 写入 `SweepProposalActions` 存储
4. 触发 `SweepToMainProposed` 事件

### 2. 投票

管理员统一调用 `InternalVote::cast(proposal_id, approve)`。投票引擎使用创建时的管理员快照和固定阈值判定，达阈值后回调本模块自动执行 sweep。

### 3. try_execute_sweep（内部方法）

1. 校验提案状态为 `STATUS_PASSED`
2. `InstitutionAsset::can_spend` 检查（action = `OffchainFeeSweepExecute`）
3. 计算手续费：`calculate_onchain_fee(amount)` — 费率 0.1%，有最低值
4. **余额检查**：划转和手续费支出后，费用账户余额必须 `>= ED`
5. **Cap 检查**：`amount <= (fee_balance - ED) * 80 / 100`
6. 执行 `Currency::transfer` 从 fee_account 到 main_account（KeepAlive）
7. 执行 `Currency::withdraw` 扣取手续费
8. 手续费通过 `FeeRouter`（即 `TransferFeeRouter` -> `OnchainFeeRouter`）按 80/10/10 分账
9. 触发 `SweepToMainExecuted` 事件（含 `reserve_left` 余额）
10. 返回 `ProposalExecutionOutcome::Executed`，由投票引擎设置提案状态为 `STATUS_EXECUTED`

## 手续费分账路径

`TransferFeeRouter` 将旧 `NegativeImbalance` 转换为新 `Credit`，传递给 `OnchainFeeRouter`：
- 80% -> 当前区块矿工（PoW 全节点）
- 10% -> 国家储委会费用账户
- 10% -> 国家储委会安全基金账户

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
