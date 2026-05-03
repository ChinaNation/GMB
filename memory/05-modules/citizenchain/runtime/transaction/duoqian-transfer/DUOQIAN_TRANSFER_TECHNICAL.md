# DUOQIAN Transfer Pow 技术文档（机构多签名地址转账模块）

## 2026-04-30 · 统一投票引擎状态机改造

本模块所有 3 组业务（transfer / safety_fund / sweep）已统一接入 `voting-engine` 生命周期：

- 提案创建使用 `create_internal_proposal_with_data`，在同一事务中绑定 `ProposalOwner`、`ProposalData` 和 `ProposalMeta`。
- 管理员投票统一走 `VotingEngine::internal_vote(proposal_id, approve)`，本模块不再提供独立 vote/finalize call。
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
| 3 | (已废弃 2026-05-02) | 原 `execute_transfer` 已统一到 `VotingEngine::retry_passed_proposal` |
| 4 | (已废弃 2026-05-02) | 原 `execute_safety_fund_transfer` 已统一到 `VotingEngine::retry_passed_proposal` |
| 5 | (已废弃 2026-05-02) | 原 `execute_sweep_to_main` 已统一到 `VotingEngine::retry_passed_proposal` |

## 0. 功能需求

### 0.1 核心职责

`duoqian-transfer` 负责机构多签名地址通过内部投票引擎发起转账：

- 机构管理员发起转账提案，指定收款地址、金额和备注。
- 机构管理员通过内部投票引擎逐票投票。
- 投票通过后执行转账：从机构主账户地址向收款地址划转资金。
- 手续费在投票通过后由 pallet 内部从机构主账户扣取，通过 `onchain-transaction::calculate_onchain_fee()` 计算。
- 管理员个人账户不承担任何费用。
- 覆盖两类来源：
  - 创世预置的治理机构 `main_address`（NRC / PRC / PRB）
  - `duoqian-manage` 注册并激活的 `ORG_DUOQIAN` 多签地址（`action.duoqian_address`）

### 0.2 功能边界

- 本模块处理两类机构转账：
  - 创世预置的治理机构（NRC / PRC / PRB）
  - `duoqian-manage` 注册并处于 Active 状态的多签机构（`ORG_DUOQIAN`）
- 当前也尚未接入新补充的内置机构 `ZF / LF / JC / JY / SF`。
- 本模块不负责投票引擎实现，投票逻辑委托给 `voting-engine` 的 `InternalVoteEngine`。

补充说明：
- 只要某类内置机构被本模块的 `institution_org()` / `institution_pallet_address()` 正式识别，
- 且对应管理员与阈值已接入 runtime 的 `RuntimeInternalAdminProvider / RuntimeInternalThresholdProvider`，
- 这类机构就可以直接复用本模块和内部投票引擎发起转账提案，不需要新增转账 pallet。

### 0.3 与 `duoqian-manage` 的关系

| 模块 | 职责 | 地址类型 | 审批方式 |
| --- | --- | --- | --- |
| `duoqian-manage` | 多签名地址的注册、创建、关闭 | 注册的非治理机构 | `sfid` 主签名登记 + `ORG_DUOQIAN` 内部投票 |
| `duoqian-transfer` | 多签名地址转账 | 预置治理机构 + 注册型 Active 多签机构 | 链上内部投票引擎（逐票投票） |

### 0.4 与 `resolution-destro` 的关系

两者结构高度一致，区别在于资金操作：

| 对比 | `resolution-destro` | `duoqian-transfer` |
| --- | --- | --- |
| 资金操作 | `Currency::slash()` 销毁 | `Currency::transfer()` 转账 |
| 目标 | 销毁机构持有的代币 | 转账到指定收款地址 |
| 额外字段 | 无 | `beneficiary`、`remark` |

## 1. 地址说明

### 1.1 关键区分

