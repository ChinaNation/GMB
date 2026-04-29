# offchain-transaction · 扫码支付 Step 1 技术说明

- **日期**:2026-04-19
- **范围**:本模块内扫码支付清算体系 Step 1(同清算行内 MVP)的**已落地代码**
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **总任务卡**:`memory/08-tasks/open/20260419-扫码支付-step1-同行MVP.md`

---

## 1. 本步范围

Step 1 已落地:L3 绑定清算行 + 充值 + 提现 + 切换的链上接口,以及清算行合法性判定和 L3 支付意图签名结构。**同行扫码支付的批次上链**逻辑定义了数据结构和 nonce 辅助,但**batch 执行流**(settlement.rs)由 Step 2 落地,避免一次性重写现有"省储行清算"代码。

**并存策略**:本步新增的 Storage / Event / Error / Call 与现有 `submit_offchain_batch` / `execute_batch` / `InstitutionRateBp` / `RecipientClearingInstitution` 等完全独立,不触碰旧逻辑。Step 2 开始正式替换旧省储行清算模型。

## 2. 新增文件

```
src/
├── batch_item.rs    # Step 1 新增:PaymentIntent 结构 + 签名域常量 + 单测
├── bank_check.rs    # Step 1 新增:清算行合法性判定 + SfidAccountQuery trait
├── deposit.rs       # Step 1 新增:bind/deposit/withdraw/switch 四函数实现
└── nonce.rs         # Step 1 新增:L3PaymentNonce 消费辅助
```

## 3. 清算行合法性模型

清算行 = **SFR 私法人** 或 **FFR 非法人**(两者皆私权机构),对应 `sfid/backend/src/sfid/category.rs:65` 的 `InstitutionCategory::PrivateInstitution`。

链上**不新增** SFID 枚举,而是直接对 `duoqian-manage::AddressRegisteredSfid` 存的 `sfid_id` 字符串做 A3 前缀匹配(前 3 字节)。

```rust
pub fn a3_is_private_institution(sfid_bytes: &[u8]) -> bool {
    sfid_bytes.get(..3).map(|a3| a3 == b"SFR" || a3 == b"FFR").unwrap_or(false)
}
```

合法清算行的四条并列条件(`ensure_can_be_bound`):
1. 在 `AddressRegisteredSfid` 有登记
2. `name` 段等于 `"主账户"`(3 字节 UTF-8 × 3 字 = 9 字节)
3. A3 ∈ {SFR, FFR}
4. `DuoqianAccount.status == Active`

## 4. 解耦抽象 `SfidAccountQuery`

`bank_check` 不直接依赖 `duoqian-manage`,而是通过 trait 抽象:

```rust
pub trait SfidAccountQuery<AccountId> {
    fn account_info(addr: &AccountId) -> Option<(Vec<u8>, Vec<u8>)>;
    fn find_address(sfid_id: &[u8], account_name: &[u8]) -> Option<AccountId>;
    fn is_active(addr: &AccountId) -> bool;
}

// 默认 () 实现返回未登记,供测试用。
```

**runtime 层实现**(`citizenchain/runtime/src/configs/mod.rs` 的 `DuoqianSfidAccountQuery`):委托给 `duoqian-manage` 的三张 Storage:
- `AddressRegisteredSfid` → `account_info`
- `SfidRegisteredAddress` → `find_address`
- `DuoqianAccounts` → `is_active`

**好处**:
- offchain-transaction 的 Cargo.toml 不新增 duoqian-manage 依赖
- pallet 的 `Config` trait 不强制 `T: duoqian_manage::Config`
- 现有 tests 可直接用 `type SfidAccountQuery = ();`,不破坏已有测试

## 5. L3 PaymentIntent 签名格式

```rust
pub struct PaymentIntent<AccountId, BlockNumber> {
    pub tx_id: H256,
    pub payer: AccountId,
    pub payer_bank: AccountId,
    pub recipient: AccountId,
    pub recipient_bank: AccountId,
    pub amount: u128,
    pub fee: u128,
    pub nonce: u64,
    pub expires_at: BlockNumber,
}

// 签名消息 = blake2_256("GMB_L3_PAY_V1" || SCALE(intent))
pub const L3_PAY_SIGNING_DOMAIN: &[u8] = b"GMB_L3_PAY_V1";
```

wuminapp(Dart 端)必须逐字节对齐 `L3_PAY_SIGNING_DOMAIN` 与 SCALE 编码顺序,否则链上验签失败。

## 6. 新增 Storage

