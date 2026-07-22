# 通用选举业务壳已退役删除

状态：已完成退役并由 `memory/08-tasks/20260721-citizen-election-subject-snapshot-unify.md` 取代。

## 任务需求

本任务曾建立无具体规则的通用选举业务壳。2026-07-21 最终确认改为：每一种公权选举业务都在 `runtime/public/` 下新增自己的业务模块，具体业务模块就是规则真源；`election-vote` 继续只负责投票流程。因此该通用业务壳没有改名或扩展，并已在后续任务第 7 步物理删除。

## 历史范围

- 新增 `citizenchain/runtime/public/election-campaign` crate。
- 将 `ElectionCampaign` 接入 runtime metadata。
- pallet 编号固定使用 `34`。
- 明确 `election-campaign` 与 `election-vote` 边界。
- 禁止外部直接调用 `ElectionVote::create_popular_election` 和 `ElectionVote::create_mutual_election`，避免绕过业务壳创建真实选举。

## 不做

- 不做法定代表人互选。
- 不做普选。
- 不生成候选人快照或选民快照。
- 不调用 `election-vote` 创建提案。
- 不写入 `public-admins`、`private-admins` 或法定代表人。
- 不实现选举法、同票、补选、递补、重选规则。

## 关闭结论

- 现有骨架没有真实可调用 extrinsic，也没有承载任何具体选举规则。
- crate、runtime 接线和错误文档已经物理删除；原 pallet index 32 永久保持空缺，不复用。
- 未来具体选举业务模块调用 `election-vote` 的内部接口创建提案，并在结果通过后自行复核和写回 entity 任职。
