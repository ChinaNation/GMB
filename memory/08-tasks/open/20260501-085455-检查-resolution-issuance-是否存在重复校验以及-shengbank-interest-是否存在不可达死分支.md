# 任务卡：检查 resolution-issuance 是否存在重复校验以及 shengbank-interest 是否存在不可达死分支

- 任务编号：20260501-085455
- 状态：done
- 所属模块：citizenchain/runtime/issuance
- 当前负责人：Codex
- 创建时间：2026-05-01 08:54:55

## 任务需求

检查 resolution-issuance 是否存在重复校验以及 shengbank-interest 是否存在不可达死分支

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
- 2026-05-01 只读核查结论：两个问题均成立。
  - `resolution-issuance/src/validation.rs` 的 `validate_proposal_allocations` 已校验白名单集合完全一致、收款人唯一、单笔非零、sum 匹配；`validate_execution_allocations` 又委托同一逻辑。
  - `resolution-issuance/src/execution.rs` 在调用 `validate_execution_allocations` 后，又用 `BTreeSet<Vec<u8>>` 重做收款人去重、单笔非零、sum 匹配；其中 ED 检查和执行 cap/暂停/防重放属于执行层必要检查，其余为重复。
  - `shengbank-interest/src/lib.rs` 的 `interest_bp_for_year` 中 `if !ENABLE_SHENGBANK_INTEREST_DECAY` 在当前常量 `ENABLE_SHENGBANK_INTEREST_DECAY = true` 下不可达。
  - `interest_bp_for_year` 中 `if year > SHENGBANK_INTEREST_DURATION_YEARS` 在当前调用链下不可达：`settle_next_years` 的循环条件保证 `last_year < SHENGBANK_INTEREST_DURATION_YEARS`，因此传入 `mint_interest_for_year` / `interest_bp_for_year` 的 `settling_year = last_year + 1` 不会超过上限。
  - 本轮仅做只读检查和任务卡记录，未修改业务代码，未执行测试。