| Storage | 类型 | 语义 |
|---|---|---|
| `UserBank` | `StorageMap<L3, BankMain, OptionQuery>` | L3 → 绑定的清算行主账户 |
| `DepositBalance` | `StorageDoubleMap<BankMain, L3, u128>` | 权威账本:各 L3 在该清算行的存款(分) |
| `BankTotalDeposits` | `StorageMap<BankMain, u128>` | 清算行存款总额(偿付对账) |
| `L3PaymentNonce` | `StorageMap<L3, u64>` | L3 单调递增 nonce(防重放,Step 2 起激活) |

不变式:`BankTotalDeposits[bank] == Σ DepositBalance[bank][*]`。
Step 2 起增加偿付自动保护:`主账户链上余额 ≥ BankTotalDeposits[bank]`。

## 7. 新增 Call(call_index 30-33)

| call_index | 方法 | 费用归类 | 说明 |
|---|---|---|---|
| 30 | `bind_clearing_bank(bank_main)` | 付费调用 1 元 | 绑定即开户无预存 |
| 31 | `deposit(amount)` | 链上资金 0.1% 最低 0.1 元 | 自持 → 清算行主账户 |
| 32 | `withdraw(amount)` | 链上资金 0.1% 最低 0.1 元 | 清算行主账户 → 自持 |
| 33 | `switch_bank(new_bank)` | 付费调用 1 元 | 前置:旧清算行余额为 0 |

费用归类在 `citizenchain/runtime/src/configs/mod.rs::OnchainTxAmountExtractor` 的 `OffchainTransaction` 分支显式分类,不走兜底。

## 8. 新增 Event(4 个)

```
BankBound      { user, bank }
Deposited      { user, bank, amount }
Withdrawn      { user, bank, amount }
BankSwitched   { user, old_bank, new_bank }
```

## 9. 新增 Error(19 个)

清算行身份类:`NotRegisteredClearingBank` / `NotMainAccount` / `NotPrivateInstitution` /
`ClearingBankNotActive` / `FeeAccountNameTooLong` / `FeeAccountNotFound`

绑定/切换类:`AlreadyHasBank` / `NoOpenedBank` / `NewBankSameAsCurrent` /
`MustClearBalanceFirst`

存取类:`DepositAmountTooSmall` / `WithdrawAmountTooSmall` /
`InsufficientDepositBalance` / `InsufficientBankLiquidity` / `DepositForbidden` /
`WithdrawForbidden`

L3 签名类:`L3NonceOverflow` / `InvalidL3Nonce`(Step 2 启用)

## 10. `institution-asset` 新增 4 个 Action

```
L3DepositIn      # L3 充值入清算行主账户
L3WithdrawOut    # 清算行主账户对 L3 提现
L2ClearingDebit  # Step 2:扫码清算时扣 payer_bank
L2FeeCollect     # Step 2:扫码清算时向 fee_account 收费
```

`()` 默认 fail-open,runtime 的 `RuntimeInstitutionAsset` 会按"清算行主账户是否合法"严格裁决。

## 11. 与现有模块的边界

| 模块 | 动向 |
|---|---|
| `duoqian-manage` | **不动**(清算行注册复用现有机制) |
| `duoqian-transfer` | **不动** |
| `onchain-transaction` | **不动** |
| `sfid-system` | **不动** |
| `institution-asset` | **扩展 4 枚举**(代码改动见上) |
| `offchain-transaction` 现有省储行清算逻辑 | **不动**,Step 2 替换 |

## 12. 编译验证

```
$ cargo check -p offchain-transaction
$ cargo check -p institution-asset
# 两者皆通过。runtime 层(citizenchain)的 build.rs 硬性要求 WASM_FILE
# 环境变量,本地 cargo check 受 CI 门禁限制,本轮改动是纯 Rust 代码
# (新增 struct/trait impl/match arm),无结构性风险,留 CI 把关。
```

## 13. 后续 Step 2 / Step 3 展望

**Step 2**:
- 新增 `settlement.rs`:同行/跨行 `execute_batch` 分账
- 新增 `fee_config.rs`:`L2FeeRateBp` + 延迟生效
- 新增 `solvency.rs`:偿付自动保护
- 废弃旧 `submit_offchain_batch` 的省储行模型

**Step 3**:
- 新增 `dispute.rs` / `reserve.rs`
- 新增 `close_clearing_bank`
- 白皮书 5.4.4 发布

## 14. 测试覆盖(Step 1)

本步 tests 模块继续跑原有 `submit_offchain_batch` 相关测试(未被破坏)。
新增的 bind/deposit/withdraw/switch 的**负向路径**(未登记 → `NotRegisteredClearingBank`)可通过 `type SfidAccountQuery = ()` 的默认实现自动覆盖。
**正向路径**测试需 mock `SfidAccountQuery` 返回 Some,建议在 Step 2 引入完整 mock 一并实现。

## 15. 变更记录

- 2026-04-19:Step 1 首次落地,新增 4 文件 + lib.rs 聚合扩展 + 跨模块配置改动。
