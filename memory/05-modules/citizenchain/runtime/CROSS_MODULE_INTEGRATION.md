# 跨模块集成矩阵

## Pallet Config 依赖关系

| Pallet | 依赖的 Config trait |
|--------|-------------------|
| `votingengine` | `frame_system` |
| `public-admins` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives` |
| `private-admins` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives` |
| `personal-admins` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `pallet_balances`, `admin-primitives` |
| `genesis-pallet` | `frame_system`, `public-manage`, `public-admins`（仅创世写入内置机构和初始管理员） |
| `resolution-destro` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `pallet_balances`（通过 Currency） |
| `grandpakey-change` | `frame_system`, `votingengine`（通过 InternalVoteEngine） |
| `resolution-issuance` | `frame_system`, `votingengine`（通过 JointVoteEngine）, `pallet_balances`（通过 Currency） |
| `runtime-upgrade` | `frame_system`, `votingengine`（通过 JointVoteEngine） |
| `election-campaign` | `frame_system`（当前仅 runtime metadata 骨架；后续通过 election-vote 创建选举投票） |
| `entity-primitives` | 无 storage；定义实体生命周期共用 trait |
| `public-manage` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives`（通过 lifecycle/query trait）, `entity-primitives` |
| `private-manage` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives`（通过 lifecycle/query trait）, `entity-primitives` |
| `personal-manage` | `frame_system`, `votingengine`（通过 InternalVoteEngine）, `admin-primitives`（通过 lifecycle/query trait） |
| `multisig-transfer` | `frame_system`, `votingengine`, `entity-primitives`, `personal-manage`（通过 PersonalQuery） |
| `offchain-transaction` | `frame_system`, `votingengine`（通过 InternalVoteEngine） |
| `citizen-identity` | `frame_system`, `citizen-issuance`（通过回调）, `pallet_balances`（通过签名账户） |
| `citizen-issuance` | `frame_system`, `pallet_balances`（通过 Currency） |

## 关键 Trait 提供矩阵

| Trait | 定义 Pallet | Runtime 实现体 | 消费 Pallet |
|-------|------------|---------------|------------|
| `InternalVoteEngine` | `votingengine` | `votingengine::Pallet<Runtime>` | `public-manage`, `private-manage`, `personal-manage`, `multisig-transfer`, `public-admins`, `private-admins`, `personal-admins`, `resolution-destro`, `grandpakey-change`, `offchain-transaction` |
| `JointVoteEngine` | `votingengine` | `votingengine::Pallet<Runtime>` | `resolution-issuance`, `runtime-upgrade` |
| `InternalAdminProvider` | `votingengine` | `RuntimeInternalAdminProvider` | `votingengine` (Config 注入) |
| `InternalAdminsLenProvider` | `votingengine` | `RuntimeInternalAdminsLenProvider` | `votingengine` (Config 注入) |
| `InstitutionAsset` | `institution-asset` | `RuntimeInstitutionAsset` | `public-manage`, `private-manage`, `personal-manage`, `multisig-transfer`, `offchain-transaction` |
| `NrcAccountProvider` | `onchain-transaction` | `RuntimeNrcAccountProvider` | `onchain-transaction` (OnchainFeeRouter) |
| `SafetyFundAccountProvider` | `onchain-transaction` | `RuntimeSafetyFundAccountProvider` | `onchain-transaction` (OnchainFeeRouter) |
| `FeeRouter` (OnUnbalanced) | `frame_support` trait | `TransferFeeRouter` | `public-manage`, `private-manage`, `personal-manage`, `multisig-transfer` |
| `FeeRoute` | `primitives::fee_policy` | `RuntimeFeeRouter` 生成唯一类型 | `onchain::OnchainChargeAdapter`、链下收费执行器 |
| `CallFeeRoute` | `onchain` | `RuntimeFeeRouter` | `pallet-transaction-payment` (`OnChargeTransaction`) |
| `ProtectedSourceChecker` | `entity-primitives` / `offchain-transaction` | `RuntimeProtectedSourceChecker` | `public-manage`, `private-manage`, `personal-manage`, `multisig-transfer`, `offchain-transaction` |
| `CitizenIdentityReader` | `votingengine` | `RuntimeCitizenIdentityReader`（委托 `citizen-identity`） | `votingengine` |
| `JointVoteResultCallback` | `votingengine` | `RuntimeJointVoteResultCallback` | `votingengine` (投票通过后回调) |
| `CidInstitutionVerifier` | `entity-primitives` | `RuntimeCidInstitutionVerifier` | `public-manage`, `private-manage` |
| `CitizenIdentityAuthority` | `citizen-identity` | `RuntimeCitizenIdentityAuthority` | `citizen-identity` |
| `OnVotingIdentityRegistered` | `citizen-identity` | `CitizenIssuance` | `citizen-issuance` |
| `AdminAccountLifecycle` | `admin-primitives` | `PublicAdmins` / `PrivateAdmins` / `PersonalAdmins` | `public-manage`, `private-manage`, `personal-manage`, `personal-admins` |
| `AdminAccountQuery` | `admin-primitives` | `RuntimeAdminAccountQuery` | `public-manage`, `private-manage`, `multisig-transfer`, `votingengine`, runtime verifier |

