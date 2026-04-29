# DUOQIAN Transfer Pow 技术文档（机构多签名地址转账模块）

## Step 2 · 离线 QR 聚合改造(2026-04-21)

本模块所有 3 组业务(transfer / safety_fund / sweep)已从"每管理员在线 `vote_X`"
模式统一改造为"发起人离线收齐 N 个管理员 sr25519 签名 → 一笔 `finalize_X` 代投"
的离线 QR 聚合签名模式。投票引擎零改动。

### 入口对照

| call_index | 原 extrinsic | 新 extrinsic | op_tag |
|---|---|---|---|
| 0 | propose_transfer | propose_transfer(不变) | — |
| 1 | ~~vote_transfer~~ | **finalize_transfer** | `OP_SIGN_TRANSFER = 0x15` |
| 2 | execute_transfer | execute_transfer(保留,手动重试) | — |
| 3 | propose_safety_fund_transfer | propose_safety_fund_transfer(不变) | — |
| 4 | ~~vote_safety_fund_transfer~~ | **finalize_safety_fund_transfer** | `OP_SIGN_SAFETY_FUND = 0x16` |
| 5 | propose_sweep_to_main | propose_sweep_to_main(不变) | — |
| 6 | ~~vote_sweep_to_main~~ | **finalize_sweep_to_main** | `OP_SIGN_SWEEP = 0x17` |

### 统一签名 Intent

三组共用一个 `TransferVoteIntent` SCALE 结构:

```rust
pub struct TransferVoteIntent<AccountId, Balance> {
    pub proposal_id: u64,
    pub org: u8,
    pub institution: InstitutionPalletId,
    pub from: AccountId,         // 资金源(主账户 / NRC_ANQUAN / 费用账户)
    pub to: AccountId,           // 资金目标(beneficiary / 主账户)
    pub amount: Balance,
    pub remark_hash: [u8; 32],   // blake2_256(remark);sweep 用 blake2_256(b"")
    pub proposer: AccountId,
    pub approve: bool,           // 恒 true,占位防误签
}
```

`signing_hash(ss58_prefix, op_tag)` 计算公式:

```
preimage = DUOQIAN_DOMAIN(10B) || op_tag(1B) || ss58_prefix_le(2B) || blake2_256(intent.encode())
signing_hash = blake2_256(preimage)
sig = sr25519_sign(admin_key, signing_hash)
```

三组 op_tag 不同 → signing_hash 不同 → **跨业务签名重放自动失败**。

### 三业务字段映射

| | op_tag | from | to | remark_hash 来源 |
|---|---|---|---|---|
| transfer | `OP_SIGN_TRANSFER(0x15)` | `institution_pallet_address(institution)` | `action.beneficiary` | `blake2_256(action.remark)` |
| safety_fund | `OP_SIGN_SAFETY_FUND(0x16)` | `NRC_ANQUAN_ADDRESS` 常量 | `action.beneficiary` | `blake2_256(action.remark)` |
| sweep | `OP_SIGN_SWEEP(0x17)` | `resolve_fee_account(institution)` | `resolve_main_account(institution)` | `blake2_256(b"")` |

### 共用 helper

`Pallet::<T>::verify_and_cast_votes(proposal_id, org, institution, threshold, signing_hash, sigs)`
是三个 `finalize_X` 共用的签名验证 + 代投循环。任一签名失败(成员校验 / 去重 /
长度 / 验签 / 代投)都让整笔交易回滚,不接受部分签名。

### Tx 1 event 字段补齐(供 wuminapp 生成 QR)

- `TransferProposed` 增加 `from / remark / expires_at`
- `SafetyFundTransferProposed` 增加 `from / remark / expires_at`
- `SweepToMainProposed` 增加 `from / to / expires_at`

### 新增错误码

- `UnauthorizedSignature`:sig 对应 admin 非该机构管理员
- `DuplicateSignature`:同批次 admin 重复
- `InvalidSignature`:sr25519 验签失败
- `InsufficientSignatures`:签名数 < threshold
- `MalformedSignature`:sig 长度非 64 字节

### 新增事件

每组 `finalize_X` 调用结束都发 `*Finalized { proposal_id, signatures_accepted, final_status }`
事件,便于链下追踪"N 签提交 → 投票引擎状态"的一一对应。

### SweepAction 结构变更

