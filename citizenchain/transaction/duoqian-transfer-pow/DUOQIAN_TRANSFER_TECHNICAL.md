# DUOQIAN Transfer Pow 技术文档（机构多签名地址转账模块）

## 0. 功能需求

### 0.1 核心职责

`duoqian-transfer-pow` 负责治理机构（国储会/省储会/省储行）通过内部投票引擎发起多签名地址转账：

- 机构管理员发起转账提案，指定收款地址、金额和备注。
- 机构管理员通过内部投票引擎逐票投票。
- 投票通过后执行转账：从机构 `duoqian_address` 向收款地址划转资金。
- 手续费从机构 `duoqian_address` 扣取，由 `onchain-transaction-pow` 的自定义计费规则计算。
- 管理员个人账户不承担任何费用。

### 0.2 功能边界

- 本模块仅处理治理机构（已在链上预置 `duoqian_address` 的 NRC/PRC/PRB 机构）的转账。
- 非治理机构（通过 `duoqian-transaction-pow` 注册的多签地址）不在本模块范围。
- 本模块不负责手续费计算，手续费由 `onchain-transaction-pow` 的 `PowOnchainChargeAdapter` 统一处理。
- 本模块不负责投票引擎实现，投票逻辑委托给 `voting-engine-system` 的 `InternalVoteEngine`。

### 0.3 与 `duoqian-transaction-pow` 的关系

| 模块 | 职责 | 地址类型 | 审批方式 |
| --- | --- | --- | --- |
| `duoqian-transaction-pow` | 多签名地址的注册、创建、关闭 | 注册的非治理机构 | 离线 M-of-N 签名一次性提交 |
| `duoqian-transfer-pow` | 治理机构多签名地址转账 | 预置的治理机构 | 链上内部投票引擎（逐票投票） |

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

治理机构的 `duoqian_address` 预置于 `primitives/china/china_cb.rs`（NRC + PRC）和 `primitives/china/china_ch.rs`（PRB）中，通过 `institution_pallet_address(institution_id)` 查找。

## 2. Extrinsic 接口

### 2.1 propose_transfer — 发起转账提案

```rust
pub fn propose_transfer(
    origin: OriginFor<T>,
    org: u8,                           // 机构类型：0=NRC, 1=PRC, 2=PRB
    institution: InstitutionPalletId,   // 机构 pallet id [u8; 48]
    beneficiary: T::AccountId,          // 收款地址
    amount: BalanceOf<T>,               // 转账金额
    remark: BoundedVec<u8, T::MaxRemarkLen>, // 备注
) -> DispatchResult
```

**校验规则：**

1. `origin` 必须是 `signed`，提取 `proposer = ensure_signed(origin)`。
2. `institution` 必须是有效的治理机构（在 CHINA_CB 或 CHINA_CH 中存在）。
3. `org` 必须与 `institution` 的实际机构类型匹配。
4. `proposer` 必须是该机构的当前管理员（通过 `InternalAdminProvider::is_internal_admin` 校验）。
5. `amount > 0`。
6. `beneficiary` 不能是机构自身的 `duoqian_address`（不允许自转账）。
7. `beneficiary` 不能是受保护地址（如 `keyless_address`、黑洞地址）。
8. 该机构当前不能有活跃的转账提案（一机构一提案）。
9. 机构 `duoqian_address` 的可用余额 >= `amount + ED`（预检，防止创建必定无法执行的提案）。

**执行逻辑：**

1. 调用 `InternalVoteEngine::create_internal_proposal(proposer, org, institution)` 获取 `proposal_id`。
2. 存储 `TransferAction { institution, beneficiary, amount, remark, proposer }` → `ProposalActions`。
3. 记录 `ProposalCreatedAt`。
4. 设置 `ActiveProposalByInstitution`。
5. 发出 `TransferProposed` 事件。

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
2. `proposal_id` 必须存在且有关联的 `TransferAction`。
3. 投票者必须是该机构管理员。
4. 每个管理员每个提案只能投一票（`InternalVoteEngine` 内部保证）。

