# 结合全仓库实现复查 `resolution-destro-gov` / `grandpa-key-gov` / `admins-origin-gov`

## 任务目标

结合全仓库实现，复查以下模块是否存在：

- 安全漏洞
- 可改进点
- 文档与实现偏差
- 中文注释或技术文档更新需求
- 残留实现、残留兼容层、残留脚本配置

本轮只负责检查，不直接改代码。

## 检查范围

- `citizenchain/runtime/governance/resolution-destro-gov`
- `citizenchain/runtime/governance/grandpa-key-gov`
- `citizenchain/runtime/governance/admins-origin-gov`
- `citizenchain/runtime/governance/voting-engine-system`
- `citizenchain/runtime/src/configs/mod.rs`
- 相关执行侧、路由侧、技术文档、上下文脚本、历史任务卡

## 执行记录

- 2026-04-05 12:38: 创建任务卡，待开始全仓库交叉复查。
- 2026-04-05 13:12: 已复查 `resolution-destro-gov`、`grandpa-key-gov`、`admins-origin-gov` 源码、runtime 接线、`voting-engine-system` 联动、技术文档、`MODULE_TAG` 注册表、上下文脚本和历史任务卡。
- 2026-04-05 13:12: 已执行验证：
  - `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/resolution-destro-gov/Cargo.toml`
  - `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/grandpa-key-gov/Cargo.toml`
  - `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/admins-origin-gov/Cargo.toml`
  - `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/resolution-destro-gov/Cargo.toml --features runtime-benchmarks`
  - `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/grandpa-key-gov/Cargo.toml --features runtime-benchmarks`
  - `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/admins-origin-gov/Cargo.toml --features runtime-benchmarks`

## 当前结论

- 本轮未发现新的高危可利用漏洞。
- 三个模块当前都属于 `voting-engine-system` 的内部投票消费模块，不走 `RuntimeJointVoteResultCallback` 联合投票回调链路。
- 主要剩余问题集中在 `weights.rs` 残留、技术文档状态口径漂移、测试夹具可信度和上下文脚本登记不全。

## 主要发现

1. `resolution-destro-gov`、`grandpa-key-gov`、`admins-origin-gov` 的 `weights.rs` 仍明显滞后于现实现状，proof 注释持续引用已删除的本地存储，审计可信度不足。
2. `resolution-destro-gov` 技术文档仍写“自动执行失败后提案保留为已通过状态”，但代码已改为覆写 `STATUS_EXECUTION_FAILED`，文档与状态机口径不一致。
3. `admins-origin-gov` 有一条单测直接对带 `MODULE_TAG` 前缀的 `ProposalData` 做整段解码，测试能过但没有真正校验到模块数据边界，属于测试残留。
4. `memory/scripts/load-context.sh` 只登记了 `grandpa-key-gov`，没有为 `resolution-destro-gov` 与 `admins-origin-gov` 提供模块级上下文入口，属于工具链残留。

## 本轮处理

- 已更新 `resolution-destro-gov` 技术文档，修正“自动执行失败后状态”口径，明确当前实现会覆写为 `STATUS_EXECUTION_FAILED`，且手动执行接受 `STATUS_PASSED` / `STATUS_EXECUTION_FAILED`。
- 已更新 `admins-origin-gov` 技术文档中的测试结果，修正为当前实测 `20 passed`。
- 已为 `resolution-destro-gov` 与 `admins-origin-gov` 补齐 `memory/scripts/load-context.sh` 模块级上下文入口。
- 已清理 `admins-origin-gov` 一条测试残留：改为先校验 `MODULE_TAG` 再解码 `ProposalData`，避免误把带前缀原始字节整体当业务结构。
- 已清理 `resolution-destro-gov` 测试模块的无用 import 警告。

## 本轮回归

- `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/resolution-destro-gov/Cargo.toml`
- `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/admins-origin-gov/Cargo.toml`
- `bash -n /Users/rhett/GMB/memory/scripts/load-context.sh`