`SweepAction` 新增 `proposer: AccountId` 字段,与 transfer / safety_fund 保持结构一致,
用于 `TransferVoteIntent.proposer` 填充。

## 0. 功能需求

### 0.1 核心职责

`duoqian-transfer-pow` 负责机构多签名地址通过内部投票引擎发起转账：

- 机构管理员发起转账提案，指定收款地址、金额和备注。
- 机构管理员通过内部投票引擎逐票投票。
- 投票通过后执行转账：从机构主账户地址向收款地址划转资金。
- 手续费在投票通过后由 pallet 内部从机构主账户扣取，通过 `onchain-transaction-pow::calculate_onchain_fee()` 计算。
- 管理员个人账户不承担任何费用。
- 覆盖两类来源：
  - 创世预置的治理机构 `main_address`（NRC / PRC / PRB）
  - `duoqian-manage-pow` 注册并激活的 `ORG_DUOQIAN` 多签地址（`action.duoqian_address`）

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
| `stake_address` | 质押地址 | **不允许支出**，仅用于质押 |
| `main_address`（治理机构）/ `duoqian_address`（注册多签） | 机构主账户/注册多签主账户 | 机构资金账户，转账和手续费均从此扣取 |

### 1.2 机构主账户地址来源

机构主账户地址有两种来源：

- 治理机构：`main_address` 预置于 `runtime/primitives/china/china_cb.rs`（NRC + PRC）和 `runtime/primitives/china/china_ch.rs`（PRB）中，通过 `institution_pallet_address(institution_id)` 查找。
- 注册型机构：`InstitutionPalletId(48)` 采用主账户地址 `AccountId(32) + 16 字节 0` 编码，资金账户仍从 `duoqian-manage-pow::DuoqianAccounts` 校验 Active，管理员、阈值和人数统一从 `admins-origin-gov::Institutions` 读取。

### 1.3 institution-asset-guard 边界

- 本模块在 `propose_transfer` 和 `try_execute_transfer` 两个阶段都会调用 `institution-asset-guard`。
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
5. `proposer` 必须是该机构的当前管理员（通过 `InternalAdminProvider::is_internal_admin` 校验，生产 runtime 最终读取 `admins-origin-gov::Institutions`）。
6. `amount >= ED`（转账金额不能低于存在性保证金，防止收款地址创建失败）。
7. `beneficiary` 不能是机构自身的主账户地址（不允许自转账）。
8. `beneficiary` 不能是受保护地址（如 `stake_address`、安全基金账户、费用账户等保留地址）。
9. 机构主账户的可用余额 >= `amount + fee + ED`（预检含手续费，防止创建必定无法执行的提案）。
10. 活跃提案数由 `voting-engine-system` 在 `create_internal_proposal` 中统一检查（全局限额）。

**执行逻辑：**

1. 调用 `InternalVoteEngine::create_internal_proposal(proposer, org, institution)` 获取 `proposal_id`。
2. 编码 `TransferAction { institution, beneficiary, amount, remark, proposer }` 存入 `voting_engine_system::ProposalData`。
3. 记录 `ProposalMeta`（创建块号）到 `voting_engine_system`。
4. 发出 `TransferProposed` 事件。

### 2.2 finalize_transfer — 离线聚合代投(Step 2 替换 vote_transfer)

```rust
pub fn finalize_transfer(
    origin: OriginFor<T>,
    proposal_id: u64,
    sigs: BoundedVec<
        (T::AccountId, duoqian_manage_pow::pallet::AdminSignatureOf<T>),
        <T as duoqian_manage_pow::Config>::MaxAdmins,
    >,
) -> DispatchResult
```

**校验规则：**

1. `origin` 必须是 `signed`(任何账户,发起人不必是管理员 — Tx 1 已锁定 `proposer`)。
2. `proposal_id` 必须在 `voting_engine_system::ProposalData` 中存在且能解码为 `TransferAction`。
3. 聚合签名数量 >= 该机构阈值,否则 `InsufficientSignatures`。
4. `sigs` 中每个 `admin` 必须是该机构当前管理员(`UnauthorizedSignature`),且同批次不重复(`DuplicateSignature`)。
5. 每条 `sig_bytes.len() == 64`(`MalformedSignature`),且对 `TransferVoteIntent.signing_hash(ss58, OP_SIGN_TRANSFER)` 的 sr25519 验签通过(`InvalidSignature`)。

