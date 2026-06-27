# 跨模块集成矩阵

## Pallet Config 依赖关系

| Pallet | 依赖的 Config trait |
|--------|-------------------|
| `votingengine` | `frame_system` |
| `genesis-admins` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives` |
| `public-admins` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives` |
| `private-admins` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives` |
| `personal-admins` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `pallet_balances`, `admin-primitives` |
| `resolution-destro` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `pallet_balances`（通过 Currency） |
| `grandpakey-change` | `frame_system`, `votingengine`（通过 InternalVoteEngine） |
| `resolution-issuance` | `frame_system`, `votingengine`（通过 JointVoteEngine）, `pallet_balances`（通过 Currency） |
| `runtime-upgrade` | `frame_system`, `votingengine`（通过 JointVoteEngine） |
| `organization-manage` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives`（通过 lifecycle/query trait） |
| `multisig-transfer` | `frame_system`, `votingengine`, `organization-manage`, `personal-admins`（通过 PersonalQuery） |
| `offchain-transaction` | `frame_system`, `votingengine`（通过 InternalVoteEngine） |
| `cid-system` | `frame_system` |
| `citizen-issuance` | `frame_system`, `pallet_balances`（通过 Currency） |

## 关键 Trait 提供矩阵

| Trait | 定义 Pallet | Runtime 实现体 | 消费 Pallet |
|-------|------------|---------------|------------|
| `InternalVoteEngine` | `votingengine` | `votingengine::Pallet<Runtime>` | `organization-manage`, `multisig-transfer`(间接), `genesis-admins`, `public-admins`, `private-admins`, `personal-admins`, `resolution-destro`, `grandpakey-change`, `offchain-transaction` |
| `JointVoteEngine` | `votingengine` | `votingengine::Pallet<Runtime>` | `resolution-issuance`, `runtime-upgrade` |
| `InternalAdminProvider` | `votingengine` | `RuntimeInternalAdminProvider` | `votingengine` (Config 注入) |
| `InternalAdminsLenProvider` | `votingengine` | `RuntimeInternalAdminsLenProvider` | `votingengine` (Config 注入) |
| `InstitutionAsset` | `institution-asset` | `RuntimeInstitutionAsset` | `organization-manage`, `multisig-transfer`(间接), `offchain-transaction` |
| `NrcAccountProvider` | `onchain-transaction` | `RuntimeNrcAccountProvider` | `onchain-transaction` (OnchainFeeRouter) |
| `SafetyFundAccountProvider` | `onchain-transaction` | `RuntimeSafetyFundAccountProvider` | `onchain-transaction` (OnchainFeeRouter) |
| `FeeRouter` (OnUnbalanced) | `frame_support` trait | `TransferFeeRouter` | `organization-manage`, `multisig-transfer` |
| `FeePayerExtractor` (CallFeePayer) | `onchain-transaction` | `RuntimeFeePayerExtractor` | `pallet-transaction-payment` (OnChargeTransaction) |
| `FeeKindClassifier` (CallFeeKind) | `onchain-transaction` | `RuntimeFeeKindClassifier` | `pallet-transaction-payment` (OnChargeTransaction) |
| `ProtectedSourceChecker` | `organization-manage` / `offchain-transaction` | `RuntimeProtectedSourceChecker` | `organization-manage`, `offchain-transaction` |
| `CidEligibility` | `votingengine` | `RuntimeCidEligibility` (委托 cid-system) | `votingengine` |
| `PopulationSnapshotVerifier` | `votingengine` | `RuntimePopulationSnapshotVerifier` | `votingengine` |
| `JointVoteResultCallback` | `votingengine` | `RuntimeJointVoteResultCallback` | `votingengine` (投票通过后回调) |
| `CidInstitutionVerifier` | `organization-manage` | `RuntimeCidInstitutionVerifier` | `organization-manage` |
| `CidVerifier` / `CidVoteVerifier` | `cid-system` | `RuntimeCidVerifier` / `RuntimeCidVoteVerifier` | `cid-system` |
| `AdminAccountLifecycle` | `admin-primitives` | `GenesisAdmins` / `PublicAdmins` / `PrivateAdmins` / `PersonalAdmins` | `organization-manage`, `personal-admins` |
| `AdminAccountQuery` | `admin-primitives` | `RuntimeAdminAccountQuery` | `organization-manage`, `multisig-transfer`, `votingengine`, runtime verifier |

## Runtime 级别适配器说明

| 适配器 | 作用 |
|--------|------|
| `RuntimeAdminAccountQuery` | 按机构码把管理员查询路由到 `genesis-admins`、`public-admins`、`private-admins`、`personal-admins` |
| `RuntimeInternalAdminProvider` | 所有内部投票主体统一委托 `RuntimeAdminAccountQuery` 读取管理员 |
| `RuntimeInternalAdminsLenProvider` | 所有内部投票主体统一委托 `RuntimeAdminAccountQuery` 读取管理员人数 |
| `RuntimeJointVoteResultCallback` | 按模块路由：先查 `resolution-issuance`，再查 `runtime-upgrade` |
| `TransferFeeRouter` | 旧 NegativeImbalance -> Credit 转换 -> `OnchainFeeRouter` 80/10/10 分账 |
| `RuntimeSafetyFundAccountProvider` | 将安全基金制度常量 `SAFETY_FUND_ACCOUNT` 转为 runtime 账户，避免手续费分账热路径重复 decode |
| `RuntimeInstitutionAsset` | stake 禁止一切; reserved main 仅允许转账/销户; fee_account 仅允许 sweep; 安全基金仅允许安全基金转账; CB 费用账户仅允许 sweep |

## 交易费用流

```
用户交易 -> pallet-transaction-payment
  -> OnchainChargeAdapter
    -> RuntimeFeeKindClassifier (按 call 类型归入五类费用模型)
    -> RuntimeFeePayerExtractor (offchain 批次从省储行费用地址扣; 其余由调用者扣)
    -> RuntimeNrcAccountProvider / RuntimeSafetyFundAccountProvider (提供 NRC 与安全基金收款账户)
    -> OnchainFeeRouter (80% 矿工 / 10% NRC / 10% 安全基金)
```
