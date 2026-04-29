# 任务卡：修复 smoldot 交易提交静默丢失 + 投票/提案状态显示错误

- 任务编号：20260327-fix-smoldot-submit-and-vote-display
- 状态：open（代码已改，待编译验证）
- 所属模块：wuminapp / citizenchain/node
- 当前负责人：Claude
- 创建时间：2026-03-27
- 优先级：高

## 问题描述

### Bug 1：smoldot `author_submitExtrinsic` 静默丢失交易

smoldot 的 `author_submitExtrinsic` handler 使用 `submit_transaction`（fire-and-forget），
本地计算 blake2b hash 后立即返回，不等待任何验证反馈。交易可能因 nonce 错误、era 过期、
网络不通等原因丢失，但 Dart 端收到 hash 认为"已提交"。

叠加效果：交易丢失时不触发 `NonceManager.rollback()`，导致本地 nonce 被污染，
后续所有重试使用的 nonce 都比链上期望值大，形成 nonce 死锁循环，用户无法投票。

### Bug 2：提案状态 STATUS_EXECUTED=3 显示为"执行失败"

链上 `STATUS_EXECUTED = 3` 在 node 和 wuminapp 中被错误映射为"执行失败"，
导致已成功执行的转账提案显示为失败。

## 已完成的修复

### smoldot handler（根治交易丢失 + nonce 死锁）

**文件**：`wuminapp/third_party/smoldot-pow/light-base/src/json_rpc_service/background.rs`

将 `author_submitExtrinsic` handler 中的 `submit_transaction`（fire-and-forget）
替换为 `submit_and_watch_transaction`，等待第一个有意义的状态后再返回：
- Validated / Broadcast → 返回 txHash（成功）
- Dropped → 返回 JSON-RPC error（失败）→ Dart 端触发 rollback → nonce 不被污染

### node 状态映射（3 处）

- `citizenchain/node/backend/src/governance/proposal.rs:642` — "执行失败" → "已执行"
- `citizenchain/node/frontend/governance/ProposalDetailPage.tsx:340` — "执行失败" → "已执行"
- `citizenchain/node/frontend/assets/styles/global.css:796` — 黄色 → 绿色

### wuminapp 状态映射

- `wuminapp/lib/governance/transfer_proposal_detail_page.dart` — 新增 `_statusExecuted = 3` 分支，
  `_statusLabel` 返回"已执行"，`_statusColor` 返回绿色，default 改为"未知"/灰色。

## 待验证

- [ ] smoldot Rust 编译通过
- [ ] wuminapp Flutter 编译通过
- [ ] 提交投票 → 交易有效时正常返回 hash
- [ ] 提交投票 → 交易无效时返回错误，用户看到"投票失败"
- [ ] 投票失败后立即重试，nonce 正确，不再死锁
- [ ] node 提案状态 3 显示"已执行"（绿色）
- [ ] wuminapp 提案详情页状态 3 显示"已执行"（绿色）

## 关联任务

- `20260327-fix-try-execute-transfer-atomicity.md` — runtime 层原子性问题（低优先级，需 runtime 升级）
