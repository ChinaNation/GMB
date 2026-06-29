# Multisig Transfer Pow 技术文档（机构多签名地址转账模块）

> 机构分类唯一真源 = CID 机构码（institution_code），见 [[ADR-025]]。

## 2026-04-30 · 统一投票引擎状态机改造

本模块所有 3 组业务（transfer / safety_fund / sweep）已统一接入 `votingengine` 生命周期：

- 提案创建使用 `create_internal_proposal_with_data`，在同一事务中绑定 `ProposalOwner`、`ProposalData` 和 `ProposalMeta`。
- 管理员投票统一走 `InternalVote::cast(proposal_id, approve)`，本模块不再提供独立 vote/finalize call。
- 投票通过后由 `InternalVoteExecutor` 自动执行。
- 自动执行成功返回 `ProposalExecutionOutcome::Executed`，投票引擎转 `STATUS_EXECUTED`。
- 自动执行失败返回 `ProposalExecutionOutcome::RetryableFailed`，提案保持 `STATUS_PASSED` 并进入统一 retry state。
- **统一入口（2026-05-02 整改）**：execute_xxx wrapper extrinsic 已物理删除。前端直接调用 `VotingEngine::retry_passed_proposal(proposal_id)`，由投票引擎统一校验快照管理员权限、最多 3 次手动失败和 retry deadline。

### 入口对照

| call_index | extrinsic | 说明 |
|---|---|---|
| 0 | `propose_transfer` | 发起普通机构转账提案 |
| 1 | `propose_safety_fund_transfer` | 发起安全基金转账提案 |
| 2 | `propose_sweep_to_main` | 发起费用账户划转主账户提案 |

投票走统一入口 `InternalVote::cast`(pallet 22.0),手动重试/取消走
`VotingEngine::retry_passed_proposal`(9.4)/`cancel_passed_proposal`(9.5)。

## 2026-05-08 · 第5步账户级主体接入

- `propose_transfer` 的 `institution: AccountId` 不再把 `0x02 注册机构归属关系` 当作可支出主体；`0x02` 只保留给机构归属与检索。
- 治理机构仍使用 `0x01 BuiltinInstitution`，由静态预置表解析到治理机构 `main_account`。
- 个人多签使用 `PersonalAccount AccountId + AccountId32 + 15B 零填充`，账户状态由 `personal-admins::PersonalMultisigQuery` 校验。
- 注册机构具体账户使用 `InstitutionAccount AccountId + AccountId32 + 15B 零填充`，账户状态由 `organization-manage::InstitutionMultisigQuery` 校验。
- 两类注册账户的管理员、阈值和人数都以 `RuntimeAdminAccountQuery` 为读入口；生产 runtime 按机构码路由到 `public-admins`、`private-admins` 或 `personal-admins`，内部投票仍是一人一票一笔链上交易。

## 0. 功能需求

### 0.1 核心职责

`multisig-transfer` 负责多签资金账户通过内部投票引擎发起转账：

- 机构管理员发起转账提案，指定收款地址、金额和备注。
- 机构管理员通过内部投票引擎逐票投票。
- 投票通过后执行转账：从提案绑定的资金账户向收款地址划转资金。
- 手续费在投票通过后由 pallet 内部从同一个资金账户扣取，通过 `onchain-transaction::calculate_onchain_fee()` 计算。
- 管理员个人账户不承担任何费用。
- 覆盖三类来源：
  - 创世预置的治理机构 `main_account`（NRC / PRC / PRB）
  - `personal-admins` 注册并激活的个人多签账户（`PersonalAccount AccountId`）
  - `organization-manage` 注册并激活的机构具体账户（`InstitutionAccount AccountId`）

### 0.2 功能边界

- 本模块处理三类多签账户转账：
  - 创世预置的治理机构（NRC / PRC / PRB）
  - `personal-admins` 注册并处于 Active 状态的个人多签账户（个人多签码 `PMUL`，`is_personal_code`）
  - `organization-manage` 注册并处于 Active 状态的机构账户（机构账户码 `is_institution_code`）
- 当前也尚未接入新补充的内置机构 `ZF / LF / JC / JY / SF`。
- 本模块不负责投票引擎实现，投票逻辑委托给 `votingengine` 的 `InternalVoteEngine`。

