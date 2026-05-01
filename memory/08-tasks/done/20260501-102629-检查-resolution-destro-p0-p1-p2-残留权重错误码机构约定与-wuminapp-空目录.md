# 任务卡：检查 resolution-destro P0/P1/P2 残留权重、错误码、机构约定与 wuminapp 空目录

- 任务编号：20260501-102629
- 状态：done
- 所属模块：citizenchain/runtime/governance/resolution-destro, wuminapp
- 当前负责人：Codex
- 创建时间：2026-05-01 10:26:29

## 任务需求

只读检查用户列出的 7 项问题是否存在，评估影响，并给出推荐修复方案。

## 检查项

- P0：`resolution-destro` weights 是否引用已删除存储且 proof size 失真。
- P1：`DestroyVoteSubmitted` 是否为未触发事件残留。
- P1：`ProposalActionNotFound` 是否一码两义。
- P1：`institution_org` 是否依赖 `CHINA_CB[0]` 与 `CHINA_CB[1..]` 隐式顺序约定。
- P1：`wuminapp/lib/citizen/proposal/resolution_destroy/` 是否为空目录残留。
- P2：`execute_destroy` 是否缺少 `MODULE_TAG` 校验。
- P2：测试是否缺少非本模块前缀、机构归属不匹配、retry 失败透传三类用例。

## 输出物

- 存在性判断
- 影响评估
- 推荐修复方案

## 实施记录

- 任务卡已创建
- 2026-05-01 10:30：已核查 `resolution-destro` runtime、weights、测试、技术文档、voting-engine 重试路径，以及 wuminapp 提案展示分支。

## 检查结果

- P0 成立：`weights.rs` 仍引用 `ResolutionDestro::ActiveProposalByInstitution`、`ProposalActions`、`ProposalCreatedAt`、`ProposalPassedAt` 等已移除存储项，且 proof 注释/读写次数来自旧路径。
- P1 成立：`DestroyVoteSubmitted` 事件仅保留定义与文档记录，当前投票统一走 `VotingEngine::internal_vote`，本模块没有任何对应 `deposit_event`。
- P1 成立：`ProposalActionNotFound` 同时用于“投票引擎提案不存在”和“`DestroyAction` 解码失败”，错误语义混用。
- P1 成立：`institution_org` 用 `CHINA_CB.first()` 识别 NRC，用 `CHINA_CB.iter().skip(1)` 识别 PRC，依赖隐式顺序。
- P1 成立：`wuminapp/lib/citizen/proposal/resolution_destroy/` 仅有 `.gitkeep`，实际 `res-dst` 展示逻辑仍在 `transfer_proposal_service.dart`。
- P2 成立：`execute_destroy` 当前直接委托 `VotingEngine::retry_passed_proposal_for`，入口没有先确认 `ProposalData` 属于 `MODULE_TAG = b"res-dst"`。
- P2 成立：当前 12 个测试缺少非 `res-dst` 前缀返回 `Ignored`、`InstitutionOrgMismatch` 负向、`execute_destroy` retry 错误透传/模块不匹配三类用例。

## 推荐处理

- 先做小修复 PR：删除死事件、拆分错误码、给 `execute_destroy` 增加模块前缀预检、补三类测试、删除 wuminapp 空目录或立即完成拆分。
- 机构归属建议在 primitives 增加显式 NRC 标记或 `CHINA_CB_NRC_INDEX`/辅助函数，并在 `resolution-destro` 入口增加中文注释与 debug/assert 防线。
- `weights.rs` 不建议手工改数值；待上述代码稳定后，用带 benchmarking runtime api 的 runtime blob 重跑 benchmark，再作为 spec_version 升级前必清项合入。
