# 任务卡：清理 shengbank-interest 利率不可达分支

- 任务编号：20260501-090514
- 状态：done
- 所属模块：citizenchain/runtime/issuance/shengbank-interest
- 当前负责人：Codex
- 创建时间：2026-05-01 09:05:14

## 任务需求

清理 shengbank-interest 利率不可达分支

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
- 2026-05-01 实施：
  - 检查当前 `shengbank-interest/src/lib.rs`，确认代码已采用目标修复口径：通过 `const _: () = assert!(ENABLE_SHENGBANK_INTEREST_DECAY, ...)` 编译期锁定逐年递减制度。
  - `interest_bp_for_year` 已删除 `!ENABLE_SHENGBANK_INTEREST_DECAY` 与 `year > SHENGBANK_INTEREST_DURATION_YEARS` 两个返回型不可达分支，仅保留递减模型单一路径。
  - 年度上限由 `settle_next_years` 的 `last_year < SHENGBANK_INTEREST_DURATION_YEARS` 循环条件统一约束；利率函数只保留 `debug_assert!` 表达内部不变量。
  - 检查 `SHENGBANK_TECHNICAL.md`，确认文档已记录递减开关由编译期断言锁定、利率函数不保留不可达返回分支。
  - 已执行：`cargo test --manifest-path citizenchain/runtime/issuance/shengbank-interest/Cargo.toml`，结果 19 个单测全部通过。
  - 已执行：`cargo test --manifest-path citizenchain/runtime/issuance/shengbank-interest/Cargo.toml --features runtime-benchmarks`，结果 23 个测试全部通过。
  - 残留清理：未新增临时脚本、临时调试输出或无用文件；工作树中既有 `shengbank-interest` 其它权重/自动结算相关改动保持原状。
