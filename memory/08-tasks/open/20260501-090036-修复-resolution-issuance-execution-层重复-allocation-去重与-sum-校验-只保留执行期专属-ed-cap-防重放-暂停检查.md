# 任务卡：修复 resolution-issuance execution 层重复 allocation 去重与 sum 校验，只保留执行期专属 ED/cap/防重放/暂停检查

- 任务编号：20260501-090036
- 状态：done
- 所属模块：citizenchain/runtime/issuance/resolution-issuance
- 当前负责人：Codex
- 创建时间：2026-05-01 09:00:36

## 任务需求

修复 resolution-issuance execution 层重复 allocation 去重与 sum 校验，只保留执行期专属 ED/cap/防重放/暂停检查

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
  - 精简 `citizenchain/runtime/issuance/resolution-issuance/src/execution.rs` 的执行层 allocation 校验。
  - 保留 `validate_execution_allocations` 作为共享结构性校验入口，继续覆盖白名单集合、收款人唯一性、单笔非零和总额匹配。
  - 删除执行层重复的 `BTreeSet<Vec<u8>>` 去重、二次 `ZeroAmount` 检查、二次 sum 累加与 `TotalMismatch` 检查。
  - 执行层保留 ED 检查、暂停、防重放、理由长度、单笔 cap、累计 cap 和入账结果检查。
  - 清理 `execution.rs` 中不再需要的 `codec::Encode`、`Zero`、`BTreeSet`、`Vec` import。
  - 更新 `RESOLUTIONISSUANCE_TECHNICAL.md`，记录 allocation 结构性校验集中在 `validation.rs`，执行层只保留执行期专属检查。
  - 已执行：`rustfmt citizenchain/runtime/issuance/resolution-issuance/src/execution.rs`。
  - 已执行：`cargo test --manifest-path citizenchain/runtime/issuance/resolution-issuance/Cargo.toml`，结果 16 个单测全部通过。
  - 已执行：`cargo test --manifest-path citizenchain/runtime/issuance/resolution-issuance/Cargo.toml --features runtime-benchmarks`，结果 16 个单测全部通过。
  - 残留清理：未新增临时脚本、临时调试输出或无用文件；工作树中既有其它 `resolution-issuance` 文件改动保持原状，仅新增本任务的 `execution.rs` 职责收敛。
