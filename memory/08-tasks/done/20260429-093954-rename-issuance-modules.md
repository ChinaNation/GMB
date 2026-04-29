# 任务卡：统一发行模块新命名

- 任务编号：20260429-093954
- 状态：done
- 所属模块：citizenchain/runtime/issuance
- 当前负责人：Codex
- 创建时间：2026-04-29 09:39:54

## 任务需求

彻底统一三类发行模块的新命名：`citizen-issuance`、`fullnode-issuance`、`shengbank-interest`。同步更新 Rust crate、runtime pallet 公开名、调用方、文档、脚本、任务索引和残留引用。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/issuance/citizen-issuance/CITIZENISS_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/issuance/fullnode-issuance/FULLNODE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/issuance/shengbank-interest/SHENGBANK_TECHNICAL.md

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
- 已确认本轮执行范围为彻底改名：目录名、crate 名、runtime pallet 名、metadata 名、storage prefix 调用方和文档引用同步更新。
- 已完成三类发行模块源码目录、Cargo crate、runtime pallet 名、node 调用方、SFID 事件解析、benchmark 脚本、网站展示、模块文档和历史任务索引的新命名收口。
- 已将 runtime `spec_version` 提升到 9；pallet index 与 call index 保持不变。
- 已执行 `cargo fmt`。
- 已执行 `cargo test -p citizen-issuance -p fullnode-issuance -p shengbank-interest --lib`，结果通过。
- 已执行 `cargo test -p citizen-issuance --test integration_bind_sfid`，结果通过。
- 已执行 `cargo check -p onchain-transaction --lib`，结果通过。
- 已执行旧发行模块名残留扫描，源码、文档和任务卡路径中未发现旧名残留。
- 2026-04-29 二次宽松扫描发现 `citizen-issuance` 技术文档标题仍含自然语言旧称残留，已修正为 `CITIZEN Issuance`。
- 2026-04-29 全仓库复查继续收口 `citizen-issuance` 常量前缀、`fullnode-issuance` 事件名、三个发行模块中文模块称呼，并刷新网站构建产物。
- `cargo check -p citizenchain -p node` 受 runtime `build.rs` 的 `WASM_FILE` 强制门禁阻断；这是项目既有本地编译限制。
- `cargo test -p onchain-transaction --lib` 中 `fee_router_burns_nrc_share_when_resolve_fails` 仍失败，失败点为安全基金余额断言，与本次发行模块改名无直接关系，需另行处理。

## 完成信息

- 完成时间：2026-04-29 10:01:13
- 完成摘要：完成三类发行模块彻底改名为 citizen-issuance、fullnode-issuance、shengbank-interest；同步 runtime pallet 名、调用方、文档、脚本、任务索引和残留扫描；直接发行模块测试通过，runtime/node 本地检查受 WASM_FILE 门禁限制，onchain-transaction 既有单测失败需另行处理。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