**执行逻辑：**

1. 解析 `(org, from = institution_pallet_address(institution))`。
2. 读阈值 `T::InternalThresholdProvider::pass_threshold(org, institution)`。
3. 构造 `TransferVoteIntent { proposal_id, org, institution, from, to: action.beneficiary, amount, remark_hash: blake2_256(action.remark), proposer: action.proposer, approve: true }`。
4. 计算 `signing_hash = intent.signing_hash(ss58_prefix, OP_SIGN_TRANSFER)`。
5. 调 `verify_and_cast_votes(...)`:逐条验签 + `cast_internal_vote(admin, proposal_id, true)`。
6. 读 `proposal.status`:
   - `STATUS_PASSED` → `with_transaction(|| try_execute_transfer(proposal_id))`;失败发 `TransferExecutionFailed`,提案保留供 `execute_transfer` 重试。
7. 发 `TransferFinalized { proposal_id, signatures_accepted, final_status }` 事件。

**幂等保护**:`cast_internal_vote` 内部 `AlreadyVoted` 检查会让同一 proposal 第二次 finalize 整笔回滚,不会重复入金。

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

## 4. 事件(Step 2 已更新)

```rust
#[pallet::event]
pub enum Event<T: Config> {
    /// 转账提案已创建(Tx 1,wuminapp 扫描此事件构造 QR)
    TransferProposed {
        proposal_id: u64,
        org: u8,
        institution: InstitutionPalletId,
        proposer: T::AccountId,
        from: T::AccountId,                             // Step 2 新增(= 机构主账户)
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        remark: BoundedVec<u8, T::MaxRemarkLen>,        // Step 2 新增(供 QR)
        expires_at: BlockNumberFor<T>,                  // Step 2 新增(超时区块)
    },
    /// 投票通过但执行失败(可通过 execute_transfer 手动重试)
    TransferExecutionFailed { proposal_id: u64, institution: InstitutionPalletId },
    /// 转账已执行(含手续费分账)
    TransferExecuted {
        proposal_id: u64,
        institution: InstitutionPalletId,
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        fee: BalanceOf<T>,
    },
    /// Step 2 新增:finalize_transfer 代投完成
    TransferFinalized { proposal_id: u64, signatures_accepted: u32, final_status: u8 },

    // 安全基金组:结构同上
    SafetyFundTransferProposed {
        proposal_id: u64,
        proposer: T::AccountId,
        from: T::AccountId,                             // Step 2 新增(= NRC_ANQUAN_ADDRESS)
        beneficiary: T::AccountId,
        amount: BalanceOf<T>,
        remark: BoundedVec<u8, T::MaxRemarkLen>,        // Step 2 新增
        expires_at: BlockNumberFor<T>,                  // Step 2 新增
    },
    SafetyFundTransferExecuted { proposal_id: u64, beneficiary: T::AccountId, amount: BalanceOf<T>, fee: BalanceOf<T> },
    SafetyFundExecutionFailed { proposal_id: u64 },
    SafetyFundFinalized { proposal_id: u64, signatures_accepted: u32, final_status: u8 },

    // Sweep 组:
    SweepToMainProposed {
        proposal_id: u64,
        institution: InstitutionPalletId,
        proposer: T::AccountId,
        from: T::AccountId,                             // Step 2 新增(= fee_account)
        to: T::AccountId,                               // Step 2 新增(= main_account)
        amount: BalanceOf<T>,
        expires_at: BlockNumberFor<T>,                  // Step 2 新增
    },
    SweepToMainExecuted { proposal_id: u64, institution: InstitutionPalletId, amount: BalanceOf<T>, fee: BalanceOf<T>, reserve_left: BalanceOf<T> },
    SweepExecutionFailed { proposal_id: u64 },
    SweepToMainFinalized { proposal_id: u64, signatures_accepted: u32, final_status: u8 },
}
```

