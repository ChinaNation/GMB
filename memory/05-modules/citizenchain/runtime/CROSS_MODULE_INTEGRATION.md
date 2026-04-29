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
| `duoqian-manage-pow` | `frame_system`, `voting-engine`（通过 InternalVoteEngine）, `admins-change` |
| `duoqian-transfer-pow` | `frame_system`, `voting-engine`, `duoqian-manage-pow`, `admins-change`（测试/runtime 约束） |
| `offchain-transaction-pos` | `frame_system`, `voting-engine`（通过 InternalVoteEngine） |
| `sfid-code-auth` | `frame_system` |
| `citizen-lightnode-issuance` | `frame_system`, `pallet_balances`（通过 Currency） |

## 关键 Trait 提供矩阵

| Trait | 定义 Pallet | Runtime 实现体 | 消费 Pallet |
|-------|------------|---------------|------------|
| `InternalVoteEngine` | `voting-engine` | `voting_engine::Pallet<Runtime>` | `duoqian-manage-pow`, `duoqian-transfer-pow`(间接), `admins-change`, `resolution-destro`, `grandpakey-change`, `offchain-transaction-pos` |
| `JointVoteEngine` | `voting-engine` | `voting_engine::Pallet<Runtime>` | `resolution-issuance`, `runtime-upgrade` |
| `InternalAdminProvider` | `voting-engine` | `RuntimeInternalAdminProvider` | `voting-engine` (Config 注入) |
| `InternalAdminCountProvider` | `voting-engine` | `RuntimeInternalAdminCountProvider` | `voting-engine` (Config 注入) |
| `InternalThresholdProvider` | `voting-engine` | `RuntimeInternalThresholdProvider` | `voting-engine` (Config 注入) |
| `InstitutionAssetGuard` | `institution-asset-guard` | `RuntimeInstitutionAssetGuard` | `duoqian-manage-pow`, `duoqian-transfer-pow`(间接), `offchain-transaction-pos` |
| `NrcAccountProvider` | `onchain-transaction-pow` | `RuntimeNrcAccountProvider` | `onchain-transaction-pow` (PowOnchainFeeRouter) |
| `FeeRouter` (OnUnbalanced) | `frame_support` trait | `TransferFeeRouter` | `duoqian-manage-pow`, `duoqian-transfer-pow` |
| `FeePayerExtractor` (CallFeePayer) | `onchain-transaction-pow` | `RuntimeFeePayerExtractor` | `pallet-transaction-payment` (OnChargeTransaction) |
| `AmountExtractor` (CallAmount) | `onchain-transaction-pow` | `PowTxAmountExtractor` | `pallet-transaction-payment` (OnChargeTransaction) |
| `ProtectedSourceChecker` | `duoqian-manage-pow` / `offchain-transaction-pos` | `RuntimeProtectedSourceChecker` | `duoqian-manage-pow`, `offchain-transaction-pos` |
| `SfidEligibility` | `voting-engine` | `RuntimeSfidEligibility` (委托 sfid-code-auth) | `voting-engine` |
| `PopulationSnapshotVerifier` | `voting-engine` | `RuntimePopulationSnapshotVerifier` | `voting-engine` |
| `JointVoteResultCallback` | `voting-engine` | `RuntimeJointVoteResultCallback` | `voting-engine` (投票通过后回调) |
| `SfidInstitutionVerifier` | `duoqian-manage-pow` | `RuntimeSfidInstitutionVerifier` | `duoqian-manage-pow` |
| `SfidVerifier` / `SfidVoteVerifier` | `sfid-code-auth` | `RuntimeSfidVerifier` / `RuntimeSfidVoteVerifier` | `sfid-code-auth` |

## Runtime 级别适配器说明

| 适配器 | 作用 |
|--------|------|
| `RuntimeInternalAdminProvider` | 所有内部投票主体统一读 `admins_change::Institutions` |
| `RuntimeInternalThresholdProvider` | 所有内部投票主体统一读 `admins_change::Institutions.threshold` |
| `RuntimeInternalAdminCountProvider` | 所有内部投票主体统一读 `admins_change::Institutions.admins.len()` |
| `RuntimeJointVoteResultCallback` | 按模块路由：先查 `resolution-issuance`，再查 `runtime-upgrade` |
| `TransferFeeRouter` | 旧 NegativeImbalance -> Credit 转换 -> `PowOnchainFeeRouter` 80/10/10 分账 |
| `RuntimeInstitutionAssetGuard` | stake 禁止一切; reserved main 仅允许转账/销户; fee_account 仅允许 sweep; 安全基金仅允许安全基金转账; CB 费用账户仅允许 sweep |

## 交易费用流

```
用户交易 -> pallet-transaction-payment
  -> PowOnchainChargeAdapter
    -> PowTxAmountExtractor (按 call 类型提取金额)
    -> RuntimeFeePayerExtractor (offchain 批次从省储行费用地址扣; 其余由调用者扣)
    -> PowOnchainFeeRouter (80% 矿工 / 10% NRC / 10% 安全基金)
```
