# 安全基金治理

## 概述

国储会安全基金（NRC_ANQUAN_ADDRESS）的转账通过内部投票治理。仅国储会管理员（NRC admin）可发起提案，经多签投票通过后自动执行转账并扣除手续费。

## 安全基金账户

```rust
pub const NRC_ANQUAN_ADDRESS: [u8; 32] =
    hex!("045bdb35046c60c1346ba48e1e79049519edf4c009e40c7ecead1bebd1884a37");
```

地址派生方式：`BLAKE2-256(DUOQIAN_DOMAIN + OP_AN + SS58_PREFIX_LE + 国储会 shenfen_id)`，详见 BLAKE2_ADDRESS_DERIVATION.md。

## 存储

```rust
pub type SafetyFundProposalActions<T: Config> = StorageMap<
    _, Blake2_128Concat, u64,
    SafetyFundAction<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>,
    OptionQuery,
>;

pub struct SafetyFundAction<AccountId, Balance, MaxRemarkLen> {
    pub beneficiary: AccountId,   // 收款地址
    pub amount: Balance,          // 转账金额
    pub remark: BoundedVec<u8, MaxRemarkLen>, // 备注
    pub proposer: AccountId,      // 提案人
}
```

## 提案流程

### 1. 发起提案（propose_safety_fund_transfer，call_index=3）

- **调用者**：国储会管理员（ORG_NRC admin）
- **参数**：beneficiary（收款地址）、amount（金额）、remark（备注）
- **校验**：
  1. 金额大于零
  2. 调用者是国储会管理员（通过 InternalAdminProvider::is_internal_admin 验证，org=ORG_NRC）
  3. InstitutionAssetGuard::can_spend 检查安全基金账户支出权限（NrcSafetyFundTransfer）
  4. **余额预检**：`free_balance >= amount + fee + ED`，避免创建必定无法执行的提案
- **手续费预算**：使用 `calculate_onchain_fee(amount)` 计算，即 `max(amount * 0.1%, 0.1 元)`
- **操作**：
  1. 通过 InternalVoteEngine 创建内部提案（org=ORG_NRC）
  2. 将 SafetyFundAction 写入存储
  3. 触发 SafetyFundTransferProposed 事件

### 2. 离线聚合代投(finalize_safety_fund_transfer,call_index=4,Step 2 · 2026-04-21)

- **调用者**:任意签名账户(代付 gas;发起人身份已在 Tx 1 锁定)
- **输入**:`(proposal_id, sigs: BoundedVec<(AccountId, AdminSignatureOf), MaxAdmins>)`
- **签名消息**:`TransferVoteIntent { from: NRC_ANQUAN_ADDRESS, to: beneficiary, ... }`
  的 `signing_hash(ss58_prefix, OP_SIGN_SAFETY_FUND = 0x16)`
- **校验**:
  1. SafetyFundAction 存在
  2. `sigs.len() >= 阈值(NRC 硬编码阈值)`
  3. 每个 admin 是 NRC 管理员(`UnauthorizedSignature`)
  4. 同批次不重复(`DuplicateSignature`)
  5. sig 长度 64(`MalformedSignature`)
  6. sr25519 验签通过(`InvalidSignature`)
- **操作**:
  1. 逐条 `cast_internal_vote(admin, proposal_id, true)`
  2. 达阈值后 `STATUS_PASSED` → 事务内 `try_execute_safety_fund`
  3. 发 `SafetyFundFinalized { signatures_accepted, final_status }` 事件
- **已删除**:原 `vote_safety_fund_transfer` + `SafetyFundVoteSubmitted` 事件,无兼容保留

### 3. 自动执行（try_execute_safety_fund）

投票通过后在 `with_transaction` 中原子执行：

1. 验证提案状态为 PASSED
2. 解码 NRC_ANQUAN_ADDRESS 为 AccountId
3. InstitutionAssetGuard::can_spend 再次检查支出权限
4. **计算手续费**：`fee = max(amount * 0.1%, 0.1 元)`
5. **余额检查**：`free >= amount + fee + ED`
6. **执行转账**：Currency::transfer（KeepAlive）
7. **扣取手续费**：Currency::withdraw 后通过 FeeRouter::on_unbalanced 按 80/10/10 分账
8. 设置提案状态为 EXECUTED

## 手续费分账（FeeRouter 80/10/10）

安全基金转账产生的手续费通过 FeeRouter 分配：

| 比例 | 接收方 | 说明 |
|------|--------|------|
| 80% | 全节点 | ONCHAIN_FEE_FULLNODE_PERCENT |
| 10% | 国储会手续费账户 | ONCHAIN_FEE_NRC_PERCENT |
| 10% | 安全基金账户 | ONCHAIN_FEE_SAFETY_FUND_PERCENT |

手续费率固定为链上费率 0.1%（ONCHAIN_FEE_RATE），单笔最低 0.1 元（ONCHAIN_MIN_FEE）。

## 执行失败处理

若 try_execute_safety_fund 失败：
- `with_transaction` 回滚所有状态变更
- 触发 SafetyFundExecutionFailed 事件
- 提案保留，可通过 `execute_safety_fund_transfer` 手动重试

## 源码位置

- `citizenchain/runtime/transaction/duoqian-transfer-pow/src/lib.rs`
  - `propose_safety_fund_transfer`(call_index=3)
  - `finalize_safety_fund_transfer`(call_index=4,Step 2 替换原 `vote_safety_fund_transfer`)
  - `try_execute_safety_fund`(内部方法)
  - `verify_and_cast_votes`(Step 2 · 三组 finalize_X 共用的验签 + 代投 helper)
- `citizenchain/runtime/primitives/china/china_cb.rs` - NRC_ANQUAN_ADDRESS 定义
- `citizenchain/runtime/primitives/src/core_const.rs` - `OP_SIGN_SAFETY_FUND = 0x16` 签名域常量
