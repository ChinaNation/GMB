# 任务卡：只读分析 voting-engine internal_vote 与 active_proposal_limit 以及 admins-change 回调中的 L1-L5 审查问题是否存在

- 任务编号：20260430-213345
- 状态：done
- 所属模块：citizenchain-runtime-governance
- 当前负责人：Codex
- 创建时间：2026-04-30 21:33:45

## 任务需求

只读分析 voting-engine internal_vote 与 active_proposal_limit 以及 admins-change 回调中的 L1-L5 审查问题是否存在

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

- 只读分析结论
- 修复必要性判断

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已读取 `citizenchain/runtime/governance/voting-engine/src/internal_vote.rs`、`active_proposal_limit.rs`、`voting-engine/src/lib.rs`、`citizenchain/runtime/src/configs/mod.rs`、`citizenchain/runtime/governance/admins-change/src/lib.rs`。
- L1 结论：存在。`internal_vote.rs` 中 `is_internal_admin` 仍在 `#[cfg(test)]` 下回退读取 `CHINA_CB` / `CHINA_CH` 常量管理员。生产路径不受影响，但测试语义和生产 provider 语义分叉，建议修复。
- L2 结论：当前描述部分过期。治理机构合法性已是显式常量表判断；但 `ORG_DUOQIAN` 仍通过 `pass_threshold(...).is_some()` / `pending_pass_threshold(...).is_some()` 判断主体是否存在，语义仍耦合，建议后续加显式 `is_known_institution` / `is_known_subject` API。
- L3 结论：存在。`InternalThresholdProvider for ()` 仍返回治理机构固定阈值；生产 Runtime 已注入 `RuntimeInternalThresholdProvider`，但默认实现容易掩盖漏注入，建议改为 `None`。
- L4 结论：存在。`active_proposal_limit.rs` 的 `MAX_ACTIVE_PROPOSALS = 10` 和 `ActiveProposalsByInstitution` 的 `ConstU32` 上限仍是 pallet 内硬编码。建议改为 `Config::MaxActiveProposals`。
- L5 结论：当前不存在。`admins-change` 回调的否决路径已有中文注释“无独立存储需清理，ProposalData 由投票引擎延迟清理”，且返回 `ProposalExecutionOutcome::Executed`。无需修复。
- 本任务仅做只读分析，未修改代码，未执行测试。

## 完成信息

- 完成时间：2026-04-30 21:39:00
- 完成摘要：完成 L1-L5 只读核验；L1/L3/L4 建议修复，L2 建议语义收敛，L5 当前不成立。
