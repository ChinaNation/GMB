# 通用 election-campaign 骨架退役说明

## 定位

`election-campaign` 是开发期建立的无具体规则通用业务壳。2026-07-21 已确认删除，不改名、不扩展，也不作为未来选举规则真源。

未来每一种公权选举业务都在 `citizenchain/runtime/public/` 下新增自己的业务模块。具体业务模块定义谁能发起、选举本机构哪个 `role_code`、候选条件、选民范围、席位、任期、指定投票引擎和结果写回；业务模块本身就是规则。

投票核心流程不属于本模块。创建投票提案、投票、去重、计票、超时结算、结果快照和账本清理统一归属 `election-vote`。

## 当前状态

当前代码仍是等待第 7 步删除的 runtime 可见骨架：

- pallet index 固定为 `32`。
- 模块标识为 `b"ele-camp"`。
- 不开放真实选举创建入口。
- 不调用 `election-vote`。
- 不直接写入岗位任职、管理员集合或法定代表人。
- 活动元数据中的 `term_start`、`term_end` 统一使用自纪元起 `u32` 天，与 entity、`election-vote` 同单位。

## 最终边界

- `runtime/public/*-election`：每种具体选举的业务权限、岗位、候选条件、选民范围、席位、任期和结果写回。
- `election-vote`：选举投票流程、投票记录、计票、终态结果快照、结果回调和清理。
- `public-manage` / `private-manage`：机构岗位与任职唯一真源，校验任职目标属于既有 admins，但不修改管理员集合。
- `public-admins` / `private-admins`：机构管理员姓名与钱包账户集合，不保存岗位或任职来源。
- `citizen-identity`：投票身份、参选身份和人口统计唯一真源。

## 删除与后续接入原则

第 7 步删除本 crate、runtime Config/construct_runtime/Cargo 接线和错误文档，原 pallet index 保持空缺，本任务不复用。

真实选举功能必须从对应具体业务模块创建业务活动，再由它调用 `election-vote` 创建投票提案。

`election-vote` 只形成结果快照，不再通过 runtime 结果路由直写 entity。当前缺口是本业务壳尚未实现“谁能按什么规则创建哪类选举”以及“如何复核结果并形成任职业务结果”，因此创建与任职写入都保持关闭。

`ElectionVote::create_popular_election` 与 `ElectionVote::create_mutual_election` 外部调用已物理删除；外部只保留 `ElectionVote::cast_popular_vote` 和 `ElectionVote::cast_mutual_vote` 用于已经由合法业务活动创建的投票。

后续接入必须保证活动、投票提案和 entity 结果三者绑定同一个 `proposal_id`，并在业务层复核本机构发起主体、目标 `role_code`、候选资格、席位数、任期和当选集合。不得跨机构选举，也不得把“投票通过”直接解释为“任职合法”。
