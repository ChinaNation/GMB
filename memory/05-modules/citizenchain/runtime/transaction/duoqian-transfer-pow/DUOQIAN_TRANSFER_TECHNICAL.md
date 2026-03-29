# DUOQIAN Transfer Pow 技术文档（机构多签名地址转账模块）

## 0. 功能需求

### 0.1 核心职责

`duoqian-transfer-pow` 负责机构多签名地址通过内部投票引擎发起转账：

- 机构管理员发起转账提案，指定收款地址、金额和备注。
- 机构管理员通过内部投票引擎逐票投票。
- 投票通过后执行转账：从机构 `duoqian_address` 向收款地址划转资金。
- 手续费在投票通过后由 pallet 内部从机构 `duoqian_address` 扣取，通过 `onchain-transaction-pow::calculate_onchain_fee()` 计算。
- 管理员个人账户不承担任何费用。
- 覆盖两类来源：
  - 创世预置的治理机构 `duoqian_address`（NRC / PRC / PRB）
  - `duoqian-manage-pow` 注册并激活的 `ORG_DUOQIAN` 多签地址

### 0.2 功能边界

- 本模块处理两类机构转账：
  - 创世预置的治理机构（NRC / PRC / PRB）
  - `duoqian-manage-pow` 注册并处于 Active 状态的多签机构（`ORG_DUOQIAN`）
- 当前也尚未接入新补充的内置机构 `ZF / LF / JC / JY / SF`。
- 本模块不负责投票引擎实现，投票逻辑委托给 `voting-engine-system` 的 `InternalVoteEngine`。

补充说明：
- 只要某类内置机构被本模块的 `institution_org()` / `institution_pallet_address()` 正式识别，
- 且对应管理员与阈值已接入 runtime 的 `RuntimeInternalAdminProvider / RuntimeInternalThresholdProvider`，
- 这类机构就可以直接复用本模块和内部投票引擎发起转账提案，不需要新增转账 pallet。

### 0.3 与 `duoqian-manage-pow` 的关系

| 模块 | 职责 | 地址类型 | 审批方式 |
| --- | --- | --- | --- |
| `duoqian-manage-pow` | 多签名地址的注册、创建、关闭 | 注册的非治理机构 | `sfid` 主签名登记 + `ORG_DUOQIAN` 内部投票 |
| `duoqian-transfer-pow` | 多签名地址转账 | 预置治理机构 + 注册型 Active 多签机构 | 链上内部投票引擎（逐票投票） |

### 0.4 与 `resolution-destro-gov` 的关系

两者结构高度一致，区别在于资金操作：

| 对比 | `resolution-destro-gov` | `duoqian-transfer-pow` |
| --- | --- | --- |
| 资金操作 | `Currency::slash()` 销毁 | `Currency::transfer()` 转账 |
| 目标 | 销毁机构持有的代币 | 转账到指定收款地址 |
| 额外字段 | 无 | `beneficiary`、`remark` |

## 1. 地址说明

### 1.1 关键区分

| 地址 | 类型 | 说明 |
| --- | --- | --- |
| `keyless_address` | 质押地址 | **不允许支出**，仅用于质押 |
| `duoqian_address` | 多签名地址 | 机构资金账户，转账和手续费均从此扣取 |

### 1.2 duoqian_address 来源

`duoqian_address` 现有两种来源：

- 治理机构：预置于 `runtime/primitives/china/china_cb.rs`（NRC + PRC）和 `runtime/primitives/china/china_ch.rs`（PRB）中，通过 `institution_pallet_address(institution_id)` 查找。
- 注册型机构：`InstitutionPalletId(48)` 采用 `duoqian_address(32) + 16 字节 0` 编码，再从 `duoqian-manage-pow::DuoqianAccounts` 读取 Active 账户。

### 1.3 institution-asset-guard 边界

- 本模块在 `propose_transfer` 和 `try_execute_transfer` 两个阶段都会调用 `institution-asset-guard`。
- 当前 runtime 规则下，制度保留 `duoqian_address` 只允许本模块这类治理执行动作内部扣款。
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
   - 注册型机构：能解码出 `duoqian_address`，且在 `DuoqianAccounts` 中存在并处于 Active。
4. `org` 必须与 `institution` 的实际机构类型匹配。
5. `proposer` 必须是该机构的当前管理员（通过 `InternalAdminProvider::is_internal_admin` 校验）。
6. `amount >= ED`（转账金额不能低于存在性保证金，防止收款地址创建失败）。
7. `beneficiary` 不能是机构自身的 `duoqian_address`（不允许自转账）。
8. `beneficiary` 不能是受保护地址（如 `keyless_address`、黑洞地址）。
9. 机构 `duoqian_address` 的可用余额 >= `amount + fee + ED`（预检含手续费，防止创建必定无法执行的提案）。
10. 活跃提案数由 `voting-engine-system` 在 `create_internal_proposal` 中统一检查（全局限额）。

**执行逻辑：**

