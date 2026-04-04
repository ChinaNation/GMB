# 跨模块集成矩阵

## Pallet Config 依赖关系

| Pallet | 依赖的 Config trait |
|--------|-------------------|
| `voting-engine-system` | `frame_system` |
| `admins-origin-gov` | `frame_system`, `voting-engine-system`（通过 InternalVoteEngine） |
| `resolution-destro-gov` | `frame_system`, `voting-engine-system`（通过 InternalVoteEngine）, `pallet_balances`（通过 Currency） |
| `grandpa-key-gov` | `frame_system`, `voting-engine-system`（通过 InternalVoteEngine） |
| `resolution-issuance-gov` | `frame_system`, `voting-engine-system`（通过 JointVoteEngine） |
| `runtime-root-upgrade` | `frame_system`, `voting-engine-system`（通过 JointVoteEngine） |
| `duoqian-manage-pow` | `frame_system`, `voting-engine-system`（通过 InternalVoteEngine） |
| `duoqian-transfer-pow` | `frame_system`, `voting-engine-system`, `duoqian-manage-pow` |
| `offchain-transaction-pos` | `frame_system`, `voting-engine-system`（通过 InternalVoteEngine） |
| `sfid-code-auth` | `frame_system` |
| `citizen-lightnode-issuance` | `frame_system`, `pallet_balances`（通过 Currency） |
| `resolution-issuance-iss` | `frame_system`, `pallet_balances`（通过 Currency） |

## 关键 Trait 提供矩阵

| Trait | 定义 Pallet | Runtime 实现体 | 消费 Pallet |
|-------|------------|---------------|------------|
| `InternalVoteEngine` | `voting-engine-system` | `voting_engine_system::Pallet<Runtime>` | `duoqian-manage-pow`, `duoqian-transfer-pow`(间接), `admins-origin-gov`, `resolution-destro-gov`, `grandpa-key-gov`, `offchain-transaction-pos` |
| `JointVoteEngine` | `voting-engine-system` | `voting_engine_system::Pallet<Runtime>` | `resolution-issuance-gov`, `runtime-root-upgrade` |
| `InternalAdminProvider` | `voting-engine-system` | `RuntimeInternalAdminProvider` | `voting-engine-system` (Config 注入) |
| `InternalAdminCountProvider` | `voting-engine-system` | `RuntimeInternalAdminCountProvider` | `voting-engine-system` (Config 注入) |
| `InternalThresholdProvider` | `voting-engine-system` | `RuntimeInternalThresholdProvider` | `voting-engine-system` (Config 注入) |
| `InstitutionAssetGuard` | `institution-asset-guard` | `RuntimeInstitutionAssetGuard` | `duoqian-manage-pow`, `duoqian-transfer-pow`(间接), `offchain-transaction-pos` |
| `NrcAccountProvider` | `onchain-transaction-pow` | `RuntimeNrcAccountProvider` | `onchain-transaction-pow` (PowOnchainFeeRouter) |
| `FeeRouter` (OnUnbalanced) | `frame_support` trait | `TransferFeeRouter` | `duoqian-manage-pow`, `duoqian-transfer-pow` |
| `FeePayerExtractor` (CallFeePayer) | `onchain-transaction-pow` | `RuntimeFeePayerExtractor` | `pallet-transaction-payment` (OnChargeTransaction) |
| `AmountExtractor` (CallAmount) | `onchain-transaction-pow` | `PowTxAmountExtractor` | `pallet-transaction-payment` (OnChargeTransaction) |
| `ProtectedSourceChecker` | `duoqian-manage-pow` / `offchain-transaction-pos` | `RuntimeProtectedSourceChecker` | `duoqian-manage-pow`, `offchain-transaction-pos` |
| `SfidEligibility` | `voting-engine-system` | `RuntimeSfidEligibility` (委托 sfid-code-auth) | `voting-engine-system` |
| `PopulationSnapshotVerifier` | `voting-engine-system` | `RuntimePopulationSnapshotVerifier` | `voting-engine-system` |
| `JointVoteResultCallback` | `voting-engine-system` | `RuntimeJointVoteResultCallback` | `voting-engine-system` (投票通过后回调) |
| `SfidInstitutionVerifier` | `duoqian-manage-pow` | `RuntimeSfidInstitutionVerifier` | `duoqian-manage-pow` |
| `SfidVerifier` / `SfidVoteVerifier` | `sfid-code-auth` | `RuntimeSfidVerifier` / `RuntimeSfidVoteVerifier` | `sfid-code-auth` |

## Runtime 级别适配器说明

| 适配器 | 作用 |
|--------|------|
| `RuntimeInternalAdminProvider` | ORG_DUOQIAN → 读 `DuoqianAccounts`; 治理机构 → 读 `admins_origin_gov::CurrentAdmins` |
| `RuntimeInternalThresholdProvider` | ORG_DUOQIAN → 从链上 `DuoqianAccounts.threshold` 动态读取; 治理机构 → 硬编码阈值 |
| `RuntimeInternalAdminCountProvider` | ORG_DUOQIAN → `DuoqianAccounts.duoqian_admins.len()`; 治理机构 → `CurrentAdmins.len()` |
| `RuntimeJointVoteResultCallback` | 按模块路由：先查 `resolution-issuance-gov`，再查 `runtime-root-upgrade` |
| `TransferFeeRouter` | 旧 NegativeImbalance -> Credit 转换 -> `PowOnchainFeeRouter` 80/10/10 分账 |
| `RuntimeInstitutionAssetGuard` | keyless 禁止一切; reserved_duoqian 仅允许转账/销户; fee_account 仅允许 sweep; 安全基金仅允许安全基金转账; CB 费用账户仅允许 sweep |

## 交易费用流

```
用户交易 -> pallet-transaction-payment
  -> PowOnchainChargeAdapter
    -> PowTxAmountExtractor (按 call 类型提取金额)
    -> RuntimeFeePayerExtractor (offchain 批次从省储行费用地址扣; 其余由调用者扣)
    -> PowOnchainFeeRouter (80% 矿工 / 10% NRC / 10% 安全基金)
```
