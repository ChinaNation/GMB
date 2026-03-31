# 第3步：runtime offchain-transaction-pos 模块适配验证

## 状态：open

## 前置依赖

- 第2步 node 节点改造完成

## 任务目标

验证现有 offchain-transaction-pos 模块的批量清算逻辑是否完全满足省储行即时清算交易的需求，做必要微调。

## 检查与改动范围

### 1. execute_batch 直接扣款逻辑

- 当前状态：已实现，逐笔从 payer 扣款转给 recipient + fee_account
- 预期：保持不变，不加冻结机制
- 检查：确认 KeepAlive 存在性要求、ED 校验等与快捷支付场景兼容

### 2. enqueue_offchain_batch 批量入队

- 当前状态：已实现，支持签名验证、batch_seq 序列号、重放防护
- 检查项：
  - 省储行节点自动打包调用此方法时，调用方式是否兼容（extrinsic 提交 vs 内部调用）
  - relay submitter 白名单是否已包含省储行节点的提交账户
  - batch_seq 序列号管理是否与节点端批量打包器对齐

### 3. submit_offchain_batch 直接执行路径

- 当前状态：要求无 pending 队列时才能走直接路径
- 检查：评估是否需要放宽此限制，或省储行统一走 enqueue 路径

### 4. bind_clearing_institution

- 当前状态：已实现，收款方绑定清算省储行，1年冷却期
- 检查：确认 wuminapp 调用此 extrinsic 的签名方式和参数格式

### 5. 费率治理

- 当前状态：已实现，1-10 bp（0.01%-0.1%）
- 检查：确认费率计算与 node 端预扣手续费一致

### 6. relay submitter 白名单

- 当前状态：已实现，governance 管控
- 检查：确认省储行节点的提交账户已在白名单中，或补充初始化流程

### 7. institution-asset-guard 适配

- 检查：OffchainBatchDebit action 是否覆盖省储行即时清算场景
- 可能改动：新增枚举值（如有必要）

## 涉及文件

- `citizenchain/runtime/transaction/offchain-transaction-pos/src/lib.rs` — 主模块
- `citizenchain/runtime/transaction/institution-asset-guard/src/lib.rs` — 可能微调
- `citizenchain/runtime/primitives/` — 常量或类型定义（如有需要）

## 关键设计决策

- 不新增冻结/解冻机制，保持 execute_batch 直接扣款
- 省储行统一走 enqueue_offchain_batch 路径（更安全，有重试机制）
- 费率以 enqueue 时快照为准（已有逻辑）

## 验收标准

- [ ] 省储行节点批量打包的 batch 能成功 enqueue 并 process
- [ ] execute_batch 对快捷支付场景的余额校验正确
- [ ] relay submitter 白名单配置流程明确
- [ ] 费率计算与 node 端预扣一致，无精度差异
- [ ] 端到端测试：wuminapp 扫码 → 省储行确认 → 批量上链 → 结算完成
