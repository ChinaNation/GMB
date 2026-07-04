# election-campaign 接入 runtime 骨架

## 任务需求

用户确认 `election-campaign` 是选举业务模块，负责“什么机构能选举、怎么选举”等业务规则；`election-vote` 是选举投票模块，投票核心流程必须留在 `election-vote`。当前只做最简单骨架，不做具体选举。

## 范围

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

## 验收

- `ElectionCampaign` 以 pallet index `34` 出现在 runtime。
- `election-campaign` 无真实可调用 extrinsic。
- `ElectionVote` 创建入口被 RuntimeCallFilter 拦截，投票入口继续保留。
- 相关文档说明业务壳和投票模块的职责边界。
