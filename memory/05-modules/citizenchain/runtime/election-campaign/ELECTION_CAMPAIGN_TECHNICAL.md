# election-campaign 技术说明

## 定位

`election-campaign` 是公权选举业务模块。

它只负责选举业务规则的承载位置，例如什么机构可以组织某类选举、某个职位应走普选还是互选、候选人和选民快照如何生成，以及选举结果后续如何写回业务真源。

投票核心流程不属于本模块。创建投票提案、投票、去重、计票、超时结算、结果快照和账本清理统一归属 `election-vote`。

## 当前状态

当前版本只做 runtime 可见骨架：

- pallet index 固定为 `32`。
- 模块标识为 `b"ele-camp"`。
- 不开放真实选举创建入口。
- 不调用 `election-vote`。
- 不直接写入岗位任职、管理员集合或法定代表人。
- 活动元数据中的 `term_start`、`term_end` 统一使用自纪元起 `u32` 天，与 entity、`election-vote` 同单位。

## 边界

- `election-campaign`：业务规则、活动元数据、候选/选民快照来源和投票创建入口。
- `election-vote`：选举投票流程、投票记录、计票、终态结果快照、结果回调和清理。
- `public-manage` / `private-manage`：机构岗位与任职唯一真源，校验任职目标属于既有 admins，但不修改管理员集合。
- `public-admins` / `private-admins`：机构管理员姓名与钱包账户集合，不保存岗位或任职来源。
- `citizen-identity`：投票身份、参选身份和人口统计唯一真源。

## 后续接入原则

真实选举功能必须从 `election-campaign` 创建业务活动，再由它调用 `election-vote` 创建投票提案。

`election-vote` 只形成结果快照，不再通过 runtime 结果路由直写 entity。当前缺口是本业务壳尚未实现“谁能按什么规则创建哪类选举”以及“如何复核结果并形成任职业务结果”，因此创建与任职写入都保持关闭。

`ElectionVote::create_popular_election` 与 `ElectionVote::create_mutual_election` 外部调用已物理删除；外部只保留 `ElectionVote::cast_popular_vote` 和 `ElectionVote::cast_mutual_vote` 用于已经由合法业务活动创建的投票。

后续接入必须保证活动、投票提案和 entity 结果三者绑定同一个 `proposal_id`，并在业务层复核组织者、目标机构、职位、候选资格、席位数、任期和当选集合。不得把“投票通过”直接解释为“任职合法”。
