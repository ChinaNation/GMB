# 任务卡：清算账户接入 L2 资金流 + 支出保护（Step 2）

状态：★ 已完成（2026-07-22）★
- cargo test -p offchain = 24 passed / 0 failed（含同行结算端到端改清算账户）。
- cargo test -p citizenchain = 48 passed / 0 failed，零警告。
- 新用例全绿：clearing_account_only_allows_debit_and_withdraw（漏洞回归+门禁矩阵）、clearing_account_is_reserved_against_squatting（防占号）；ordinary_account_allows_all_actions 已包进 externalities。
- 残留清理：删死枝枚举 L2FeeCollect/OffchainBatchDebit（全仓 0 引用）+ 删已成残桩的 main_account_of。
- cargo build -p citizenchain --release（no_std WASM）= 通过，零警告。
前置：CID 主键（20260722-clearing-cid-primary-key，已完成）、清算账户 OP_CLEARING=0x06（20260722-clearing-account-op06-custom-op07，已完成）

## 目标
把 L2 充值/提现/结算/偿付的资金落点从清算行**主账户**改到**清算账户**（新增 `clearing_account_of` 原语），
并给清算账户加**支出白名单**：只准扫码清算扣款（L2ClearingDebit）与用户提现（L3WithdrawOut），
堵死管理员经 multisig-transfer 挪用用户存款准备金的真实漏洞。

## 产品规则（用户已定死）
- 用户充值直接进清算行的**清算账户**，不经主账户。
- 清算账户资金只能凭用户钱包签名支出（扫码逐笔验 L3 签名、提现用户本人发起）。
- 清算行管理员不能动清算账户的钱。

## 确认决策
- S2-①：结算路径落地 `can_spend(payer_clearing, L2ClearingDebit)` 门禁（让声明动作生效）。
- S2-②：`ensure_can_be_bound` 绑定期强制清算账户已派生（资格收紧到 SFGF）。
- S2-③：删除死枝枚举 `L2FeeCollect` / `OffchainBatchDebit`。

## 改动清单（具体到文件/函数）
- offchain/src/bank_check.rs：+ACCOUNT_NAME_CLEARING、+clearing_account_of、ensure_can_be_bound 加 fail-fast；main_account_of 注释改「身份锚」。
- offchain/src/lib.rs：+Error::ClearingAccountNotFound、+Error::ClearingDebitForbidden。
- offchain/src/deposit.rs：充值/提现落点改 clearing_account_of；withdraw 门禁源改清算账户。
- offchain/src/settlement.rs：payer/recipient 本金落点改 clearing_account_of；execute_single_item 加 L2ClearingDebit 门禁；use 补 InstitutionAssetAction。
- offchain/src/solvency.rs：偿付读清算账户；不变式注释改「清算账户 free_balance ≥ BankTotalDeposits」。
- runtime/src/configs.rs：+is_clearing_account（私法人反查索引）、can_spend 加清算账户分支、is_reserved 加防占号。
- primitives/src/institution_asset.rs：删 2 个死枝变体 + 测试同步。

## 身份锚 vs 资金池
- 不变（主账户）：institution_account 参数、ensure_institution_account(...,ACCOUNT_NAME_MAIN)、岗位授权、绑定资格/节点解析。
- 改线（清算账户）：充值/提现/结算本金/偿付准备金。
- 不变（费用账户）：手续费归集目标 fee_account_of。
- 保护外科式：清算行主账户仍普通可动（银行自有营运资金），只锁清算账户。

## 验收标准
- cargo test -p offchain 全绿；cargo test -p citizenchain-runtime 全绿；cargo build -p citizenchain-runtime 通过。
- 漏洞回归：管理员 multisig-transfer 且 funding_account=清算账户 → 拒；对主账户 → 放行。
- 落点正确性：充值后清算账户余额↑、主账户不变；同/跨行结算与偿付按清算账户守住。
- 死枝清零：L2FeeCollect/OffchainBatchDebit 全仓无引用。

## 跨端跟进（本步不改，仅记录）
- onchina/CitizenWallet 若有「清算行准备金/偿付率」展示且直接读主账户余额，需改读清算账户。

## 落地顺序
本卡（Step 2）→ Step 3（批次上链收链上费）。
