# votingengine 技术说明

## 定位

`votingengine` 是链上中国 runtime 的统一投票引擎。

业务模块只提交提案语义，不能自行实现投票流程、人口快照、投票资格、计票、通过判定或清理状态机。

## 公民身份真源

公民投票和参选资格统一通过 `CitizenIdentityReader` 读取 `citizen-identity`：

- `can_vote(who, scope)`：判断账户在作用域内是否有投票资格。
- `can_be_candidate(who, scope)`：判断账户在作用域内是否有参选资格。
- `population_count(scope)`：读取链上人口分母。

OnChina 本地数据库只能用于注册局录入和界面提示，不能作为链上投票资格真源。

## 人口作用域

`PopulationScope` 支持四级：

- `Country`
- `Province(province_code)`
- `City(province_code, city_code)`
- `Town(province_code, city_code, town_code)`

联合公投和立法特别案在创建提案前先调用对应的 `prepare_*_population_snapshot(scope)`，runtime 在当前区块从 `citizen-identity` 读取人口分母并缓存到发起账户。

## 联合投票

- 内部阶段：`JointVote::cast_admin(proposal_id, institution, approve)`。
- 联合公投阶段：`JointVote::cast_referendum(proposal_id, approve)`。
- 公民投票按 `proposal_id + who` 去重。
- 公民资格由 `CitizenIdentityReader::can_vote(who, scope)` 判定。

## 立法投票

- 人口快照：`LegislationVote::prepare_population_snapshot(scope)`。
- 院内表决：`cast_house_vote(proposal_id, approve)`。
- 特别案公投：`cast_referendum_vote(proposal_id, approve)`。
- 行政签署、三人会签、护宪终审继续按账户和机构管理员快照判定。

## 清理

提案完成后统一进入投票引擎清理状态机，清理内部投票记录、联合投票记录、公民投票账户记录、提案对象和反向索引。

## 验收

- `cargo test -p votingengine`
- `cargo test -p joint-vote`
- `cargo test -p legislation-vote`
- `cargo test -p internal-vote`
