# 第2步-A：offchain-transaction-pos 简化密钥机制

## 状态：open

## 任务目标

删除 verify key + relay submitter 双密钥机制，改为直接复用省储行 9 个管理员身份进行 batch 签署和提交。

## 设计

- 任意一个省储行管理员都可以签署 batch + 提交上链
- 链上验证：提交者是否是该省储行的管理员 + batch 签名是否该管理员签的
- 9 个管理员任意 1 个能用就不停服
- 管理员丢失私钥 → 走现有管理员更换治理流程

## 改动范围

### 删除的代码

- `VerifyKeys` 存储 + `PendingVerifyKeys` + `VerifyKeyEpoch` + `VerifyKeyEffectiveAt`
- `RelaySubmitters` 存储 + `RelaySubmittersProposalActions`
- extrinsic：`init_verify_key`、`emergency_rotate_verify_key`、`init_relay_submitters`、`propose_relay_submitters`、`vote_relay_submitters`
- `OffchainBatchVerifier` trait（不再需要外部验签器）
- `QueuedBatchRecord` 中的 `verify_key_epoch_snapshot`
- 相关错误枚举和事件

### 修改的逻辑

- `submit_offchain_batch` / `enqueue_offchain_batch`：
  - 验证提交者是该省储行的管理员（`is_prb_admin`）
  - 用提交者公钥验证 batch_signature（sr25519）
- `process_queued_batch`：去掉 verify key epoch 检查
- Config trait：去掉 `OffchainBatchVerifier`、`MaxVerifyKeyLen`、`MaxRelaySubmitters` 关联类型

### 涉及文件

- `citizenchain/runtime/transaction/offchain-transaction-pos/src/lib.rs` — 核心重构
- `citizenchain/runtime/src/configs/mod.rs` — Config 实现适配
- 相关测试代码

## 安全设计

- 链上验证管理员身份（`duoqian_admins` 白名单）
- 即使有人伪造节点配置，链上拒绝非管理员提交
- 管理员更换走现有治理投票流程

## 验收标准

- [ ] 管理员可直接签署 batch 并提交上链
- [ ] 非管理员提交被链上拒绝
- [ ] verify key / relay submitter 相关代码和存储清理干净
- [ ] 编译通过，测试通过