**执行逻辑：**

1. 调用 `InternalVoteEngine::cast_internal_vote(who, proposal_id, approve)`。
2. 发出 `TransferVoteSubmitted { proposal_id, who, approve }` 事件。
3. 读取投票引擎中的提案状态：
   - 如果 `STATUS_PASSED`（赞成票 >= 阈值）：**立即自动执行转账**，无需手动触发。
     - 执行 `Currency::transfer(duoqian_address, beneficiary, amount, KeepAlive)`。
     - 清理所有关联存储。
     - 发出 `TransferExecuted` 事件。
   - 如果 `STATUS_REJECTED`（投票超时未达阈值）：清理所有关联存储。

## 3. 存储项

| 存储项 | Key | Value | 说明 |
| --- | --- | --- | --- |
| `ProposalActions<T>` | `u64` | `TransferAction` | 提案对应的转账动作 |
| `ProposalCreatedAt<T>` | `u64` | `BlockNumber` | 提案创建块号 |
| `ActiveProposalByInstitution<T>` | `InstitutionPalletId` | `u64` | 每机构仅一个活跃提案 |

### 3.1 TransferAction 结构

```rust
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TransferAction<AccountId, Balance, Remark> {
    pub institution: InstitutionPalletId,  // 转出机构
    pub beneficiary: AccountId,             // 收款地址
    pub amount: Balance,                    // 转账金额
    pub remark: Remark,                     // 备注
    pub proposer: AccountId,                // 发起管理员
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
    /// 转账已执行（投票通过后自动触发）
    TransferExecuted {
        proposal_id: u64,
        institution: InstitutionPalletId,
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
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
    SelfTransferNotAllowed,          // 不能转给自己
    BeneficiaryIsProtectedAddress,   // 收款地址是受保护地址
    ProposalActionNotFound,          // 提案不存在
    InstitutionAccountDecodeFailed,  // 机构地址解码失败
    InsufficientBalance,             // 余额不足（amount + ED）
    ActiveProposalExists,            // 该机构已有活跃提案
    TransferFailed,                  // 转账执行失败
}
```

## 6. 手续费机制

### 6.1 计费规则

由 `onchain-transaction-pow` 的 `custom_fee_with_tip()` 统一计算：

- 基础手续费 = `max(amount × ONCHAIN_FEE_RATE, ONCHAIN_MIN_FEE)`
- `ONCHAIN_FEE_RATE` = 0.1%（`Perbill::from_parts(1_000_000)`）
- `ONCHAIN_MIN_FEE` = 10 分 = 0.1 元
- 按"分"四舍五入

### 6.2 手续费支付者

通过 `RuntimeFeePayerExtractor`（实现 `CallFeePayer` trait）将手续费支付者从签名管理员切换到机构 `duoqian_address`：

```rust
// runtime/src/configs/mod.rs 中扩展 RuntimeFeePayerExtractor
impl CallFeePayer<AccountId, RuntimeCall> for RuntimeFeePayerExtractor {
    fn fee_payer(_who: &AccountId, call: &RuntimeCall) -> Option<AccountId> {
        match call {
            RuntimeCall::DuoqianTransferPow(
                duoqian_transfer_pow::Call::propose_transfer { institution, .. }
            ) => institution_pallet_address(*institution)
                .and_then(|raw| AccountId::decode(&mut &raw[..]).ok()),
            RuntimeCall::DuoqianTransferPow(
                duoqian_transfer_pow::Call::vote_transfer { proposal_id, .. }
            ) => {
                // 从 ProposalActions 中获取 institution，再查 duoqian_address
                duoqian_transfer_pow::ProposalActions::<Runtime>::get(proposal_id)
                    .and_then(|action| institution_pallet_address(action.institution))
                    .and_then(|raw| AccountId::decode(&mut &raw[..]).ok())
            }
            _ => None,
        }
    }
}
```

