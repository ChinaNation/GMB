# Multisig Transfer 技术文档（多签资金账户转账模块）

> 机构唯一身份真源是 `actor_cid_number`；具体资金账户只表达本次资产来源，必须通过 entity 正反索引归属于该 CID。个人多签不使用机构 CID，以 `personal_account` 为主体。

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
| 0 | `propose_transfer` | 发起多签资金账户转账提案 |
| 1 | `propose_safety_fund_transfer` | 发起安全基金转账提案 |
| 2 | `propose_sweep_to_main` | 发起费用账户划转主账户提案 |

投票走统一入口 `InternalVote::cast`(pallet 20.0),手动重试/取消走
`VotingEngine::retry_passed_proposal`(9.4)/`cancel_passed_proposal`(9.5)。

## 2026-05-08 · 第5步账户级主体接入

- 机构转账显式携带 `actor_cid_number + funding_account`；任何机构账户都不得替代 CID 作为机构身份。
- 治理机构与注册机构使用同一模型：CID 下挂多个协议账户或自定义账户，账户正反索引用于证明 `funding_account` 属于该 CID。
- 个人多签直接使用个人多签 `AccountId` 作为资金账户，账户状态由 `personal-manage::PersonalMultisigQuery` 校验；管理员真源由 `personal-admins` 提供。
- 注册机构具体账户直接使用机构账户 `AccountId` 作为资金账户，账户状态由 `entity-primitives::InstitutionMultisigQuery` 校验。
- 管理员、阈值和人数通过唯一查询出口读取：个人多签走 `personal_account`，机构走 CID；内部投票仍是一人一票一笔链上交易。

## 0. 功能需求

### 0.1 核心职责

`multisig-transfer` 负责多签资金账户通过内部投票引擎发起转账：

- 多签资金账户管理员发起转账提案，指定收款地址、金额和备注。
- 多签资金账户管理员通过内部投票引擎逐票投票。
- 投票通过后执行转账：从提案绑定的资金账户向收款地址划转资金。
- 执行手续费通过 `fee_policy::calculate_onchain_fee()` 计算，并由统一 `OnchainFeeCharger` 收取：机构路径只扣 actor CID 费用账户，个人多签路径扣个人账户。
- 管理员钱包只承担自己发起的个人操作费和实际投票费，绝不承担机构操作费或机构资金执行费。
- 覆盖两种身份模型：
  - 机构：`actor_cid_number + funding_account + origin(admin)`；资金账户可以是该 CID 下允许支出的主账户、费用账户、安全基金账户或自定义账户，具体业务仍受 `institution-asset` 限制。
  - 个人多签：`personal_account + origin(admin)`，不携带机构 CID。

### 0.2 功能边界

- 本模块处理三类多签账户转账：
  - 创世预置的治理机构（NRC / PRC / PRB）
  - `personal-manage` 注册并处于 Active 状态的个人多签账户（个人多签码 `PMUL`，`is_personal_code`）
  - 实体生命周期模块注册并处于 Active 状态的机构账户（机构账户码 `is_institution_code`）
- 任一接入机构必须遵循同一 CID 身份模型；是否允许某具体账户支出由账户类型与 `institution-asset` 业务规则决定。
- 联邦注册局 `FRG` 是一个 CID 下挂多个账户、多个管理员和岗位任职的机构；省域岗位组只表达岗位权限，不是独立机构或独立管理员真源。
- 本模块不负责投票引擎实现，投票逻辑委托给 `votingengine` 的 `InternalVoteEngine`。
- 执行回调不是单凭 `proposal_id + PASSED` 放行：必须处于投票引擎 callback scope，并同时匹配 `ProposalOwner`、内部投票 kind/stage、业务 action、机构码、资金账户和 CID 集合。执行前还会重新读取当前 entity 生命周期与业务权限，防止提案创建后账户失活或上下文被替换。
- 本模块不负责个人多签账户创建、关闭、清理或管理员集合变更；这些职责分别归属 `personal-manage` 和 `personal-admins`。

补充说明：
- 只要某类机构的 CID 和账户正反索引已接入 `RuntimeInstitutionQuery`，
- 且对应 CID 的管理员已接入 runtime 的 `RuntimeInternalAdminProvider`；固定阈值、机构 CID 动态阈值或单例机构的提案快照严格过半均由投票引擎自身提供，
- 这类机构就可以直接复用本模块和内部投票引擎发起转账提案，不需要新增转账 pallet。

### 0.3 与多签管理模块的关系