补充说明：
- 只要某类内置机构被本模块的 `institution_code()` / 主账户解析逻辑正式识别，
- 且对应管理员已接入 runtime 的 `RuntimeInternalAdminProvider`，固定阈值或动态阈值已由投票引擎自身提供，
- 这类机构就可以直接复用本模块和内部投票引擎发起转账提案，不需要新增转账 pallet。

### 0.3 与 `organization-manage` 的关系

| 模块 | 职责 | 地址类型 | 审批方式 |
| --- | --- | --- | --- |
| `organization-manage` | 多签名地址的注册、创建、关闭 | 注册的非治理机构账户 | `cid` 主签名登记 + 机构账户码（`is_institution_code`）内部投票 |
| `multisig-transfer` | 多签名地址转账 | 预置治理机构 + 注册型 Active 多签机构 | 链上内部投票引擎（逐票投票） |

### 0.4 与 `resolution-destro` 的关系

两者结构高度一致，区别在于资金操作：

| 对比 | `resolution-destro` | `multisig-transfer` |
| --- | --- | --- |
| 资金操作 | `Currency::slash()` 销毁 | `Currency::transfer()` 转账 |
| 目标 | 销毁机构持有的代币 | 转账到指定收款地址 |
| 额外字段 | 无 | `beneficiary`、`remark` |

## 1. 地址说明

### 1.1 关键区分

| 地址 | 类型 | 说明 |
| --- | --- | --- |
| `stake_account` | 质押地址 | **不允许支出**，仅用于质押 |
| `main_account`（治理机构）/ `account`（注册多签） | 多签资金账户 | 转账和手续费均从此扣取 |

### 1.2 资金账户来源

资金账户有三种来源：

- 治理机构：`main_account` 预置于 `runtime/primitives/cid/china/china_cb.rs`（NRC + PRC）和 `runtime/primitives/cid/china/china_ch.rs`（PRB）中，通过主账户解析逻辑查找。
- 个人多签账户：`AccountId32` 使用 `AdminAccountKind::PersonalMultisig` + 账户 `AccountId` 前 32 字节 + 15 字节零填充；账户状态从 `PersonalAdmins::PersonalAccounts` 校验 Active。
- 注册型机构账户：`AccountId32` 使用 `AdminAccountKind::PublicInstitution` 或 `AdminAccountKind::PrivateInstitution` + 账户 `AccountId` 前 32 字节 + 15 字节零填充；非法人按所属法人归属选择 public/private 管理员模块，账户状态从 `OrganizationManage::InstitutionAccounts` 校验 Active。

### 1.3 institution-asset 边界

- 本模块在 `propose_transfer` 和 `try_execute_transfer_from_callback` 两个阶段都会调用 `institution-asset`。
- 当前 runtime 规则下，制度保留 `main_account` 只允许本模块这类治理执行动作内部扣款。
- 这样可以防止其他交易模块绕开治理流程直接动用受治理资金账户余额。

## 2. Extrinsic 接口

### 2.1 propose_transfer — 发起转账提案

```rust
pub fn propose_transfer(
    origin: OriginFor<T>,
    institution_code: [u8; 4],          // CID 机构码（NRC / PRC / PRB / PMUL / 机构账户码）
    institution: AccountId,   // 机构 pallet id [u8; 48]
    beneficiary: T::AccountId,          // 收款地址
    amount: BalanceOf<T>,               // 转账金额
    remark: BoundedVec<u8, T::MaxRemarkLen>, // 备注
) -> DispatchResult
```

**校验规则：**

1. `origin` 必须是 `signed`，提取 `proposer = ensure_signed(origin)`。
2. `amount > 0`。
3. `institution` 必须是有效机构：
   - 治理机构：在 CHINA_CB / CHINA_CH 中存在；
   - 个人多签账户：能从 `PersonalAccount AccountId` 解码出账户，且对应 `PersonalAdmins::PersonalAccounts` 处于 Active；
   - 注册型机构账户：能从 `InstitutionAccount AccountId` 解码出账户，且对应 `OrganizationManage::InstitutionAccounts` 处于 Active；
   - `0x02 注册机构归属关系` 只用于机构归属/检索，不能作为转账支出主体。
