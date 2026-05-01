# 跨模块集成矩阵

## Pallet Config 依赖关系

| Pallet | 依赖的 Config trait |
|--------|-------------------|
| `voting-engine` | `frame_system` |
| `admins-change` | `frame_system`, `voting-engine`（通过 InternalVoteEngine） |
| `resolution-destro` | `frame_system`, `voting-engine`（通过 InternalVoteEngine）, `pallet_balances`（通过 Currency） |
| `grandpakey-change` | `frame_system`, `voting-engine`（通过 InternalVoteEngine） |
| `resolution-issuance` | `frame_system`, `voting-engine`（通过 JointVoteEngine）, `pallet_balances`（通过 Currency） |
| `runtime-upgrade` | `frame_system`, `voting-engine`（通过 JointVoteEngine） |
| `duoqian-manage` | `frame_system`, `voting-engine`（通过 InternalVoteEngine）, `admins-change` |
| `duoqian-transfer` | `frame_system`, `voting-engine`, `duoqian-manage`, `admins-change`（测试/runtime 约束） |
| `offchain-transaction` | `frame_system`, `voting-engine`（通过 InternalVoteEngine） |
| `sfid-system` | `frame_system` |
| `citizen-issuance` | `frame_system`, `pallet_balances`（通过 Currency） |

## 关键 Trait 提供矩阵

| Trait | 定义 Pallet | Runtime 实现体 | 消费 Pallet |
|-------|------------|---------------|------------|
| `InternalVoteEngine` | `voting-engine` | `voting_engine::Pallet<Runtime>` | `duoqian-manage`, `duoqian-transfer`(间接), `admins-change`, `resolution-destro`, `grandpakey-change`, `offchain-transaction` |
| `JointVoteEngine` | `voting-engine` | `voting_engine::Pallet<Runtime>` | `resolution-issuance`, `runtime-upgrade` |
| `InternalAdminProvider` | `voting-engine` | `RuntimeInternalAdminProvider` | `voting-engine` (Config 注入) |
| `InternalAdminCountProvider` | `voting-engine` | `RuntimeInternalAdminCountProvider` | `voting-engine` (Config 注入) |
| `InternalThresholdProvider` | `voting-engine` | `RuntimeInternalThresholdProvider` | `voting-engine` (Config 注入) |
| `InstitutionAsset` | `institution-asset` | `RuntimeInstitutionAsset` | `duoqian-manage`, `duoqian-transfer`(间接), `offchain-transaction` |
| `NrcAccountProvider` | `onchain-transaction` | `RuntimeNrcAccountProvider` | `onchain-transaction` (OnchainFeeRouter) |
| `SafetyFundAccountProvider` | `onchain-transaction` | `RuntimeSafetyFundAccountProvider` | `onchain-transaction` (OnchainFeeRouter) |
| `FeeRouter` (OnUnbalanced) | `frame_support` trait | `TransferFeeRouter` | `duoqian-manage`, `duoqian-transfer` |
| `FeePayerExtractor` (CallFeePayer) | `onchain-transaction` | `RuntimeFeePayerExtractor` | `pallet-transaction-payment` (OnChargeTransaction) |
| `AmountExtractor` (CallAmount) | `onchain-transaction` | `OnchainTxAmountExtractor` | `pallet-transaction-payment` (OnChargeTransaction) |
| `ProtectedSourceChecker` | `duoqian-manage` / `offchain-transaction` | `RuntimeProtectedSourceChecker` | `duoqian-manage`, `offchain-transaction` |
| `SfidEligibility` | `voting-engine` | `RuntimeSfidEligibility` (委托 sfid-system) | `voting-engine` |
| `PopulationSnapshotVerifier` | `voting-engine` | `RuntimePopulationSnapshotVerifier` | `voting-engine` |
| `JointVoteResultCallback` | `voting-engine` | `RuntimeJointVoteResultCallback` | `voting-engine` (投票通过后回调) |
| `SfidInstitutionVerifier` | `duoqian-manage` | `RuntimeSfidInstitutionVerifier` | `duoqian-manage` |
| `SfidVerifier` / `SfidVoteVerifier` | `sfid-system` | `RuntimeSfidVerifier` / `RuntimeSfidVoteVerifier` | `sfid-system` |

## Runtime 级别适配器说明

| 适配器 | 作用 |
|--------|------|
| `RuntimeInternalAdminProvider` | 所有内部投票主体统一读 `admins_change::Institutions` |
| `RuntimeInternalThresholdProvider` | 所有内部投票主体统一读 `admins_change::Institutions.threshold` |
| `RuntimeInternalAdminCountProvider` | 所有内部投票主体统一读 `admins_change::Institutions.admins.len()` |
| `RuntimeJointVoteResultCallback` | 按模块路由：先查 `resolution-issuance`，再查 `runtime-upgrade` |
| `TransferFeeRouter` | 旧 NegativeImbalance -> Credit 转换 -> `OnchainFeeRouter` 80/10/10 分账 |
| `RuntimeSafetyFundAccountProvider` | 将安全基金制度常量 `NRC_ANQUAN_ADDRESS` 转为 runtime 账户，避免手续费分账热路径重复 decode |
| `RuntimeInstitutionAsset` | stake 禁止一切; reserved main 仅允许转账/销户; fee_account 仅允许 sweep; 安全基金仅允许安全基金转账; CB 费用账户仅允许 sweep |

## 交易费用流

```
用户交易 -> pallet-transaction-payment
  -> OnchainChargeAdapter
    -> OnchainTxAmountExtractor (按 call 类型提取金额)
    -> RuntimeFeePayerExtractor (offchain 批次从省储行费用地址扣; 其余由调用者扣)
    -> RuntimeNrcAccountProvider / RuntimeSafetyFundAccountProvider (提供 NRC 与安全基金收款账户)
    -> OnchainFeeRouter (80% 矿工 / 10% NRC / 10% 安全基金)
```
