# 任务卡：清理管理员更换模块旧投票残留并修复 admins-change benchmark 中硬编码 proposal id 的问题，暂不实现 Pending 主体 API 重构

- 任务编号：20260430-115508
- 状态：done
- 所属模块：admins-change
- 当前负责人：Codex
- 创建时间：2026-04-30 11:55:08

## 任务需求

清理管理员更换模块旧投票残留并修复 admins-change benchmark 中硬编码 proposal id 的问题，暂不实现 Pending 主体 API 重构

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/MODULE_TAG_REGISTRY.md
- memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md

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
- 已修复 `admins-change` benchmark 中硬编码 `proposal_id = 0/1` 的旧假设，改为在创建提案后通过 `VotingEngine::next_proposal_id().saturating_sub(1)` 推导真实提案编号。
- 已调整 `execute_admin_replacement` benchmark 的状态模拟顺序：先制造自动执行失败的中间态，再把提案推到 `PASSED`，最后恢复数据并执行补救路径。
- 已清理管理员更换模块旧业务投票入口残留：移除 `AdminReplacementVoteSubmitted` 事件残留，避免文档继续暗示存在 `vote_admin_replacement` 专属投票入口。
- 已清理旧 benchmark 权重文件中的过期 storage proof 注释，保留保守临时权重；正式发布前仍应重新运行 benchmark CLI 生成精确权重。
- 已同步更新 admins-change 技术文档、治理 tag 注册表、wuminapp 治理技术文档。
- 已执行残留扫描，目标代码与相关技术文档中未再发现 `vote_admin_replacement`、`AdminReplacementVoteSubmitted`、`AdminReplacementProposal`、`ProposalActions`、`ActiveProposalByInstitution`、`ProposalCreatedAt`、`ProposalPassedAt` 残留。
- 验证通过：`cargo test -p admins-change --lib`。
- 验证通过：`cargo test -p admins-change --lib --features runtime-benchmarks`。

## 后续说明

- 本任务暂不实现 Pending 主体 API 语义重构；该问题应单独拆成 active-only 公共业务 API 与 pending+active 快照 API 的边界调整任务。