| 模块 | 职责 | 地址类型 | 审批方式 |
| --- | --- | --- | --- |
| `personal-manage` | 个人多签账户生命周期 | 个人多签账户 | `PMUL` 内部投票 |
| `personal-admins` | 个人多签管理员真源和管理员集合变更 | 个人多签账户管理员集合 | `PMUL` 内部投票 |
| `public-manage` | 公权机构生命周期 | 公权机构账户 | CID 注册凭证 + 公权机构码内部投票 |
| `private-manage` | 私权机构生命周期 | 私权机构账户 | CID 注册凭证 + 私权机构码内部投票 |
| `multisig-transfer` | 多签资金账户转账 | CID 下允许支出的机构账户 + Active 个人多签账户 | 链上内部投票引擎（逐票投票） |

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
| `funding_account` | 具体资金账户 | 机构交易必须归属于显式 CID；个人多签时就是 `personal_account` |

### 1.2 资金账户来源

资金账户有两种来源：

- 机构账户：调用必须携带 `Some(actor_cid_number)`；`RuntimeInstitutionQuery` 校验账户存在，并通过 `lookup_cid/lookup_org` 证明其归属与显式 CID、机构码一致。管理员集合和机构阈值均按 CID 查询。
- 个人多签账户：调用必须携带 `None`；直接使用 `personal-manage` 派生并激活的 `personal_account`，管理员集合和个人阈值按该账户查询。

### 1.3 institution-asset 边界

- 本模块在 `propose_transfer` 和 `try_execute_transfer_from_callback` 两个阶段都会调用 `institution-asset`。
- runtime 按账户类型和业务 action 判断具体账户能否支出，不允许以“它是主账户”替代 CID 身份或授权校验。
- 这样可以防止其他交易模块绕开治理流程直接动用受治理资金账户余额。

## 2. Extrinsic 接口

### 2.1 propose_transfer — 发起转账提案

```rust
pub fn propose_transfer(
    origin: OriginFor<T>,
    actor_cid_number: Option<CidNumber>, // 机构为 Some(CID)，个人多签严格为 None
    funding_account: AccountId,          // 实际转出资金账户
    beneficiary: T::AccountId,          // 收款地址
    amount: BalanceOf<T>,               // 转账金额
    remark: BoundedVec<u8, T::MaxRemarkLen>, // 备注
) -> DispatchResult
```

**校验规则：**

1. `origin` 必须是 `signed`，提取 `proposer = ensure_signed(origin)`。
2. `amount > 0`。
3. 机构路径要求 `Some(actor_cid_number)`：`funding_account` 必须存在于机构账户正反索引，且 `lookup_cid/lookup_org` 与显式 CID 及其机构码一致。
4. 个人多签路径要求 `None`：`personal-manage` 必须判定 `funding_account` 处于 Active。
5. 机构路径按 `is_institution_admin(institution_code, actor_cid_number, proposer)` 授权；个人路径按 `is_personal_admin(funding_account, proposer)` 授权。
6. `amount >= ED`（转账金额不能低于存在性保证金，防止收款地址创建失败）。
7. `beneficiary` 不能是转出资金账户自身（不允许自转账）。
8. `beneficiary` 不能是受保护地址（如 `stake_account`、安全基金账户、费用账户等保留地址）。
9. 机构路径分别预检 `funding_account >= amount + ED`、actor CID 费用账户 `>= fee + ED`；个人路径预检 `funding_account >= amount + fee + ED`。
10. 活跃提案数由 `votingengine` 在 `create_internal_proposal_with_data` 中统一检查（全局限额）。

**执行逻辑：**

1. 编码 `MODULE_TAG + TransferAction { actor_cid_number, funding_account, beneficiary, amount, remark, proposer }`。
2. 机构调用 `create_institution_proposal_with_data`，个人多签调用 `create_personal_proposal_with_data`；二者都原子写入 owner/data/meta，机构提案同时绑定 CID 和执行账户。
3. 发出 `TransferProposed` 事件。

### 2.2 投票入口

本模块不提供独立的投票/超时结算 extrinsic。管理员投票统一走:

```rust
InternalVote::cast(origin, proposal_id, approve)  // pallet 20.0
```

投票引擎根据提案创建时的管理员快照和阈值快照做权限、防双投和阈值判定。达到通过阈值后，投票引擎回调本模块的 `InternalVoteExecutor` 自动执行转账。

### 2.3 手动重试 / 取消入口

本 pallet 不暴露任何业务 wrapper extrinsic。前端直接调用投票引擎公开 extrinsic:

- 手动重试: `VotingEngine::retry_passed_proposal(proposal_id)`(pallet 9.4)
- 取消失败提案: `VotingEngine::cancel_passed_proposal(proposal_id, reason)`(pallet 9.5)

投票引擎在 `InternalVoteExecutor` 回调阶段会自动调用本 pallet 的
`try_execute_transfer_from_callback` 完成业务执行;手动重试也走相同回调。