**Step 2 已删除**:`TransferVoteSubmitted` / `SafetyFundVoteSubmitted` / `SweepToMainVoteSubmitted`(三个投票事件与 `vote_X` extrinsic 一起删除,统一由 `*Finalized` 替代)。

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
    ProposalNotPassed,               // 提案未通过(execute_transfer 校验)
    TransferFailed,                  // 转账执行失败
    // safety_fund / sweep 专有
    SafetyFundProposalNotFound, SafetyFundInsufficientBalance, SafetyFundProposalNotPassed,
    SweepProposalNotFound, InvalidSweepAmount, InsufficientFeeReserve, SweepAmountExceedsCap, SweepProposalNotPassed,
    // Step 2 · 离线聚合签名改造新增 5 个
    UnauthorizedSignature,           // sig 对应 admin 非该机构管理员
    DuplicateSignature,              // 同批次 admin 重复
    InvalidSignature,                // sr25519 验签失败
    InsufficientSignatures,          // 签名数 < 阈值
    MalformedSignature,              // sig 长度非 64 字节
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
- 10% → 安全基金账户

## 7. 转账执行逻辑(Step 2 · 离线聚合版)

### 7.1 自动执行流程

`finalize_transfer` 代投结束后,若投票引擎进入 `STATUS_PASSED`,在同一交易内
**尝试自动执行转账**:

```
1. verify_and_cast_votes() 循环结束,最后一次 cast_internal_vote() 触发 STATUS_PASSED
2. 读取 proposal.status:
3. 若 STATUS_PASSED(yes >= threshold):
   a. 从 voting_engine_system::ProposalData 读取 TransferAction
   b. institution_pallet_address(institution) 获取 from 地址
   c. 计算手续费 fee = calculate_onchain_fee(amount)
   d. 校验余额: free_balance >= amount + fee + ED
   e. with_transaction(|| {
        Currency::transfer(from, beneficiary, amount, KeepAlive)
        Currency::withdraw(from, fee, FEE, KeepAlive) → FeeRouter 分账
        set_status_and_emit(STATUS_EXECUTED)
        deposit_event(TransferExecuted)
      })
   f. 若事务失败:deposit_event(TransferExecutionFailed),提案保留供 execute_transfer 重试
4. 最后 deposit_event(TransferFinalized { proposal_id, signatures_accepted, final_status })
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

## 8. 提案与 finalize 的区块写入(Step 2 · 离线聚合版)

### 8.1 两步式时序

```
区块 N  : 管理员A 发起 propose_transfer(...)
           → 创建提案 proposal_id=X
           → emit TransferProposed { from, beneficiary, amount, remark, expires_at, ... }
           → wuminapp 扫此事件,生成 QR 包含所有字段

离线    : 管理员 B..M 扫描 QR → 对 TransferVoteIntent 做 sr25519 签名 → 回传发起人

区块 N+k: 发起人(或任何签名账户)提交 finalize_transfer(X, sigs)
           → 循环验签 + 循环 cast_internal_vote
           → STATUS_PASSED 达阈值
           → 同一交易内 try_execute_transfer 自动执行
           → emit TransferExecuted + TransferFinalized
```

### 8.2 关键差异

- **原模式**:N 个管理员各发一笔在线 `vote_X` extrinsic,N 笔上链交易。
- **离线聚合**:仅 Tx 1 + Tx 2 两笔上链交易,中间签名聚合完全线下,不占链带宽。
- 幂等保护:`cast_internal_vote` 内部 `AlreadyVoted` 让重复 finalize 自动回滚。

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

## 11. Weight 估算(Step 2 已更新)

| Extrinsic | 预估 Weight | DB 读 | DB 写 |
| --- | --- | --- | --- |
| `propose_transfer` | ~55 ms | 5 | 7 |
| `finalize_transfer(n)` | ~60 ms + 40 ms × n | 6 + n | 8 + n |
| `execute_transfer`(手动重试) | ~75 ms | 4 | 4 |
| `finalize_safety_fund_transfer(n)` | ~60 ms + 40 ms × n | 6 + n | 8 + n |
| `finalize_sweep_to_main(n)` | ~60 ms + 40 ms × n | 6 + n | 8 + n |

说明:`n` = 聚合签名数量(= 参与代投的管理员数)。基础成本覆盖读取 `ProposalData` +
解析 Action + 构造 intent + 查阈值 + 调投票引擎;每签名增量包含一次 sr25519 验签
(~35 ms)+ 一次 `cast_internal_vote`(~5 ms DB 写)。实际值需跑 benchmark 生成,
占位公式详见 `src/weights.rs::finalize_base`。

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
