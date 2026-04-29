# Institution Asset Technical Notes

模块：`institution-asset`  
范围：机构账户内部资金动作白名单公共模块

## 0. 核心职责

- 统一定义“哪些内部执行动作可以从哪些制度账户扣钱”。
- 只管内部资金动作白名单，不管提案、投票、管理员变更等纯治理行为。
- 供 `duoqian-manage`、`duoqian-transfer`、`offchain-transaction` 复用。

## 1. 设计边界

- 本模块不是 pallet。
- 本模块不包含 storage。
- 本模块不提供 extrinsic。
- 本模块只提供公共枚举和 trait，真正的放行/拒绝规则由 runtime 实现。

## 2. 公共接口

### 2.1 InstitutionAssetAction

当前定义 4 类动作：

1. `DuoqianTransferExecute`
2. `DuoqianCloseExecute`
3. `OffchainBatchDebit`
4. `OffchainFeeSweepExecute`

### 2.2 InstitutionAsset<AccountId>

```rust
pub trait InstitutionAsset<AccountId> {
    fn can_spend(source: &AccountId, action: InstitutionAssetAction) -> bool;
}
```

语义：

- `source`：实际扣款源账户
- `action`：内部业务动作类型
- 返回 `true`：允许该模块从该账户扣钱
- 返回 `false`：不允许该模块内部动用该账户资金

## 3. 当前 runtime 规则

当前 `citizenchain/runtime/src/configs/mod.rs` 中的 `RuntimeInstitutionAsset` 规则是：

1. `stake_address`
   - 一律拒绝
2. 制度保留 `main_address`
   - 只允许 `DuoqianTransferExecute`
   - 只允许 `DuoqianCloseExecute`
3. 制度 `fee_account`
   - 只允许 `OffchainFeeSweepExecute`
4. 其他普通账户
   - 默认允许

## 4. 接入点

### 4.1 duoqian-manage

- `propose_close`
- `execute_close`

### 4.2 duoqian-transfer

- `propose_transfer`
- `try_execute_transfer`

### 4.3 offchain-transaction

- 批次付款源 `item.payer`
- 手续费归集源 `fee_account`

## 5. 设计原因

- `ProtectedSourceChecker` 只能表达“完全禁止的源地址”，适合 `stake_address`。
- `main_address` 不是完全禁止，而是“只有特定治理执行模块能动钱”。
- 所以需要单独的资金动作白名单层，不能继续复用单一布尔语义的 `ProtectedSourceChecker`。

## 6. 安全注意事项

- 默认 `()` 实现为 **fail-open（全放行）**，仅适用于测试 mock。
- 生产 runtime 必须配置为 `RuntimeInstitutionAsset`，否则这一层资金白名单将完全失效。
- 建议 runtime 层维护集成测试，锁定 stake / 保留 main / fee_account / 普通账户的允许矩阵，防止规则意外退化。