## 3. 存储项

普通转账提案数据统一存储在 `votingengine` 中；安全基金和费用划转保留本模块本地动作表：

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
    pub actor_cid_number: Option<CidNumber>,     // 机构 Some(CID)，个人 None
    pub funding_account: AccountId,              // 实际转出资金账户
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
        actor_cid_number: Option<CidNumber>,
        proposer: T::AccountId,
        funding_account: T::AccountId,
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        remark: BoundedVec<u8, T::MaxRemarkLen>,
        expires_at: BlockNumberFor<T>,                  // 投票超时区块
    },
    /// 投票通过但执行失败(可通过 VotingEngine::retry_passed_proposal 手动重试)
    TransferExecutionFailed { proposal_id: u64, funding_account: AccountId },
    /// 转账已执行(含手续费分账)
    TransferExecuted {
        proposal_id: u64,
        funding_account: AccountId,
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        fee: BalanceOf<T>,
    },
    // 安全基金组:结构同上
    SafetyFundTransferProposed {
        proposal_id: u64,
        actor_cid_number: CidNumber,
        proposer: T::AccountId,
        institution_account: T::AccountId,
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
        actor_cid_number: CidNumber,
        proposer: T::AccountId,
        institution_account: T::AccountId,              // fee_account
        main_account: T::AccountId,
        amount: BalanceOf<T>,
        expires_at: BlockNumberFor<T>,
    },
    SweepToMainExecuted { proposal_id: u64, actor_cid_number: CidNumber, institution_account: AccountId, amount: BalanceOf<T>, fee: BalanceOf<T>, reserve_left: BalanceOf<T> },
    SweepExecutionFailed { proposal_id: u64 },
}
```

投票事件统一由 `votingengine::InternalVoteCast`、`ProposalFinalized`、`ProposalExecutionRetryScheduled`、`ProposalExecutionRetried` 等事件表达。

## 5. 错误码

```rust
#[pallet::error]
pub enum Error<T> {
    InvalidInstitution,              // 多签资金账户不存在或未激活
    InstitutionCodeMismatch,         // institution_code 与资金账户分类不匹配
    UnauthorizedAdmin,               // 非该多签资金账户管理员(propose 阶段)
    ZeroAmount,                      // 金额为零
    AmountBelowExistentialDeposit,   // 金额低于 ED
    SelfTransferNotAllowed,          // 不能转给自己
    BeneficiaryIsProtectedAddress,   // 收款地址是受保护地址
    ProposalActionNotFound,          // 提案不存在或数据解码失败
    InstitutionAccountDecodeFailed,  // 内置账户地址解码失败
    InsufficientBalance,             // 本金账户余额不足；个人路径同时覆盖 amount + fee + ED
    InsufficientFeeBalance,          // 机构费用账户不足 fee + ED
    ProposalNotPassed,               // 提案未通过(retry/cancel 校验由 VotingEngine 承担)
    TransferFailed,                  // 转账执行失败
    // safety_fund / sweep 专有
    SafetyFundProposalNotFound, SafetyFundInsufficientBalance, SafetyFundProposalNotPassed,
    SweepProposalNotFound, InvalidSweepAmount, InsufficientFeeReserve, SweepAmountExceedsCap, SweepProposalNotPassed,
}
```

## 6. 手续费机制

### 6.1 计费规则

由 `primitives::fee_policy::calculate_onchain_fee()` 计算：

- 基础手续费 = `max(amount × ONCHAIN_FEE_RATE, ONCHAIN_MIN_FEE)`
- `ONCHAIN_FEE_RATE` = 0.1%（`Perbill::from_parts(1_000_000)`）
- `ONCHAIN_MIN_FEE` = 10 分 = 0.1 元
- 按"分"四舍五入

### 6.2 手续费处理方式

提案提交和投票交易不是同一类费用：

- 机构 `propose_transfer / propose_safety_fund_transfer / propose_sweep_to_main` 是链上机构操作，由 actor CID 的费用账户支付 0.1 元；管理员钱包只签名。
- 个人多签 `propose_transfer` 是普通链上操作，由签名者支付 0.1 元。
- `InternalVote::cast` 才是实际投票，由投票管理员钱包支付 1 元。
- 投票通过后的机构资金执行中，具体 `funding_account` 或安全基金账户只支付本金，actor CID 的费用账户支付执行手续费；sweep 的本金和执行手续费都由其明确的费用账户支付。
- 个人多签执行中，个人账户同时支付本金和执行手续费。任何路径都不允许改扣提案管理员钱包。

投票通过后，pallet 通过统一 `OnchainFeeCharger` 处理执行费用：

1. 通过 `calculate_onchain_fee(amount)` 计算手续费。
2. 机构普通转账分别校验资金账户 `amount + ED`、费用账户 `fee + ED`；个人转账校验个人账户 `amount + fee + ED`。
3. 安全基金转账分别校验安全基金账户 `amount + ED`、国家储委会费用账户 `fee + ED`；sweep 校验费用账户 `amount + fee + ED`。
4. 在同一 storage transaction 中调用 `OnchainFeeCharger::charge()` 并执行本金转账。
5. 任一扣款或转账失败全部回滚；成功后通过统一分账器按 80/10/10 分账并在执行事件中记录 `fee_payer`（sweep 的 `institution_account` 本身即付款账户）。

因此前端必须区分四项余额责任：机构提案操作费、机构执行费检查 actor CID 费用账户，机构执行本金检查具体资金账户，个人提案/个人执行检查个人账户，实际投票检查投票签名者；不得用管理员钱包为机构费用兜底。

### 6.3 手续费分账

按 `OnchainExecutionFeeDistributor`（复用 `OnchainFeeRouter` 规则）：
- 80% → 全节点出块者
- 10% → 国家储委会
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
   c. 按机构/个人路径分别校验本金账户、确切费用付款账户与 ED
   d. 在同一 storage transaction 中调用 OnchainFeeCharger::charge(...)
   e. Currency::transfer(...)；失败时连同手续费、分账和事件一起回滚
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

- 使用 `ExistenceRequirement::KeepAlive` 确保转账后资金账户不被 reap（删除）。
- 机构执行分别校验具体资金账户 `amount + ED` 和 actor CID 费用账户 `fee + ED`；个人执行校验个人账户 `amount + fee + ED`。

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
| NRC（国家储委会） | 13 |
| PRC（省储委会） | 6 |
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
        frame_system::Config + votingengine::Config
{
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// 备注最大长度
    #[pallet::constant]
    type MaxRemarkLen: Get<u32>;

    /// 执行期链上费统一收取器；付款账户由本模块按 actor CID/个人账户精确确定。
    type OnchainFeeCharger: primitives::fee_policy::OnchainFeeCharger<
        Self::AccountId,
        BalanceOf<Self>,
    >;

    /// 个人多签账户状态与管理员配置查询，由 personal-manage 聚合 personal-admins 提供。
    type PersonalQuery: personal_manage::traits::PersonalMultisigQuery<Self::AccountId>;

    /// 注册机构账户状态与管理员配置查询，由 runtime 聚合 public-manage / private-manage 提供。
    type InstitutionQuery: entity_primitives::InstitutionMultisigQuery<Self::AccountId>;

    /// Weight 配置
    type WeightInfo: crate::weights::WeightInfo;
}
```