4. `institution_code` 必须与 `institution` 的实际机构类型匹配。
5. `proposer` 必须是该机构的当前管理员（通过 `InternalAdminProvider::is_internal_admin` 校验，生产 runtime 最终委托 `RuntimeAdminAccountQuery`）。
6. `amount >= ED`（转账金额不能低于存在性保证金，防止收款地址创建失败）。
7. `beneficiary` 不能是机构自身的主账户（不允许自转账）。
8. `beneficiary` 不能是受保护地址（如 `stake_account`、安全基金账户、费用账户等保留地址）。
9. 转出资金账户的可用余额 >= `amount + fee + ED`（预检含手续费，防止创建必定无法执行的提案）。
10. 活跃提案数由 `votingengine` 在 `create_internal_proposal_with_data` 中统一检查（全局限额）。

**执行逻辑：**

1. 编码 `MODULE_TAG + TransferAction { institution, beneficiary, amount, remark, proposer }`。
2. 调用 `InternalVoteEngine::create_internal_proposal_with_data(proposer, institution_code, institution, MODULE_TAG, encoded)` 获取 `proposal_id`，并原子写入 owner/data/meta。
3. 发出 `TransferProposed` 事件。

### 2.2 投票入口

本模块不提供独立的投票/超时结算 extrinsic。管理员投票统一走:

```rust
InternalVote::cast(origin, proposal_id, approve)  // pallet 22.0
```

投票引擎根据提案创建时的管理员快照和阈值快照做权限、防双投和阈值判定。达到通过阈值后，投票引擎回调本模块的 `InternalVoteExecutor` 自动执行转账。

### 2.3 手动重试 / 取消入口

本 pallet 不暴露任何业务 wrapper extrinsic。前端直接调用投票引擎公开 extrinsic:

- 手动重试: `VotingEngine::retry_passed_proposal(proposal_id)`(pallet 9.4)
- 取消失败提案: `VotingEngine::cancel_passed_proposal(proposal_id, reason)`(pallet 9.5)

投票引擎在 `InternalVoteExecutor` 回调阶段会自动调用本 pallet 的
`try_execute_transfer_from_callback` 完成业务执行;手动重试也走相同回调。

## 3. 存储项

本模块**自身不定义存储项**。所有提案数据统一存储在 `votingengine` 中：

| 存储位置 | Key | Value | 说明 |
| --- | --- | --- | --- |
| `votingengine::ProposalData` | `u64` | `Vec<u8>`（编码的 `TransferAction`） | 提案业务数据 |
| `votingengine::ProposalOwner` | `u64` | `MODULE_TAG` | 业务 owner，禁止跨模块覆写 |
| `votingengine::ProposalMeta` | `u64` | `ProposalMetadata` | 提案元数据（创建块号等） |
| `votingengine::Proposals` | `u64` | `Proposal` | 提案核心状态（status、timing） |
| `SafetyFundProposalActions` | `u64` | `SafetyFundAction` | 安全基金动作独立存储，owner 仍为 `MODULE_TAG` |
| `SweepProposalActions` | `u64` | `SweepAction` | 费用划转动作独立存储，owner 仍为 `MODULE_TAG` |

### 3.1 TransferAction 结构

```rust
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TransferAction<AccountId, Balance, MaxRemarkLen: Get<u32>> {
    pub institution: AccountId,       // 转出机构
    pub beneficiary: AccountId,                  // 收款地址
    pub amount: Balance,                         // 转账金额
    pub remark: BoundedVec<u8, MaxRemarkLen>,    // 备注
    pub proposer: AccountId,                     // 发起管理员
}
```

## 4. 事件

