# 任务卡：检查 fullnode-issuance bind_reward_wallet 是否存在任意签名账户可永久写入 RewardWalletByMiner 导致存储膨胀的问题

- 任务编号：20260501-081928
- 状态：done
- 所属模块：citizenchain/runtime/issuance
- 当前负责人：Codex
- 创建时间：2026-05-01 08:19:28

## 任务需求

检查 fullnode-issuance bind_reward_wallet 是否存在任意签名账户可永久写入 RewardWalletByMiner 导致存储膨胀的问题

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 2026-05-01 核查结论：问题成立。
  - `citizenchain/runtime/issuance/fullnode-issuance/src/lib.rs:128-140` 中 `bind_reward_wallet` 仅执行 `ensure_signed`、未绑定检查和 `wallet != miner` 检查，随后直接写入 `RewardWalletByMiner`。
  - `RewardWalletByMiner` 是 `StorageMap<AccountId, AccountId>`，当前代码未提供 TTL、押金、解绑、清理或矿工资格校验。
  - `citizenchain/runtime/src/configs/mod.rs:139-156` 的 `RuntimeCallFilter` 默认放行未特别匹配的调用，未拦截 `FullnodeIssuance`。
  - `citizenchain/runtime/src/configs/mod.rs:403-405` 将 `FullnodeIssuance` 归类为需要支付 `100000` 的调用，因此攻击前提是账户能签名并支付交易费，而不是完全免费。
  - 修复前模块文档已明确承认“不做真实矿工资格校验”和“无效状态膨胀”风险。
- 影响判断：
  - 不会造成多发奖励，因为 `on_finalize` 只按共识 digest 识别出的真实区块作者发奖。
  - 会形成低成本永久状态膨胀向量：每个可付费账户可写入一条长期存在的 `RewardWalletByMiner` KV。
- 建议修复方向：
  - 优先引入矿工资格来源后在 `bind_reward_wallet` 前置校验。
  - 若短期没有矿工注册表，可考虑绑定押金/可回收解绑、TTL/过期清理、或只允许最近出块作者绑定等方案。
  - 修复属于 runtime 链上规则变更，需要同步更新权重、测试和 `FULLNODE_TECHNICAL.md`。
- 前一轮仅做安全核查与任务卡记录，未修改 runtime 代码，未执行测试。
- 2026-05-01 修复执行：
  - `fullnode-issuance` 新增 `LastAuthoredBlockByMiner`，在 `on_finalize` 成功解析 PoW 作者后记录最近真实出块高度。
  - `bind_reward_wallet` 新增真实出块记录校验，未出过块的签名账户返回 `MinerNeverAuthoredBlock`，不能再写入永久 `RewardWalletByMiner`。
  - 保留原有“未绑定”“奖励钱包不得等于矿工账户”校验，绑定表仍只影响后续奖励收款地址，不改变出块作者身份。
  - `on_initialize` 最坏路径预算从 `reads_writes(3, 3)` 上调为 `reads_writes(3, 4)`，覆盖出块记录写入。
  - `src/benchmarks.rs` 为 `bind_reward_wallet` 预置已出块矿工，`src/weights.rs` 对新增读取做保守补偿。
- 2026-05-01 测试与文档：
  - 新增/调整测试覆盖：未出块账户绑定失败、首次出块记录生成、首次奖励进入矿工账户、出块后绑定成功、后续奖励进入绑定钱包。
  - 已更新 `memory/05-modules/citizenchain/runtime/issuance/fullnode-issuance/FULLNODE_TECHNICAL.md`，删除“任意签名账户可先绑定”的旧 trade-off 口径。
  - 已执行：`cargo test --manifest-path citizenchain/runtime/issuance/fullnode-issuance/Cargo.toml`，结果 19 个单测全部通过。
  - 已执行：`cargo test --manifest-path citizenchain/runtime/issuance/fullnode-issuance/Cargo.toml --features runtime-benchmarks`，结果 19 个单测全部通过。
  - 已执行：`cargo fmt --manifest-path citizenchain/runtime/issuance/fullnode-issuance/Cargo.toml`。
  - 残留清理：未新增临时脚本、临时调试输出或无用文件；工作树中既有/并行出现的 `resolution-issuance`、`shengbank-interest` 相关改动未纳入本任务处理。
