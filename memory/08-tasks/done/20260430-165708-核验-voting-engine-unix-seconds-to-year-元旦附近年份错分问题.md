# 任务卡：核验 voting-engine unix_seconds_to_year 元旦附近年份错分问题

- 任务编号：20260430-165708
- 状态：done
- 所属模块：citizenchain
- 当前负责人：Codex
- 创建时间：2026-04-30 16:57:08

## 任务需求

核验 voting-engine unix_seconds_to_year 元旦附近年份错分问题

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
- 2026-04-30 16:57：已完成只读核验。`allocate_proposal_id` 使用 `unix_seconds_to_year` 生成 `年份 × 1,000,000 + 年内计数器`，该年份会真实进入 `proposal_id`、`CurrentProposalYear`、`YearProposalCounter` 和 `NextProposalId`。
- 2026-04-30 16:57：原示例中 `2027-01-01 00:00:00 UTC` 算成 2026 的验算不成立；实际 `1798761600 / 31556952 = 57.000485...`，取整后年份为 2027。
- 2026-04-30 16:57：同类问题实际存在。平均年秒数算法不能对齐 UTC 公历年边界，例如 `2028-01-01 00:00:00 UTC` 会算成 2027，直到 2028 年边界阈值 `1830303216` 秒才切到 2028，错误窗口为 5616 秒（约 1 小时 33 分 36 秒）。
- 已完成代码修复：`unix_seconds_to_year` 改为按 UTC 公历年边界换算，使用真实闰年规则，不再使用 `365.2425 天` 平均年长整除。
- 已补充单测：覆盖 2027、2028、2029、2032 元旦边界、2000/2100/2400 闰年规则，以及 `allocate_proposal_id` 在 `2027-12-31 23:59:59 UTC` 到 `2028-01-01 00:00:00 UTC` 的年份计数器重置。
- 已更新文档：`memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md` 明确 Proposal ID 年份段必须按真实 UTC 公历年计算。
- 已清理残留：旧的“简化算法、不需要精确到天”和平均年秒数常量已删除。
- 已执行格式化：`cargo fmt --manifest-path citizenchain/Cargo.toml --package voting-engine`。
- 已执行测试：`cargo test --manifest-path citizenchain/Cargo.toml -p voting-engine --lib --offline`，结果 61/61 通过。

## 核验结论

- 问题存在，但不是“每年元旦附近都向上一年错分”，而是平均年边界与真实公历年边界发生正负漂移。
- 当算法边界晚于真实元旦时，元旦后一小段时间会继续使用上一年 ID 段。
- 当算法边界早于真实元旦时，上一年年末一小段时间会提前使用下一年 ID 段。
- 因为提案 ID 明确按年份分段，该问题属于 `voting-engine` 提案 ID 年份分段 bug。

## 建议修复方向

- 不再使用 `365.2425 天` 平均秒数整除计算年份。
- 在 runtime `no_std` 环境内实现确定性的 UTC 公历年换算，至少用闰年规则和逐年/常量算法把 Unix 秒数映射到真实 UTC 年份。
- 补充单测覆盖 2027、2028、2032 等正负漂移边界，并验证 `allocate_proposal_id` 在真实元旦边界按预期切换年份。
- 修复后同步更新 `memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md` 中 Proposal ID 年份分段说明。

## 完成信息

- 完成时间：2026-04-30 17:53:00
- 完成摘要：修复 voting-engine Proposal ID 年份换算：改为 UTC 公历年边界，补元旦与闰年单测，更新技术文档并通过 voting-engine lib 测试。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
