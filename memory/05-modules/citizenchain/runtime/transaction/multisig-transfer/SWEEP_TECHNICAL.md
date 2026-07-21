# 手续费划转 (Fee Sweep) 技术文档

## 概述

`multisig-transfer` 模块中的 sweep 子功能，将机构手续费账户 (`fee_account`) 余额划转至机构主账户 (`main_account`)。仅限国家储委会 (NRC) 和省储行 (PRB) 使用，注册多签机构不支持。

## 数据结构

```rust
pub struct SweepAction<AccountId, Balance> {
    pub actor_cid_number: CidNumber,       // 机构唯一主键
    pub institution_account: AccountId,    // 实际转出的费用账户
    pub amount: Balance,                   // 划转金额（分）
    pub proposer: AccountId,               // 签名岗位任职人
}
```

存储：`SweepProposalActions<T>` — 独立 `StorageMap<u64, SweepAction>`；投票引擎 `ProposalOwner` 仍绑定 `multisig-transfer` 的 `MODULE_TAG`。

## 常量

| 常量 | 值 | 含义 |
|------|-----|------|
| `FEE_SWEEP_MAX_PERCENT` | 80 | 单次划转上限：可用余额的 80% |

费用账户没有固定预存金额；支出后只要求余额不低于链上 `ED`。

## 权限控制

- **发起者**：显式 `actor_cid_number + proposer_role_code` 对应、拥有 sweep `Propose` 权限的 NRC 或 PRB 岗位有效任职人
- **机构码判断**：`resolve_sweep_org(actor_cid_number)` 从 CID 解析机构码，仅允许 NRC 和 PRB
- 注册账户（个人多签码 PMUL，`is_personal_code`）不在 sweep 范围内

## 账户解析

- `resolve_fee_account(actor_cid_number)`：从机构账户真源精确读取 `(actor_cid_number, InstitutionFee)`，并由正反索引聚合查询保证归属一致
- `resolve_main_account(actor_cid_number)`：从同一机构账户真源精确读取 `(actor_cid_number, InstitutionMain)`
- 外部参数 `institution_account` 必须等于查询到的费用账户；主账户只是划转目标，二者都不能替代 CID 作为机构身份

## 提案/投票/执行流程

### 1. propose_sweep_to_main (call_index=2)

1. 接收 `actor_cid_number + proposer_role_code + institution_account + amount`，校验 CID 属于 NRC/PRB、账户等于该 CID 的费用账户，并按完整岗位主体校验业务提案权限
2. 查询同一业务 `Vote` 权限岗位、构造 `VotePlan`，通过 `InternalVoteEngine::create_institution_proposal_with_data` 创建提案，并绑定 CID、执行账户、岗位快照、owner/data/meta（获取 proposal_id）
3. 写入 `SweepProposalActions` 存储
4. 触发 `SweepToMainProposed` 事件

### 2. 投票

岗位有效选民统一调用 `InternalVote::cast(proposal_id, InstitutionRole(role_code), approve)`。投票引擎使用创建时对应岗位的 `VoterSnapshot`、完整岗位票据和机构阈值判定，不建立岗位阈值；达阈值后回调本模块自动执行 sweep。

### 3. try_execute_sweep（内部方法）

1. 校验提案状态为 `STATUS_PASSED`
2. `InstitutionAsset::can_spend` 检查（action = `OffchainFeeSweepExecute`）
3. 计算手续费：`calculate_onchain_fee(amount)` — 费率 0.1%，有最低值
4. **余额检查**：划转和手续费支出后，费用账户余额必须 `>= ED`
5. **Cap 检查**：`amount <= (fee_balance - ED) * 80 / 100`
6. 在同一 storage transaction 中先由 `OnchainFeeCharger::charge(fee_account, amount)` 收取执行手续费，再执行 `Currency::transfer` 从 fee_account 到 main_account（KeepAlive）
7. 任一失败时手续费、分账和本金划转全部回滚
8. 成功手续费经 `OnchainExecutionFeeDistributor` -> `OnchainFeeRouter` 按 80/10/10 分账
9. 触发 `SweepToMainExecuted` 事件（含 `reserve_left` 余额）
10. 返回 `ProposalExecutionOutcome::Executed`，由投票引擎设置提案状态为 `STATUS_EXECUTED`

## 手续费分账路径

`OnchainExecutionFeeDistributor` 将执行期 `NegativeImbalance` 等额转换为 `Credit`，传递给 `OnchainFeeRouter`：
- 80% -> 当前区块矿工（PoW 全节点）
- 10% -> 国家储委会费用账户
- 10% -> 国家储委会安全基金账户

## 错误码

| 错误 | 触发条件 |
|------|----------|
| `InvalidSweepAmount` | 金额为 0 |
| `InvalidInstitution` | 机构非 NRC/PRB |
| `UnauthorizedAdmin` | 稳定错误码：调用者不是所提交岗位的有效任职人，或该岗位没有 sweep 提案权限 |
| `SweepProposalNotFound` | proposal_id 无对应记录 |
| `SweepProposalNotPassed` | 提案未通过 |
| `InsufficientFeeReserve` | 余额不足以覆盖划转+手续费+保留 |
| `SweepAmountExceedsCap` | 超过可用余额 80% 上限 |
| `InstitutionSpendNotAllowed` | 资产保护检查未通过 |