1. 调用 `InternalVoteEngine::create_internal_proposal(proposer, org, institution)` 获取 `proposal_id`。
2. 编码 `TransferAction { institution, beneficiary, amount, remark, proposer }` 存入 `voting_engine_system::ProposalData`。
3. 记录 `ProposalMeta`（创建块号）到 `voting_engine_system`。
4. 发出 `TransferProposed` 事件。

### 2.2 vote_transfer — 投票

```rust
pub fn vote_transfer(
    origin: OriginFor<T>,
    proposal_id: u64,
    approve: bool,
) -> DispatchResult
```

**校验规则：**

1. `origin` 必须是 `signed`。
2. `proposal_id` 必须在 `voting_engine_system::ProposalData` 中存在且能解码为 `TransferAction`。
3. 投票者必须是该机构管理员。
4. 每个管理员每个提案只能投一票（`InternalVoteEngine` 内部保证）。

**执行逻辑：**

1. 调用 `InternalVoteEngine::cast_internal_vote(who, proposal_id, approve)`。
2. 发出 `TransferVoteSubmitted { proposal_id, who, approve }` 事件。
3. 读取投票引擎中的提案状态：
   - 如果 `STATUS_PASSED`（赞成票 >= 阈值）：**尝试自动执行转账**。
     - 成功：发出 `TransferExecuted` 事件。
     - 失败：发出 `TransferExecutionFailed` 事件（投票已记录，提案状态为 PASSED，可通过 `execute_transfer` 手动重试）。

### 2.3 execute_transfer — 手动执行已通过的转账提案

```rust
pub fn execute_transfer(
    origin: OriginFor<T>,
    proposal_id: u64,
) -> DispatchResult
```

**用途：** 当投票通过后自动执行失败（如余额不足），可在补充余额后通过此接口重试。

**校验规则：**

1. `origin` 必须是 `signed`（任何签名账户，不限管理员）。
2. `proposal_id` 必须存在。
3. 提案状态必须为 `STATUS_PASSED`。
4. 提案数据（`TransferAction`）必须存在且可解码。

**执行逻辑：** 与 `vote_transfer` 中自动执行的逻辑完全一致（共用 `try_execute_transfer` 内部方法）。执行成功后提案状态变为 `STATUS_EXECUTED`，防止双重执行。

**设计说明：** 任何签名账户都可调用（不限管理员），避免因管理员离线导致已通过的提案无法落地。与 `resolution-destro-gov::execute_destroy` 保持一致。

## 3. 存储项

本模块**自身不定义存储项**。所有提案数据统一存储在 `voting-engine-system` 中：

| 存储位置 | Key | Value | 说明 |
| --- | --- | --- | --- |
| `voting_engine_system::ProposalData` | `u64` | `Vec<u8>`（编码的 `TransferAction`） | 提案业务数据 |
| `voting_engine_system::ProposalMeta` | `u64` | `ProposalMetadata` | 提案元数据（创建块号等） |
| `voting_engine_system::Proposals` | `u64` | `Proposal` | 提案核心状态（status、timing） |

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
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
    },
    /// 投票已提交
    TransferVoteSubmitted {
        proposal_id: u64,
        who: T::AccountId,
        approve: bool,
    },
    /// 投票通过但执行失败（投票已记录，提案已标记 PASSED，可通过 execute_transfer 手动重试）
    TransferExecutionFailed {
        proposal_id: u64,
        institution: InstitutionPalletId,
    },
    /// 转账已执行（投票通过后自动触发，含手续费分账）
    TransferExecuted {
        proposal_id: u64,
        institution: InstitutionPalletId,
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        fee: BalanceOf<T>,
    },
}
```

## 5. 错误码

```rust
#[pallet::error]
pub enum Error<T> {
    InvalidInstitution,              // 机构不存在
    InstitutionOrgMismatch,          // org 与机构类型不匹配
    UnauthorizedAdmin,               // 非该机构管理员
    ZeroAmount,                      // 金额为零
    AmountBelowExistentialDeposit,   // 金额低于 ED
    SelfTransferNotAllowed,          // 不能转给自己
    BeneficiaryIsProtectedAddress,   // 收款地址是受保护地址
    ProposalActionNotFound,          // 提案不存在或数据解码失败
    InstitutionAccountDecodeFailed,  // 机构地址解码失败
    InsufficientBalance,             // 余额不足（amount + fee + ED）
    ProposalNotPassed,               // 提案未通过（execute_transfer 校验）
    TransferFailed,                  // 转账执行失败
}
```

## 6. 手续费机制

### 6.1 计费规则

由 `onchain-transaction-pow::calculate_onchain_fee()` 计算：

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

`RuntimeFeePayerExtractor` 对 `DuoqianTransferPow` 返回 `None`（不参与外部手续费流程）。

### 6.3 手续费分账

按 `TransferFeeRouter`（复用 `PowOnchainFeeRouter` 规则）：
- 80% → 全节点出块者
- 10% → 国储会
- 10% → 黑洞销毁

## 7. 转账执行逻辑

### 7.1 自动执行流程

投票达到阈值后，在最后一票的 `vote_transfer` 交易内**尝试自动执行转账**：

```
1. cast_internal_vote() 返回后，读取提案状态
2. 若 STATUS_PASSED（yes >= threshold）：
   a. 从 voting_engine_system::ProposalData 读取编码的 TransferAction
   b. 通过 institution_pallet_address(institution) 获取 duoqian_address [u8; 32]
   c. 解码为 T::AccountId
   d. 计算手续费 fee = calculate_onchain_fee(amount)
   e. 校验余额: free_balance >= amount + fee + ED
   f. 执行 Currency::transfer(duoqian_address, beneficiary, amount, KeepAlive)
   g. 执行 Currency::withdraw(duoqian_address, fee) 扣取手续费
   h. 通过 FeeRouter 分账
   i. 调用 set_status_and_emit(STATUS_EXECUTED) 标记为已执行（防止双重执行）
   j. 发出 TransferExecuted 事件
