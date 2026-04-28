# 任务卡：sfid 修复 clearing_bank_watcher 启动崩溃

- 任务编号：20260428-115654
- 状态：open
- 所属模块：sfid
- 当前负责人：Codex
- 创建时间：2026-04-28 11:56:54

## 任务需求

修复 `sfid/backend` 启动时 `clearing_bank_watcher` 在 Tokio runtime 未初始化前调用 `tokio::spawn` 导致的启动崩溃。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/05-modules/sfid/backend/chain/CHAIN_TECHNICAL.md

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
- 已定位根因：`clearing_bank_watcher::spawn_watcher()` 在同步 `main()` 初始化阶段直接调用 `tokio::spawn`，早于 Tokio runtime 创建
- 已将 `ClearingBankNodeCache` 改为在同步启动阶段先构造并放入 `AppState`
- 已将 watcher 启动时机改到 `runtime.block_on(...)` 内，避免无 Tokio reactor 的启动 panic
- 已将 `spawn_watcher` 重构为 `start_watcher`，并显式要求当前调用上下文已经处于 Tokio runtime
- 已补充 `memory/05-modules/sfid/backend/chain/CHAIN_TECHNICAL.md`，固定 `ClearingBankNodes watcher` 的启动约束
- 已完成验证：`cargo fmt --manifest-path /Users/rhett/GMB/sfid/backend/Cargo.toml`
- 已完成验证：`cargo check --manifest-path /Users/rhett/GMB/sfid/backend/Cargo.toml`
- 本轮未执行整机启动联调；当前确认范围为 `sfid/backend` 编译通过且 watcher 启动顺序已修正
