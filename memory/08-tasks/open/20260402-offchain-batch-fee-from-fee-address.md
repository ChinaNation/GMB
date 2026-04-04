# 任务卡：省储行清算批次链上手续费从 fee_address 扣取

任务需求：省储行提交清算批次上链时，按批次中链下手续费总额收取链上交易费，费用从省储行的 fee_address 自动扣取，不再免费。

所属模块：citizenchain/runtime（主）

## 背景

省储行清算流程：
1. 用户扫码支付 10000 元 → 收款人收到 10000 元，付款人额外支付链下手续费 100 元
2. 链下手续费 100 元转入省储行的 fee_address
3. 省储行 signing admin 将清算批次提交上链（submit_offchain_batch）
4. **当前问题**：submit_offchain_batch 被标记为 NoAmount，链上交易费 = 0，完全免费

## 设计目标

- 清算批次的主交易金额（用户支付的 10000 元）不收链上手续费
- 省储行获得的链下手续费（100 元）要收链上交易费
- 链上交易费 = 链下手续费总额 × 链上费率（当前 0.1%）
- 链上交易费从 fee_address 自动扣取（runtime 内部操作，不需要 fee_address 的私钥）
- fee_address 的多签保护不受影响（institution-asset-guard 仍然拦截人为转账）

## 费用流向示例

```
清算批次包含 100 笔交易，链下手续费总计 500 元

链上交易费 = 500 元 × 0.1% = 0.50 元
fee_address 净收入 = 500 - 0.50 = 499.50 元
链上交易费分账：矿工 80% + 国储会手续费账户 10% + 安全基金 10%
```

## 技术方案

### 1. AmountExtractor 改动

文件：`citizenchain/runtime/src/configs/mod.rs`

将 `submit_offchain_batch` 从 `NoAmount` 改为提取批次中链下手续费总额：

```rust
// 当前（免费）：
offchain_transaction_pos::pallet::Call::submit_offchain_batch { .. } => {
    onchain_transaction_pow::AmountExtractResult::NoAmount
}

// 改为（按链下手续费收费）：
offchain_transaction_pos::pallet::Call::submit_offchain_batch { batch, .. } => {
    let total_fee: u128 = batch.iter()
        .map(|item| item.offchain_fee_amount.saturated_into::<u128>())
        .sum();
    if total_fee == 0 {
        onchain_transaction_pow::AmountExtractResult::NoAmount
    } else {
        onchain_transaction_pow::AmountExtractResult::Amount(total_fee.saturated_into())
    }
}
```

enqueue_offchain_batch 和 process_queued_batch 保持 NoAmount（排队和处理不重复收费）。

### 2. FeePayerExtractor 改动

文件：`citizenchain/runtime/src/configs/mod.rs`

当前 `RuntimeFeePayerExtractor::fee_payer` 对所有调用返回 `None`（由交易签名者支付）。
需要对 `submit_offchain_batch` 返回 fee_address：

```rust
impl onchain_transaction_pow::CallFeePayer<AccountId, RuntimeCall> for RuntimeFeePayerExtractor {
    fn fee_payer(_who: &AccountId, call: &RuntimeCall) -> Option<AccountId> {
        match call {
            RuntimeCall::OffchainTransactionPos(
                offchain_transaction_pos::pallet::Call::submit_offchain_batch { institution, .. }
            ) => {
                // 从 institution 派生 fee_address
                offchain_transaction_pos::Pallet::<Runtime>::fee_account_of(*institution).ok()
            }
            _ => None,
        }
    }
}
```

### 3. institution-asset-guard 白名单

文件：`citizenchain/runtime/transaction/institution-asset-guard/src/lib.rs`

当前 fee_address 的合法操作只有 `OffchainBatchDebit` 和 `OffchainFeeSweepExecute`。
OnChargeTransaction 的扣费不经过 institution-asset-guard（它是 runtime 内部余额操作），所以不需要额外改动。

但需要确认：institution-asset-guard 的拦截点是在 `Balances::transfer` 等 extrinsic 层面，而 `OnChargeTransaction::withdraw_fee` 调用的是 `Currency::withdraw`，属于不同的执行路径，不会触发 asset-guard 检查。

### 4. 边界情况

- fee_address 余额不足以支付链上手续费 → 交易被拒绝（withdraw_fee 失败），批次无法上链
- 解决：fee_address 在首次清算收入到账后才有余额，首笔清算上链时 fee_address 可能为空
- 方案：首笔清算如果 fee_address 余额不足，fallback 到 signing admin 支付，或在创世时给 fee_address 预存少量余额

## 执行顺序

```
Step 1: 修改 AmountExtractor — submit_offchain_batch 按链下手续费总额计费
Step 2: 修改 FeePayerExtractor — submit_offchain_batch 的手续费由 fee_address 支付
Step 3: 验证 institution-asset-guard 不拦截 OnChargeTransaction 扣费
Step 4: 处理首笔清算 fee_address 余额为空的边界情况
Step 5: 补充测试
```

## 必须遵守

- 不可突破模块边界
- 涉及 runtime 改动，需要 spec_version 递增
- 需要链上升级才能生效
- 不清楚逻辑时先沟通

## 验收标准

- submit_offchain_batch / enqueue_offchain_batch 链上交易费 = 批次链下手续费总额 × 0.1%
- process_queued_batch 链上交易费 = 入队时快照的手续费总额 × 0.1%
- 链上交易费从提交者账户扣取
- fee_address 的多签保护不受影响
- 首笔清算 fee_address 为空时有兜底方案
- spec_version 已递增
- 测试通过