3. 若执行失败：发出 TransferExecutionFailed 事件（状态保持 PASSED，可通过 execute_transfer 手动重试）
```

### 7.2 提案状态流转

```
VOTING → PASSED（待执行） → EXECUTED（已执行，终态）
                ↓ 执行失败
           保持 PASSED（可通过 execute_transfer 重试）
```

- `VOTING`（0）：投票进行中
- `PASSED`（1）：投票通过，待执行或执行失败待重试
- `REJECTED`（2）：投票超时未达阈值
- `EXECUTED`（3）：执行成功，终态，无法再次执行

### 7.3 余额保护

- 使用 `ExistenceRequirement::KeepAlive` 确保转账后机构账户不被 reap（删除）。
- 执行时校验 `free_balance >= amount + fee + ED`。

### 7.4 转账 vs 销毁

| 操作 | resolution-destro-gov | duoqian-transfer-pow |
| --- | --- | --- |
| API | `Currency::slash()` | `Currency::transfer()` |
| 总发行量 | 减少（资金销毁） | 不变（资金转移） |
| 目标 | 无（资金消失） | `beneficiary` 账户 |

## 8. 提案与投票的区块写入

### 8.1 每个操作独立写入不同区块

```
区块 N  : 管理员A 发起 propose_transfer()      → 创建提案 proposal_id=X
区块 N+1: 管理员B 调用 vote_transfer(X, true)  → 第1票
区块 N+2: 管理员C 调用 vote_transfer(X, true)  → 第2票
...
区块 N+K: 管理员M 调用 vote_transfer(X, true)  → 达到阈值 → 自动执行转账（同一交易内）
```

### 8.2 原因

- 每位管理员**独立签名提交交易**，物理上不可能同时在同一区块。
- `InternalVoteEngine::cast_internal_vote()` 每次处理一票。
- 防重复：`InternalVotesByAccount` 保证每人每提案只投一票。
- 当 `yes >= threshold` 时，最后一票的投票交易内自动执行转账，无需额外操作。

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
    frame_system::Config + voting_engine_system::Config + duoqian_manage_pow::Config
{
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// 备注最大长度
    #[pallet::constant]
    type MaxRemarkLen: Get<u32>;

    /// 手续费分账路由（复用 PowOnchainFeeRouter）
    type FeeRouter: frame_support::traits::OnUnbalanced<
        <<Self as duoqian_manage_pow::Config>::Currency as Currency<Self::AccountId>>::NegativeImbalance,
    >;

    /// Weight 配置
    type WeightInfo: crate::weights::WeightInfo;
}
```

说明：`Currency`、`InternalVoteEngine`、`ProtectedSourceChecker`、`InstitutionAssetGuard` 等类型由上游 `duoqian_manage_pow::Config` 和 `voting_engine_system::Config` 提供，本模块不再单独声明。

## 11. Weight 估算

| Extrinsic | 预估 Weight | DB 读 | DB 写 |
| --- | --- | --- | --- |
| `propose_transfer` | ~55 ms | 5 | 7 |
| `vote_transfer`（含自动执行） | ~140 ms | 9 | 12 |
| `execute_transfer`（手动重试） | ~75 ms | 4 | 4 |

说明：以上为参考 `resolution-destro-gov` 的基准估算，实际值需跑 benchmark 生成。
`vote_transfer` 在最后一票触发自动转账时 DB 写入最多（投票记录 + 转账 + 手续费扣取）。
`execute_transfer` 仅读取提案数据并执行转账，不涉及投票逻辑。

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
pub type DuoqianTransferPow = duoqian_transfer_pow;
```

在 `runtime/src/configs/mod.rs` 中配置：
```rust
impl duoqian_transfer_pow::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxRemarkLen = ConstU32<256>;
    type InternalVoteEngine = VotingEngineSystem;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type FeeRouter = TransferFeeRouter;
    type WeightInfo = duoqian_transfer_pow::weights::SubstrateWeight<Runtime>;
}
```

### 13.2 CallAmount 配置

`DuoqianTransferPow` 的所有 extrinsic 返回 `NoAmount`（免费提交），手续费在 pallet 内部扣取：
```rust
RuntimeCall::DuoqianTransferPow(_) => {
    onchain_transaction_pow::AmountExtractResult::NoAmount
}
```

### 13.3 Benchmark 注册

在 `define_benchmarks!` 中添加 `[duoqian_transfer_pow, DuoqianTransferPow]`。