```rust
#[pallet::event]
pub enum Event<T: Config> {
    /// 转账提案已创建
    TransferProposed {
        proposal_id: u64,
        institution_code: [u8; 4],
        institution: AccountId,
        proposer: T::AccountId,
        from: T::AccountId,                             // 转出资金账户
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        remark: BoundedVec<u8, T::MaxRemarkLen>,
        expires_at: BlockNumberFor<T>,                  // 投票超时区块
    },
    /// 投票通过但执行失败(可通过 VotingEngine::retry_passed_proposal 手动重试)
    TransferExecutionFailed { proposal_id: u64, institution: AccountId },
    /// 转账已执行(含手续费分账)
    TransferExecuted {
        proposal_id: u64,
        institution: AccountId,
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        fee: BalanceOf<T>,
    },
    // 安全基金组:结构同上
    SafetyFundTransferProposed {
        proposal_id: u64,
        proposer: T::AccountId,
        from: T::AccountId,                             // SAFETY_FUND_ACCOUNT
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        remark: BoundedVec<u8, T::MaxRemarkLen>,
        expires_at: BlockNumberFor<T>,
    },
    SafetyFundTransferExecuted { proposal_id: u64, beneficiary: T::AccountId, amount: BalanceOf<T>, fee: BalanceOf<T> },
    SafetyFundExecutionFailed { proposal_id: u64 },

    // Sweep 组:
    SweepToMainProposed {
        proposal_id: u64,
        institution: AccountId,
        proposer: T::AccountId,
        from: T::AccountId,                             // fee_account
        to: T::AccountId,                               // main_account
        amount: BalanceOf<T>,
        expires_at: BlockNumberFor<T>,
    },
    SweepToMainExecuted { proposal_id: u64, institution: AccountId, amount: BalanceOf<T>, fee: BalanceOf<T>, reserve_left: BalanceOf<T> },
    SweepExecutionFailed { proposal_id: u64 },
}
```

投票事件统一由 `votingengine::InternalVoteCast`、`ProposalFinalized`、`ProposalExecutionRetryScheduled`、`ProposalExecutionRetried` 等事件表达。

## 5. 错误码

```rust
#[pallet::error]
pub enum Error<T> {
    InvalidInstitution,              // 机构不存在
    InstitutionCodeMismatch,         // institution_code 与机构类型不匹配
    UnauthorizedAdmin,               // 非该机构管理员(propose 阶段)
    ZeroAmount,                      // 金额为零
    AmountBelowExistentialDeposit,   // 金额低于 ED
    SelfTransferNotAllowed,          // 不能转给自己
    BeneficiaryIsProtectedAddress,   // 收款地址是受保护地址
    ProposalActionNotFound,          // 提案不存在或数据解码失败
    InstitutionAccountDecodeFailed,  // 机构地址解码失败
    InsufficientBalance,             // 余额不足(amount + fee + ED)
    ProposalNotPassed,               // 提案未通过(retry/cancel 校验由 VotingEngine 承担)
    TransferFailed,                  // 转账执行失败
    // safety_fund / sweep 专有
    SafetyFundProposalNotFound, SafetyFundInsufficientBalance, SafetyFundProposalNotPassed,
    SweepProposalNotFound, InvalidSweepAmount, InsufficientFeeReserve, SweepAmountExceedsCap, SweepProposalNotPassed,
}
```

## 6. 手续费机制

### 6.1 计费规则

由 `onchain-transaction::calculate_onchain_fee()` 计算：

- 基础手续费 = `max(amount × ONCHAIN_FEE_RATE, ONCHAIN_MIN_FEE)`
- `ONCHAIN_FEE_RATE` = 0.1%（`Perbill::from_parts(1_000_000)`）
- `ONCHAIN_MIN_FEE` = 10 分 = 0.1 元
- 按"分"四舍五入

### 6.2 手续费处理方式

提案提交和投票交易不是免费交易：

- `MultisigTransfer::propose_transfer / propose_safety_fund_transfer / propose_sweep_to_main` 由签名管理员钱包按转账金额计费（`amount × 0.1%`，最低 0.1 元）。
- `InternalVote::cast` 由投票管理员钱包按 `VOTE_FLAT_FEE = 1 元` 计费。
- 多签资金账户仍需在执行阶段承担实际转账金额、内部手续费和 ED 保留要求。

投票通过后，pallet 的 `try_execute_transfer_from_callback` 内部还会处理转出账户侧的执行费用：

1. 通过 `calculate_onchain_fee(amount)` 计算手续费。
2. 校验余额 >= `amount + fee + ED`。
3. 执行 `Currency::transfer()` 转账。
4. 执行 `Currency::withdraw()` 扣取手续费。
5. 通过 `FeeRouter` 按规则分账。

因此前端必须同时提示两类余额：管理员钱包余额不足会导致提案/投票交易被交易支付扩展拒绝；多签账户余额不足会导致提案执行失败。