| 地址 | 类型 | 说明 |
| --- | --- | --- |
| `stake_address` | 质押地址 | **不允许支出**，仅用于质押 |
| `main_address`（治理机构）/ `duoqian_address`（注册多签） | 机构主账户/注册多签主账户 | 机构资金账户，转账和手续费均从此扣取 |

### 1.2 机构主账户地址来源

机构主账户地址有两种来源：

- 治理机构：`main_address` 预置于 `runtime/primitives/china/china_cb.rs`（NRC + PRC）和 `runtime/primitives/china/china_ch.rs`（PRB）中，通过 `institution_pallet_address(institution_id)` 查找。
- 注册型机构：`InstitutionPalletId(48)` 采用主账户地址 `AccountId(32) + 16 字节 0` 编码，资金账户仍从 `duoqian-manage::DuoqianAccounts` 校验 Active，管理员、阈值和人数统一从 `admins-change::Institutions` 读取。

### 1.3 institution-asset 边界

- 本模块在 `propose_transfer` 和 `try_execute_transfer` 两个阶段都会调用 `institution-asset`。
- 当前 runtime 规则下，制度保留 `main_address` 只允许本模块这类治理执行动作内部扣款。
- 这样可以防止其他交易模块绕开治理流程直接动用机构主账户余额。

## 2. Extrinsic 接口

### 2.1 propose_transfer — 发起转账提案