### 6.3 手续费分账

按 `PowOnchainFeeRouter` 规则：
- 80% → 全节点出块者
- 10% → 国储会
- 10% → 黑洞销毁

### 6.4 手续费金额提取

需要在 runtime 的 `CallAmount` 实现中为 `DuoqianTransferPow` 的转账交易返回 `Amount(transfer_amount)`，使计费规则能正确按转账金额计算费率。

## 7. 转账执行逻辑

### 7.1 自动执行流程

投票达到阈值后，在最后一票的 `vote_transfer` 交易内**立即自动执行转账**：

```
1. cast_internal_vote() 返回后，读取提案状态
2. 若 STATUS_PASSED（yes >= threshold）：
   a. 从 ProposalActions 读取 TransferAction
   b. 通过 institution_pallet_address(institution) 获取 duoqian_address [u8; 32]
   c. 解码为 T::AccountId
   d. 校验余额: free_balance >= amount + ED
   e. 执行 Currency::transfer(duoqian_address, beneficiary, amount, KeepAlive)
   f. 清理关联存储: ProposalActions, ProposalCreatedAt, ActiveProposalByInstitution
   g. 调用 InternalVoteEngine::cleanup_internal_proposal(proposal_id)
   h. 发出 TransferExecuted 事件
```

没有手动执行入口，投票通过即转账，一步到位。

### 7.2 余额保护

- 使用 `ExistenceRequirement::KeepAlive` 确保转账后机构账户不被 reap（删除）。
- 预检 `free_balance >= amount + ED`，其中 `ED = 111 分`（`ACCOUNT_EXISTENTIAL_DEPOSIT`）。

### 7.3 转账 vs 销毁

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
pub trait Config: frame_system::Config + voting_engine_system::Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// 货币类型，用于余额查询和转账执行
    type Currency: Currency<Self::AccountId>;

    /// 备注最大长度
    #[pallet::constant]
    type MaxRemarkLen: Get<u32>;

    /// 内部投票引擎
    type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

    /// 受保护地址检查器
    type ProtectedAddressChecker: ProtectedAddressCheck<Self::AccountId>;

    /// Weight 配置
    type WeightInfo: WeightInfo;
}
```

## 11. Weight 估算

| Extrinsic | 预估 Weight | DB 读 | DB 写 |
| --- | --- | --- | --- |
| `propose_transfer` | ~55 ms | 5 | 7 |
| `vote_transfer`（含自动执行） | ~140 ms | 9 | 12 |

说明：以上为参考 `resolution-destro-gov` 的基准估算，实际值需跑 benchmark 生成。
`vote_transfer` 在最后一票触发自动转账时 DB 写入最多（投票记录 + 转账 + 清理）。

## 12. 文件清单

| 文件 | 说明 |
| --- | --- |
| `src/lib.rs` | Pallet 主体（Config、Storage、Event、Error、Extrinsics） |
| `src/weights.rs` | Weight 定义（先用占位值，后续 benchmark 生成） |
| `src/benchmarks.rs` | 基准测试 |
| `Cargo.toml` | 依赖声明 |
| `DUOQIAN_TRANSFER_TECHNICAL.md` | 本技术文档 |

## 13. Runtime 集成要点

### 13.1 注册 pallet

在 `runtime/src/configs/mod.rs` 中：
- 配置 `duoqian_transfer_pow::Config`
- 在 `construct_runtime!` 中注册 `DuoqianTransferPow`

### 13.2 扩展 RuntimeFeePayerExtractor

为 `propose_transfer` 和 `vote_transfer` 返回机构 `duoqian_address` 作为手续费支付者。

### 13.3 扩展 CallAmount

为 `DuoqianTransferPow` 的转账交易返回 `Amount(transfer_amount)`，使计费规则按转账金额计算。

### 13.4 Benchmark 注册

在 `define_benchmarks!` 中添加 `[duoqian_transfer_pow, DuoqianTransferPow]`。