### 6.3 手续费分账

按 `TransferFeeRouter`（复用 `OnchainFeeRouter` 规则）：
- 80% → 全节点出块者
- 10% → 国储会
- 10% → 安全基金账户

## 7. 转账执行逻辑

### 7.1 自动执行流程

`InternalVote::cast` 达到阈值后，投票引擎进入 `STATUS_PASSED` 并在同一事务内回调本模块自动执行：

```
1. 最后一票触发 votingengine 的 STATUS_PASSED 判定
2. votingengine 调用 InternalVoteExecutor::on_internal_vote_finalized(proposal_id, approved=true)
3. 本模块按 ProposalOwner / ProposalData / 独立 action storage 认领 transfer、safety_fund 或 sweep
4. 执行业务转账:
   a. 解析资金源和目标账户
   b. 计算手续费 fee = calculate_onchain_fee(amount)
   c. 校验余额与 ED
   d. Currency::transfer(...)
   e. Currency::withdraw(..., FEE, KeepAlive) → FeeRouter 分账
   f. deposit_event(*Executed)
5. 执行成功返回 ProposalExecutionOutcome::Executed
6. 执行失败发 *ExecutionFailed，返回 ProposalExecutionOutcome::RetryableFailed
```

### 7.2 提案状态流转

```
VOTING → PASSED（待执行） → EXECUTED（已执行，终态）
                ↓ 执行失败
           保持 PASSED（可通过 VotingEngine::retry_passed_proposal 重试）
                ↓ 3 次手动失败或 deadline 到期
           EXECUTION_FAILED（终态）
```

- `VOTING`（0）：投票进行中
- `PASSED`（1）：投票通过，待执行或执行失败待重试
- `REJECTED`（2）：投票超时未达阈值
- `EXECUTED`（3）：执行成功，终态，无法再次执行
- `EXECUTION_FAILED`（4）：重试失败满 3 次或超过宽限期，终态

### 7.3 余额保护

- 使用 `ExistenceRequirement::KeepAlive` 确保转账后机构账户不被 reap（删除）。
- 执行时校验 `free_balance >= amount + fee + ED`。

### 7.4 转账 vs 销毁

| 操作 | resolution-destro | multisig-transfer |
| --- | --- | --- |
| API | `Currency::slash()` | `Currency::transfer()` |
| 总发行量 | 减少（资金销毁） | 不变（资金转移） |
| 目标 | 无（资金消失） | `beneficiary` 账户 |

## 8. 提案与投票区块写入

### 8.1 时序

```
区块 N  : 管理员A 发起 propose_transfer(...)
           → 创建提案 proposal_id=X
           → emit TransferProposed { from, beneficiary, amount, remark, expires_at, ... }

区块 N+k: 快照管理员逐个提交 InternalVote::cast(X, approve=true)
           → STATUS_PASSED 达阈值
           → 同一交易内 callback 自动执行
           → emit TransferExecuted 或 TransferExecutionFailed
```

### 8.2 关键差异

- 投票完全复用 `votingengine::internal_vote`，不再有业务 pallet 自己的 vote/finalize 状态机。
- 幂等保护由投票引擎的 `InternalVotesByAccount` / `AlreadyVoted` 统一提供。
- 手动重试、取消、3 次失败终态和 deadline 终态由投票引擎统一处理。

### 8.3 投票阈值

| 机构类型 | 阈值 |
| --- | --- |
| NRC（国储会） | 13 |
| PRC（省储会） | 6 |
| PRB（省储行） | 6 |

## 9. 投票结果展示

### 9.1 链上存储（投票引擎已有）

| 存储 | 说明 |
| --- | --- |
| `InternalVotesByAccount<(proposal_id, AccountId)>` → `bool` | 每位管理员的投票记录 |
| `InternalTallies<proposal_id>` → `{ yes: u32, no: u32 }` | 赞成/反对计数 |
| `Proposals<proposal_id>` → `Proposal` | 提案状态（voting/passed/rejected） |

### 9.2 App 端展示

App 可通过 `state_getStorage` 查询上述存储项，展示：
- 提案当前状态（投票中/已通过/已拒绝）
- 赞成票数 / 反对票数 / 阈值
- 每位管理员的投票明细（赞成/反对/未投票）

