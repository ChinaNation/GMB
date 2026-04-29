# 任务卡：修复 duoqian-transfer try_execute_transfer 缺少原子性保护

- 任务编号：20260327-fix-try-execute-transfer-atomicity
- 状态：open
- 所属模块：citizenchain/runtime
- 当前负责人：待分配
- 创建时间：2026-03-27
- 优先级：中（当前被余额预检缓解，非紧急）

## 任务需求

为 `duoqian-transfer` 的 `try_execute_transfer` 函数添加 `with_transaction` 原子性保护，防止转账成功但扣费失败时出现部分生效。

## 问题描述

`try_execute_transfer`（`citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs:397-468`）内部三步操作没有子事务包裹：

1. `Currency::transfer()` — 转账
2. `Currency::withdraw()` — 扣手续费
3. `set_status_and_emit()` — 标记 STATUS_EXECUTED

如果第 1 步成功但第 2 步失败：
- 转账已生效（钱已转走）
- 手续费未扣
- 状态仍为 STATUS_PASSED
- 外层 `vote_transfer` 吞掉错误返回 Ok(())，所有存储变更被提交
- 用户可通过 `execute_transfer` 手动重试 → 再次执行转账 → 双重转账

## 当前缓解措施

执行前有余额预检（`free >= amount + fee + ED`），且单 extrinsic 内无并发，预检到执行之间余额不会变化。触发条件极苛刻（需 KeepAlive/ED 边界与预检逻辑存在微妙差异）。

## 修复方案

用 `with_transaction` 包裹 `try_execute_transfer` 函数体：

```rust
fn try_execute_transfer(proposal_id: u64) -> DispatchResult {
    frame_support::storage::with_transaction(|| {
        match Self::do_execute_transfer(proposal_id) {
            Ok(()) => TransactionOutcome::Commit(Ok(())),
            Err(e) => TransactionOutcome::Rollback(Err(e)),
        }
    })
}
```

将现有函数体移到 `do_execute_transfer`，`try_execute_transfer` 只做事务包装。

## 部署约束

- **需要 runtime 升级**，不能重启链
- 建议与下次有 runtime 变更时打包一起升级

## 关联发现

同次审查中发现的前端状态映射 Bug（STATUS_EXECUTED=3 显示为"执行失败"）已在 node 侧直接修复，不涉及 runtime。