说明：`Currency`、`InternalVoteEngine`、`ProtectedSourceChecker`、`InstitutionAsset` 等类型由本模块 Config 和 `votingengine::Config` 提供；个人/机构注册账户状态通过本模块的 `PersonalQuery` / `InstitutionQuery` 配置项注入。个人多签账户的实际 `institution_code` 为 `PMUL`；机构账户的实际 `institution_code` 由 `InstitutionQuery::lookup_org(account)` 返回且必须满足 `is_institution_code`。

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

在 `runtime/src/lib.rs` 中注册（pallet_index = 17）：
```rust
#[runtime::pallet_index(17)]
pub type MultisigTransfer = multisig;
```

在 `runtime/src/configs/mod.rs` 中配置：
```rust
impl multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxRemarkLen = ConstU32<256>;
    type OnchainFeeCharger =
        onchain::OnchainExecutionFeeCharger<Runtime, Balances, OnchainExecutionFeeDistributor>;
    type PersonalQuery = PersonalManage;
    type InstitutionQuery = RuntimeInstitutionQuery;
    type WeightInfo = multisig::weights::SubstrateWeight<Runtime>;
}
```

### 13.2 `FeeRoute` 配置

`MultisigTransfer` 的 propose 系列 extrinsic 负责创建提案，但“发起提案”不是“投票”。当前统一规则为：

- 机构提案：显式携带 `actor_cid_number + funding_account/institution_account`，校验账户属于该 CID 后，从该 CID 的唯一费用账户扣 0.1 元。
- 个人多签提案：由签名者支付 0.1 元链上操作费。
- 管理员后续执行 `InternalVote::cast` 时，才由投票签名者支付 1 元投票费。
- 提案通过后的机构资金执行费只从 actor CID 费用账户支付，本金只从明确的机构账户支付；个人多签由个人账户支付。所有扣款原子执行，不得让机构管理员钱包垫付。

### 13.3 Benchmark 注册

在 `define_benchmarks!` 中添加 `[multisig, MultisigTransfer]`。
