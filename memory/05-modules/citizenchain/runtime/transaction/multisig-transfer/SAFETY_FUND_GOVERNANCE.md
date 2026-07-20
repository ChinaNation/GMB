# 安全基金治理

## 概述

国家储委会安全基金（SAFETY_FUND_ACCOUNT）的转账通过内部投票治理。仅 NRC 中拥有安全基金转账 `Propose` 权限的岗位有效任职人可发起，经岗位快照选民投票通过后自动执行转账并扣除手续费。

## 安全基金账户

```rust
pub const SAFETY_FUND_ACCOUNT: [u8; 32] =
    hex!("045bdb35046c60c1346ba48e1e79049519edf4c009e40c7ecead1bebd1884a37");
```

账户派生方式：`BLAKE2-256(GMB + OP_SAFETY + SS58_PREFIX_LE + 国家储委会 cid_number)`，详见 BLAKE2_ADDRESS_DERIVATION.md。

## 存储

```rust
pub type SafetyFundProposalActions<T: Config> = StorageMap<
    _, Blake2_128Concat, u64,
    SafetyFundAction<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>,
    OptionQuery,
>;

pub struct SafetyFundAction<AccountId, Balance, MaxRemarkLen> {
    pub actor_cid_number: CidNumber, // 国家储委会唯一 CID
    pub institution_account: AccountId, // 安全基金账户
    pub beneficiary: AccountId,   // 收款地址
    pub amount: Balance,          // 转账金额
    pub remark: BoundedVec<u8, MaxRemarkLen>, // 备注
    pub proposer: AccountId,      // 提案人
}
```

## 提案流程

### 1. 发起提案（propose_safety_fund_transfer，call_index=1）

- **调用者**：国家储委会中拥有该业务提案权限的岗位有效任职人
- **参数**：actor_cid_number、proposer_role_code、institution_account、beneficiary（收款地址）、amount（金额）、remark（备注）
- **校验**：
  1. 金额大于零
  2. `actor_cid_number` 必须是国家储委会 CID，`institution_account` 必须是该 CID 下的安全基金账户；调用者必须对完整 `RoleSubject(actor_cid_number, proposer_role_code)` 具有该业务 `Propose` 权限
  3. InstitutionAsset::can_spend 检查安全基金账户支出权限（NrcSafetyFundTransfer）
  4. **分账户余额预检**：安全基金账户必须覆盖 `amount + ED`，国家储委会费用账户必须覆盖 `fee + ED`
- **手续费预算**：使用 `calculate_onchain_fee(amount)` 计算，即 `max(amount * 0.1%, 0.1 元)`
- **操作**：
  1. 查询同一业务 `Vote` 权限岗位并构造内部 `VotePlan`，通过 `InternalVoteEngine::create_institution_proposal_with_data` 创建提案，绑定 CID、执行账户、岗位快照、owner/data/meta
  2. 将 `SafetyFundAction` 写入独立存储
  3. 触发 `SafetyFundTransferProposed` 事件

### 2. 投票

- 提案岗位有效选民统一调用 `InternalVote::cast(proposal_id, approve)`。
- 投票引擎使用提案创建时锁定的 `EffectiveVoterSnapshot` 和固定 NRC 机构阈值判定；不新增岗位阈值。
- 达阈值后回调本模块自动执行安全基金转账。

### 3. 自动执行（try_execute_safety_fund）

投票通过后在 `with_transaction` 中原子执行：

1. 验证提案状态为 PASSED
2. 解码 SAFETY_FUND_ACCOUNT 为 AccountId
3. InstitutionAsset::can_spend 再次检查支出权限
4. **计算手续费**：`fee = max(amount * 0.1%, 0.1 元)`
5. **分账户余额检查**：安全基金账户覆盖 `amount + ED`，actor CID 的费用账户覆盖 `fee + ED`
6. 在同一 storage transaction 中从费用账户调用 `OnchainFeeCharger::charge`，并从安全基金账户执行 `Currency::transfer`（KeepAlive）
7. 任一失败全部回滚；成功手续费按 80/10/10 分账，`SafetyFundTransferExecuted.fee_payer` 记录国家储委会费用账户
8. 返回 `ProposalExecutionOutcome::Executed`，由投票引擎设置提案状态为 EXECUTED

## 手续费分账（统一执行期收费器 80/10/10）

安全基金转账本金由安全基金账户支出，执行手续费只由国家储委会费用账户支出，随后进入统一分账：

| 比例 | 接收方 | 说明 |
|------|--------|------|
| 80% | 全节点 | ONCHAIN_FEE_FULLNODE_PERCENT |
| 10% | 国家储委会手续费账户 | ONCHAIN_FEE_NRC_PERCENT |
| 10% | 安全基金账户 | ONCHAIN_FEE_SAFETY_FUND_PERCENT |

手续费率固定为链上费率 0.1%（ONCHAIN_FEE_RATE），单笔最低 0.1 元（ONCHAIN_MIN_FEE）。

## 执行失败处理

若 try_execute_safety_fund 失败：
- `with_transaction` 回滚所有状态变更
- 触发 SafetyFundExecutionFailed 事件
- 提案保持 `STATUS_PASSED` 并进入 votingengine retry state
- 岗位有效选民快照成员通过 `VotingEngine::retry_passed_proposal`（pallet 9.4）手动重试
- 3 次手动失败或超过 `ExecutionRetryGraceBlocks` 后,投票引擎统一转 `STATUS_EXECUTION_FAILED`

## 源码位置

- `citizenchain/runtime/transaction/multisig/src/lib.rs`
  - `propose_safety_fund_transfer`(call_index=1)
  - `try_execute_safety_fund_from_callback`(内部方法,投票通过后由 InternalVoteExecutor 回调触发)
- `citizenchain/runtime/primitives/cid/china/china_cb.rs` - SAFETY_FUND_ACCOUNT 定义