```rust
pub fn propose_transfer(
    origin: OriginFor<T>,
    org: u8,                           // 机构类型：0=NRC, 1=PRC, 2=PRB, 3=DUOQIAN
    institution: InstitutionPalletId,   // 机构 pallet id [u8; 48]
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
   - 注册型机构：能解码出主账户地址，且在 `DuoqianAccounts` 中存在并处于 Active。
4. `org` 必须与 `institution` 的实际机构类型匹配。
5. `proposer` 必须是该机构的当前管理员（通过 `InternalAdminProvider::is_internal_admin` 校验，生产 runtime 最终读取 `admins-change::Institutions`）。
6. `amount >= ED`（转账金额不能低于存在性保证金，防止收款地址创建失败）。
7. `beneficiary` 不能是机构自身的主账户地址（不允许自转账）。
8. `beneficiary` 不能是受保护地址（如 `stake_address`、安全基金账户、费用账户等保留地址）。
9. 机构主账户的可用余额 >= `amount + fee + ED`（预检含手续费，防止创建必定无法执行的提案）。
10. 活跃提案数由 `voting-engine` 在 `create_internal_proposal_with_data` 中统一检查（全局限额）。

**执行逻辑：**

1. 编码 `MODULE_TAG + TransferAction { institution, beneficiary, amount, remark, proposer }`。
2. 调用 `InternalVoteEngine::create_internal_proposal_with_data(proposer, org, institution, MODULE_TAG, encoded)` 获取 `proposal_id`，并原子写入 owner/data/meta。
3. 发出 `TransferProposed` 事件。

### 2.2 投票入口

本模块不再提供独立 `vote_transfer` / `finalize_transfer`。管理员投票统一走：

```rust
VotingEngine::internal_vote(origin, proposal_id, approve)
```

投票引擎根据提案创建时的管理员快照和阈值快照做权限、防双投和阈值判定。达到通过阈值后，投票引擎回调本模块的 `InternalVoteExecutor` 自动执行转账。

### 2.3 已废弃: execute_transfer / execute_safety_fund_transfer / execute_sweep_to_main

2026-05-02 unified voting entry 整改后，本 pallet 的所有 `execute_xxx` wrapper extrinsic 物理删除。前端必须直接调用 voting-engine 公开 extrinsic：

- 手动重试: `VotingEngine::retry_passed_proposal(proposal_id)`
- 取消失败提案: `VotingEngine::cancel_passed_proposal(proposal_id, reason)`

投票引擎在 `InternalVoteExecutor` 回调阶段会自动调用本 pallet 的 `try_execute_transfer_from_callback` 完成业务执行；手动重试也走相同回调。

## 3. 存储项

本模块**自身不定义存储项**。所有提案数据统一存储在 `voting-engine` 中：

| 存储位置 | Key | Value | 说明 |
| --- | --- | --- | --- |
| `voting_engine::ProposalData` | `u64` | `Vec<u8>`（编码的 `TransferAction`） | 提案业务数据 |
| `voting_engine::ProposalOwner` | `u64` | `MODULE_TAG` | 业务 owner，禁止跨模块覆写 |
| `voting_engine::ProposalMeta` | `u64` | `ProposalMetadata` | 提案元数据（创建块号等） |
| `voting_engine::Proposals` | `u64` | `Proposal` | 提案核心状态（status、timing） |
| `SafetyFundProposalActions` | `u64` | `SafetyFundAction` | 安全基金动作独立存储，owner 仍为 `MODULE_TAG` |
| `SweepProposalActions` | `u64` | `SweepAction` | 费用划转动作独立存储，owner 仍为 `MODULE_TAG` |

### 3.1 TransferAction 结构

```rust
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TransferAction<AccountId, Balance, MaxRemarkLen: Get<u32>> {
    pub institution: InstitutionPalletId,       // 转出机构
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
        org: u8,
        institution: InstitutionPalletId,
        proposer: T::AccountId,
        from: T::AccountId,                             // 机构主账户
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        remark: BoundedVec<u8, T::MaxRemarkLen>,
        expires_at: BlockNumberFor<T>,                  // 投票超时区块
    },
    /// 投票通过但执行失败(可通过 VotingEngine::retry_passed_proposal 手动重试)
    TransferExecutionFailed { proposal_id: u64, institution: InstitutionPalletId },
    /// 转账已执行(含手续费分账)
    TransferExecuted {
        proposal_id: u64,
        institution: InstitutionPalletId,
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        fee: BalanceOf<T>,
    },
    // 安全基金组:结构同上
    SafetyFundTransferProposed {
        proposal_id: u64,
        proposer: T::AccountId,
        from: T::AccountId,                             // NRC_ANQUAN_ADDRESS
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
        institution: InstitutionPalletId,
        proposer: T::AccountId,
        from: T::AccountId,                             // fee_account
        to: T::AccountId,                               // main_account
        amount: BalanceOf<T>,
        expires_at: BlockNumberFor<T>,
    },
    SweepToMainExecuted { proposal_id: u64, institution: InstitutionPalletId, amount: BalanceOf<T>, fee: BalanceOf<T>, reserve_left: BalanceOf<T> },
    SweepExecutionFailed { proposal_id: u64 },
}
```

投票事件统一由 `voting-engine::InternalVoteCast`、`ProposalFinalized`、`ProposalExecutionRetryScheduled`、`ProposalExecutionRetried` 等事件表达。

## 5. 错误码

```rust
#[pallet::error]
pub enum Error<T> {
    InvalidInstitution,              // 机构不存在
    InstitutionOrgMismatch,          // org 与机构类型不匹配
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

提案提交和投票交易本身**免费**（`CallAmount` 返回 `NoAmount`）。手续费在投票通过后由 pallet 的 `try_execute_transfer` 内部处理：

1. 通过 `calculate_onchain_fee(amount)` 计算手续费。
2. 校验余额 >= `amount + fee + ED`。
3. 执行 `Currency::transfer()` 转账。
4. 执行 `Currency::withdraw()` 扣取手续费。
5. 通过 `FeeRouter` 按规则分账。

`RuntimeFeePayerExtractor` 对 `DuoqianTransfer` 返回 `None`（不参与外部手续费流程）。

### 6.3 手续费分账

按 `TransferFeeRouter`（复用 `OnchainFeeRouter` 规则）：
- 80% → 全节点出块者
- 10% → 国储会
- 10% → 安全基金账户

## 7. 转账执行逻辑

### 7.1 自动执行流程

`VotingEngine::internal_vote` 达到阈值后，投票引擎进入 `STATUS_PASSED` 并在同一事务内回调本模块自动执行：

```
1. 最后一票触发 voting-engine 的 STATUS_PASSED 判定
2. voting-engine 调用 InternalVoteExecutor::on_internal_vote_finalized(proposal_id, approved=true)
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

| 操作 | resolution-destro | duoqian-transfer |
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

区块 N+k: 快照管理员逐个提交 VotingEngine::internal_vote(X, approve=true)
           → STATUS_PASSED 达阈值
           → 同一交易内 callback 自动执行
           → emit TransferExecuted 或 TransferExecutionFailed
```

### 8.2 关键差异

- 投票完全复用 `voting-engine::internal_vote`，不再有业务 pallet 自己的 vote/finalize 状态机。
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
    frame_system::Config + voting_engine::Config + duoqian_manage::Config
{
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// 备注最大长度
    #[pallet::constant]
    type MaxRemarkLen: Get<u32>;

    /// 手续费分账路由（复用 OnchainFeeRouter）
    type FeeRouter: frame_support::traits::OnUnbalanced<
        <<Self as duoqian_manage::Config>::Currency as Currency<Self::AccountId>>::NegativeImbalance,
    >;

    /// Weight 配置
    type WeightInfo: crate::weights::WeightInfo;
}
```

说明：`Currency`、`InternalVoteEngine`、`ProtectedSourceChecker`、`InstitutionAsset` 等类型由上游 `duoqian_manage::Config` 和 `voting_engine::Config` 提供，本模块不再单独声明。

## 11. Weight 估算

| Extrinsic | 预估 Weight | DB 读 | DB 写 |
| --- | --- | --- | --- |
| `propose_transfer` | ~55 ms | 5 | 7 |
| `propose_safety_fund_transfer` | 待 benchmark | - | - |
| `propose_sweep_to_main` | 待 benchmark | - | - |

说明：投票权重由 `voting-engine::internal_vote` 承担；手动重试走 `VotingEngine::retry_passed_proposal`，权重由投票引擎统一计入。本模块 2026-05-02 起不再保留 `execute_xxx` wrapper。正式数值需重新跑 benchmark 生成。

## 12. 文件清单

| 文件 | 说明 |
| --- | --- |
| `src/lib.rs` | Pallet 主体（Config、Event、Error、Extrinsics、TransferAction） |
| `src/weights.rs` | Weight 定义（先用占位值，后续 benchmark 生成） |
| `src/benchmarks.rs` | 基准测试 |
| `Cargo.toml` | 依赖声明 |
| `DUOQIAN_TRANSFER_TECHNICAL.md` | 本技术文档 |

## 13. Runtime 集成要点

### 13.1 注册 pallet

在 `runtime/src/lib.rs` 中注册（pallet_index = 19）：
```rust
#[runtime::pallet_index(19)]
pub type DuoqianTransfer = duoqian_transfer;
```

在 `runtime/src/configs/mod.rs` 中配置：
```rust
impl duoqian_transfer::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxRemarkLen = ConstU32<256>;
    type InternalVoteEngine = VotingEngine;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type FeeRouter = TransferFeeRouter;
    type WeightInfo = duoqian_transfer::weights::SubstrateWeight<Runtime>;
}
```

### 13.2 CallAmount 配置

`DuoqianTransfer` 的所有 extrinsic 返回 `NoAmount`（免费提交），手续费在 pallet 内部扣取：
```rust
RuntimeCall::DuoqianTransfer(_) => {
    onchain_transaction::AmountExtractResult::NoAmount
}
```

### 13.3 Benchmark 注册

在 `define_benchmarks!` 中添加 `[duoqian_transfer, DuoqianTransfer]`。