## Runtime 级别适配器说明

| 适配器 | 作用 |
|--------|------|
| `RuntimeAdminAccountQuery` | 按机构码把管理员查询路由到 `public-admins`、`private-admins`、`personal-admins`；固定治理机构也读 `public-admins` |
| `RuntimeInstitutionQuery` | 按公权/私权机构生命周期模块聚合机构账户状态和管理员快照，供 `multisig-transfer` 使用 |
| `RuntimeInternalAdminProvider` | 所有内部投票主体统一委托 `RuntimeAdminAccountQuery` 读取管理员 |
| `RuntimeInternalAdminsLenProvider` | 所有内部投票主体统一委托 `RuntimeAdminAccountQuery` 读取管理员人数 |
| `RuntimeCitizenIdentityReader` | 给投票引擎读取投票资格、参选资格和链上人口分母 |
| `RuntimeCitizenIdentityAuthority` | 给公民身份模块校验注册局权限和公民钱包签名 |
| `RuntimeJointVoteResultCallback` | 按模块路由：先查 `resolution-issuance`，再查 `runtime-upgrade` |
| `TransferFeeRouter` | 旧 NegativeImbalance -> Credit 转换 -> `OnchainFeeRouter` 80/10/10 分账 |
| `RuntimeSafetyFundAccountProvider` | 将安全基金制度常量 `SAFETY_FUND_ACCOUNT` 转为 runtime 账户，避免手续费分账热路径重复 decode |
| `RuntimeInstitutionAsset` | stake 禁止一切; reserved main 仅允许转账/销户; fee_account 仅允许 sweep; 安全基金仅允许安全基金转账; CB 费用账户仅允许 sweep |

## 选举业务与投票边界

`election-campaign` 是公权选举业务壳，负责后续承载“什么机构能组织什么职位选举、普选/互选如何选择、候选人和选民快照从哪里生成、结果写回哪个业务真源”等规则。

`election-vote` 是选举投票模块，负责选举投票提案、投票去重、计票、超时结算、结果快照和清理。`ElectionVote::create_popular_election` 与 `ElectionVote::create_mutual_election` 外部入口已删除；runtime 只保留 `cast_popular_vote` 和 `cast_mutual_vote` 作为投票动作。结果快照必须由 `election-campaign` 复核业务规则后才能形成 entity 任职结果。

## 交易费用流

```
用户交易 -> pallet-transaction-payment
  -> OnchainChargeAdapter
    -> RuntimeFeeRouter (一次返回 FeeRoute，费用类别和确切付款账户不可分离)
       -> 机构操作：actor CID + admins 授权 + 唯一费用账户，任一失败即 Reject
       -> 实际投票：Vote，固定由投票签名者支付
       -> 普通用户/Fullnode：Onchain，由签名者支付
       -> 未分类/未开放：Reject，不存在默认分支或付款方回落
    -> RuntimeNrcAccountProvider / RuntimeSafetyFundAccountProvider (提供 NRC 与安全基金收款账户)
    -> OnchainFeeRouter (80% 矿工 / 10% NRC / 10% 安全基金)
```

`primitives::fee_policy::TRANSACTION_TIP` 固定为零；`WeightToFee` 和 `LengthToFee` 也固定为零，因此收费只可能来自五类路由对应执行器。
