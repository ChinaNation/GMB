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
- `public-manage` / `private-manage`：机构岗位与任职唯一真源，校验结果并派生 admins。
- `public-admins` / `private-admins`：机构管理员钱包账户集合，不保存岗位或任职来源。
- `citizen-identity`：投票身份、参选身份和人口统计唯一真源。

## 后续接入原则

真实选举功能必须从 `election-campaign` 创建业务活动，再由它调用 `election-vote` 创建投票提案。

`election-vote` 终态已经通过 runtime 结果路由接入 entity；当前缺口是本业务壳尚未实现“谁能按什么规则创建哪类选举”，不得通过开放底层创建 extrinsic 绕过该缺口。

外部不得直接调用 `ElectionVote::create_popular_election` 或 `ElectionVote::create_mutual_election` 绕过业务规则；外部只保留 `ElectionVote::cast_popular_vote` 和 `ElectionVote::cast_mutual_vote` 用于实际投票。
