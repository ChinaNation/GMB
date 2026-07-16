# onchain 交易与统一手续费技术文档

## 1. 模块定位

代码目录：`citizenchain/runtime/transaction/onchain/`。

本模块提供两项能力：

- `OnchainChargeAdapter`：实现 `pallet-transaction-payment::OnChargeTransaction`，消费 runtime 给出的唯一费用路由并执行链上交易费、投票费扣款。
- `OnchainFeeRouter`：将已扣手续费按 80% / 10% / 10% 分给当前块作者绑定的奖励钱包、国家储委会费用账户和安全基金账户。

本模块不维护费用类别表、不维护机构身份或管理员表，也不使用 weight、length 或动态 multiplier 计算制度费用。

## 2. 唯一费用协议

费用类型、付款账户、常量和链上费公式的唯一协议源位于：

- `citizenchain/runtime/primitives/src/fee_policy.rs`

统一类型为 `FeeRoute<AccountId, Balance>`：

- `Free`：系统、Root 回调或内部维护调用，外层不扣交易费。
- `Onchain { transaction_amount, payer }`：按 `max(round(amount × 0.1%), 0.1 元)` 收取；机构普通操作传零金额，所以固定为 0.1 元。
- `Offchain { fee_amount, payer: OffchainFeePayer::BatchItemPayers }`：链下清算批次可有多个付款公民；每个 item 的 `payer` 从其 L2 存款支付对应 `fee_amount`，外层适配器不重复扣款。收款方机构费用账户是手续费收款账户，不是付款账户。
- `Vote { payer }`：只有实际投票行为，由投票签名者固定支付 1 元。
- `Reject`：调用未分类、未开放、CID/账户不匹配、签名者不是该 CID 的 `admins`，或费用账户无法唯一解析时拒绝入池和执行。

`FeeRoute` 的收费分支都强制携带确切付款账户，不存在 `Option<payer>`、默认付款人或回落到签名者的表达空间。

`runtime/src/configs.rs::RuntimeFeeRouter` 是 `RuntimeCall -> FeeRoute` 的唯一映射。`CallFeeRoute` 只是依赖注入接口，不定义第二套费用类型。

## 3. 机构费用路由

机构身份主键为 `actor_cid_number`，外层 `origin` 是管理员钱包签名；机构账户没有私钥。

机构路由按以下顺序严格校验：

1. 从 `actor_cid_number` 解析机构码。
2. 通过 `RuntimeInstitutionAdminQuery` 校验签名者属于该 CID 的 `admins`。
3. 从 `InstitutionAccounts[(cid_number, InstitutionFee)]` 读取费用账户。
4. 校验 `AccountRegisteredCid[fee_account]` 的 CID 和账户名反向索引完全一致。
5. 若交易携带 `institution_account` / `funding_account`，再校验该账户与同一 CID 的正反索引完全一致。

任何一步失败都返回 `FeeRoute::Reject`。公权/私权存储同时出现同一 CID、费用账户缺失、正反索引不一致、跨 CID 使用账户或非管理员签名都不允许改扣管理员钱包。

机构发起提案、机构资料操作和机构账户操作属于链上操作，由该 CID 的费用账户支付最低 0.1 元。只有后续管理员执行 `cast_*` 等实际投票时，才由投票签名者支付 1 元。

Fullnode 不是机构。`bind_reward_wallet` / `rebind_reward_wallet` 由全节点自己的私钥签名，并由该签名者支付 0.1 元。

## 4. tip 与框架费用

- `primitives::fee_policy::TRANSACTION_TIP = 0` 是唯一协议值。
- Rust 统一签名构造器和 CitizenApp 都只编码 `tip=0`。
- CitizenWallet 在签名前解析 `Compact<tip>`，非零立即拒签。
- runtime 的 `can_withdraw_fee` 和 `withdraw_fee` 对非零 tip 返回 `InvalidTransaction::Payment`。
- `WeightToFee = 0`、`LengthToFee = 0`、固定 multiplier，不产生第六类框架费用。

tip 不属于交易费，不参与 `FeePaid`、RPC 聚合或 80/10/10 分账。

## 5. 扣款语义

`OnchainChargeAdapter` 只消费 `FeeRoute`：

- `Onchain`：调用 `primitives::fee_policy::calculate_onchain_fee`，从路由中的 `payer` 扣款。
- `Vote`：从路由中的 `payer` 扣固定 `VOTE_FLAT_FEE`。
- `Offchain` / `Free`：外层不扣款。
- `Reject`：返回 `InvalidTransaction::Call`。

余额扣款使用 `Precision::Exact + Preservation::Preserve + Fortitude::Polite`：必须完整扣除，并保证普通支出后不低于 ED；否则整笔交易失败。适配器不尝试第二付款账户，不做执行后退款。

成功扣款发出：

```text
FeePaid { who: 实际付款账户, fee: 完整手续费 }
```

其中 `fee` 已是完整链上交易费或投票费；协议不存在额外 tip。

## 6. 分账路由

分账常量同样来自 `primitives::fee_policy`：

- 全节点奖励钱包：80%。
- 国家储委会费用账户：10%。
- 安全基金账户：10%。

块作者缺失、奖励钱包未绑定或任一制度账户无法安全入账时，对应 credit 被销毁并发出 `FeeShareBurnt { reason, amount }`，绝不转给未知账户。原因包括：

- `AuthorMissing`
- `WalletUnbound`
- `FullnodeResolveFailed`
- `NrcMissing`
- `NrcResolveFailed`
- `SafetyFundResolveFailed`

## 7. 外部同步

- `citizenchain/crates/chain-signing/`：统一构造 `tip=0` 的交易扩展。
- `citizenchain/node/src/core/rpc.rs`：`fee_blockFees` 只累计 `FeePaid.fee`，不再拼接 FRAME tip 事件。
- `citizenapp/lib/rpc/signed_extrinsic_builder.dart`：热签 payload 和 extrinsic 固定 `tip=0`。
- `citizenwallet/lib/signer/payload_decoder.dart`：冷签前拒绝非零 tip。
- `citizenchain/onchina/`：机构身份仍只使用 CID、机构账户和 `admins`；不建立费用付款方缓存或第二路由表。

## 8. 测试要求

必须覆盖：

- 五种 `FeeRoute` 行为。
- 链上费四舍五入、最低 0.1 元和极大金额。
- 实际投票由签名者扣 1 元。
- 机构操作只扣精确费用账户，管理员余额不变。
- 费用账户缺失或映射不一致时直接拒绝，不回落管理员。
- Fullnode 绑定由签名者扣 0.1 元。
- 非零 tip 在 runtime 和 CitizenWallet 两端拒绝。
- 80/10/10 正常分账与所有安全销毁路径。
- `WeightToFee` / `LengthToFee` 不产生费用。

主要命令：

- `cargo test -p primitives -p onchain -p citizenchain`
- `cargo test -p chain-signing -p node -p onchina`
- `flutter test` 与 `flutter analyze`（CitizenApp、CitizenWallet）