## 10. Config Trait

```rust
#[pallet::config]
pub trait Config:
        frame_system::Config + votingengine::Config + organization_manage::Config
{
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// 备注最大长度
    #[pallet::constant]
    type MaxRemarkLen: Get<u32>;

    /// 手续费分账路由（复用 OnchainFeeRouter）
    type FeeRouter: frame_support::traits::OnUnbalanced<
        <<Self as organization_manage::Config>::Currency as Currency<Self::AccountId>>::NegativeImbalance,
    >;

    /// 个人多签账户状态查询，由 personal-manage 实现。
    type PersonalQuery: personal_manage::traits::PersonalMultisigQuery<Self::AccountId>;

    /// 注册机构账户状态查询，由 organization-manage 实现。
    type InstitutionQuery: organization_manage::traits::InstitutionMultisigQuery<Self::AccountId>;

    /// Weight 配置
    type WeightInfo: crate::weights::WeightInfo;
}
```

说明：`Currency`、`InternalVoteEngine`、`ProtectedSourceChecker`、`InstitutionAsset` 等类型由上游 `organization_manage::Config` 和 `votingengine::Config` 提供；个人/机构注册账户状态通过本模块的 `PersonalQuery` / `InstitutionQuery` 配置项注入。机构账户提案的实际 institution_code 由 `InstitutionQuery::lookup_institution_code(account)` 返回，必须是机构账户码（`is_institution_code`），传个人多签码（`is_personal_code`）会被 `InstitutionCodeMismatch` 拒绝。

## 11. Weight 估算

| Extrinsic | 预估 Weight | DB 读 | DB 写 |
| --- | --- | --- | --- |
| `propose_transfer` | ~55 ms | 5 | 7 |
| `propose_safety_fund_transfer` | 待 benchmark | - | - |
| `propose_sweep_to_main` | 待 benchmark | - | - |

说明：投票权重由 `votingengine::internal_vote` 承担；手动重试走 `VotingEngine::retry_passed_proposal`，权重由投票引擎统一计入。本模块 2026-05-02 起不再保留 `execute_xxx` wrapper。正式数值需重新跑 benchmark 生成。

## 12. 文件清单

| 文件 | 说明 |
| --- | --- |
| `src/lib.rs` | Pallet 主体（Config、Event、Error、Extrinsics、TransferAction） |
| `src/weights.rs` | Weight 定义（先用占位值，后续 benchmark 生成） |
| `src/benchmarks.rs` | 基准测试 |
| `Cargo.toml` | 依赖声明 |
| `MULTISIG_TRANSFER_TECHNICAL.md` | 本技术文档 |

## 13. Runtime 集成要点

### 13.1 注册 pallet

在 `runtime/src/lib.rs` 中注册（pallet_index = 19）：
```rust
#[runtime::pallet_index(19)]
pub type MultisigTransfer = multisig_transfer;
```

在 `runtime/src/configs/mod.rs` 中配置：
```rust
impl multisig_transfer::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxRemarkLen = ConstU32<256>;
    type FeeRouter = TransferFeeRouter;
    type PersonalQuery = PersonalAdmins;
    type InstitutionQuery = OrganizationManage;
    type WeightInfo = multisig_transfer::weights::SubstrateWeight<Runtime>;
}
```

### 13.2 CallFeeKind 配置

`MultisigTransfer` 的 propose 系列 extrinsic 只负责创建治理提案，交易本身按投票统一价 1 元计费；真正执行转账时，模块内部再按转出金额 `max(amount × 0.1%, 0.1 元)` 扣链上交易费：
```rust
RuntimeCall::MultisigTransfer(ref dt_call) => match dt_call {
    multisig_transfer::pallet::Call::propose_transfer { .. }
    | multisig_transfer::pallet::Call::propose_safety_fund_transfer { .. }
    | multisig_transfer::pallet::Call::propose_sweep_to_main { .. } => {
        onchain_transaction::FeeChargeKind::VoteFlat
    }
    _ => onchain_transaction::FeeChargeKind::VoteFlat,
}
```

### 13.3 Benchmark 注册

在 `define_benchmarks!` 中添加 `[multisig_transfer, MultisigTransfer]`。
