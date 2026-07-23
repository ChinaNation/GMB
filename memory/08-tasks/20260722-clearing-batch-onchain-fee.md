# 任务卡：清算行批次上链收链上费（Step 3）

状态：★ 已完成（2026-07-22）★
- cargo test -p offchain = 25 passed / 0 failed（+batch_rejected_when_fee_account_cannot_pay_onchain_fee fail-closed 用例）。
- cargo test -p citizenchain = 48 passed / 0 failed。
- cargo build -p citizenchain --release（no_std WASM）= 通过，零警告零错误。
- 金标：同行结算单笔 fee=5 → 链上费 calculate_onchain_fee(5)=10 FEN，费用账户断言改 before+fee−链上费。
- 分账 80/10/10 复用 runtime OnchainExecutionFeeCharger（与 multisig 同一条路径），offchain 测试桩 MockOnchainFeeCharger 只验证费用账户被扣。
- 残留清理：删死常量 OTHER_BANK_BYTES + 修 MockCid 注释腐烂；mock NegativeImbalance 显式消费。零警告。
前置：Step 2（clearing-l2-fund-flow-spend-guard，已完成）

## 目标
清算行打包批次上链时,对本批**累计手续费收益**支付一次链上交易费(标准 80/10/10 分账),
由清算行从**费用账户**出。链下清算手续费仍 100% 归清算行(无省储会分成)。

## 产品规则（用户已定死）
- 链下清算手续费 100% 归清算行。
- 批次上链对累计手续费收一次链上交易费 = calculate_onchain_fee(Σfee) = max(Σfee×0.1%, 0.1元)。
- 从费用账户出,不碰用户存款准备金(solvency 不受影响)。

## 确认决策
- S3-①：费用账户不足 fail-closed 拒整批。
- S3-②：沿用 calculate_onchain_fee 取整（四舍五入 + max(·,10FEN)）。
- S3-③：收费基数=累计手续费 Σ item.fee_amount（对收益收费,非按转账本金）。

## 改动清单
- offchain/src/lib.rs：+Config `type OnchainFeeCharger`（完全限定 Balance 关联类型）；+Error::ClearingBatchOnchainFeeUnpaid。
- offchain/src/settlement.rs：execute_clearing_bank_batch 累加 total_batch_fee + 执行循环后对 fee_account_of(actor) charge 一次 + use OnchainFeeCharger。
- runtime/src/configs.rs：offchain::Config 接 OnchainExecutionFeeCharger<Runtime, Balances, OnchainExecutionFeeDistributor>。
- offchain/src/tests/mod.rs：+MockOnchainFeeCharger（真 calculate_onchain_fee + Balances::withdraw）并接 Config。
- offchain/src/tests/cases.rs：修既有同行结算费用账户断言（-链上费）+ 金标向量 + fail-closed 用例。

## 经济测算（核过）
100 笔 / 1 万元 / 0.01% → Σfee=1元=100FEN → calculate_onchain_fee(100)=max(0,10)=10FEN=0.1元 → 清算行净得 0.9 元。

## 验收标准
- cargo test -p offchain 全绿；cargo test -p citizenchain 全绿；cargo build -p citizenchain --release 通过。
- fail-closed：费用账户不足 → ClearingBatchOnchainFeeUnpaid + 全回滚。
- 不碰准备金：清算账户余额/BankTotalDeposits 收费前后恒等。

## 跨端跟进（本步不改，仅记录）
- node packer / onchina 打包前宜预检费用账户余额 ≥ 该批链上费。
