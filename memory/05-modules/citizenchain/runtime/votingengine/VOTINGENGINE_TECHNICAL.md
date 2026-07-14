# votingengine 技术说明

## 定位

`votingengine` 是链上中国 runtime 的统一投票引擎。

业务模块只提交提案语义，不能自行实现投票流程、人口快照、投票资格、计票、通过判定或清理状态机。

## 公民身份真源

投票资格和参选资格统一通过 `CitizenIdentityReader` 读取 `citizen-identity`：

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
- 联合公投按 `proposal_id + who` 去重。
- 联合公投资格由 `CitizenIdentityReader::can_vote(who, scope)` 判定。

## 立法投票

- 人口快照：`LegislationVote::prepare_population_snapshot(scope)`。
- 代表机构表决：`cast_representative_vote(proposal_id, approve)`。
- 特别案公投：`cast_referendum_vote(proposal_id, approve)`。
- 行政签署、三人会签、护宪终审继续按账户和机构管理员快照判定。

### 固定框架

```text
legislation-vote/
├── representative/   # 单机构、顺序多机构和逐机构计票
├── legislation/      # 法律专属公投、签署、会签和护宪终审
├── types.rs          # 路线、数学规则、后续程序强类型
├── rules.rs          # 三类数学门槛唯一实现
├── result.rs         # ProposalOwner 业务结果路由边界
└── cleanup.rs        # 代表票据和法律票据清理边界
```

- `RepresentativeRoute::Single/Sequential` 只表达一个或多个代表机构的推进顺序，不把教育委员会误称为立法院一院。
- `RepresentativeVoteRule::Regular/Major/Special` 是投票引擎唯一数学规则；教育等业务分类不进入引擎规则枚举。
- `VoteProcedure::RepresentativeOnly` 表决完成即把结果交给任免、预算等业务；`Legislation` 才继续执行法律专属程序。
- `RepresentativeMetas` 与 `LegislationMetas` 分离；非法律业务不得写法律元数据。
- `RepresentativeTallies[proposal_id][body_index]` 独立保存每个机构计票。
- `RepresentativeVotesByAccount[proposal_id][(body_index, account)]` 按机构席位去重，同一钱包可在不同机构分别投票。
- 终局回调由各业务模块依据 `ProposalOwner/MODULE_TAG` 认领；投票引擎不解析法律、任免职书或预算正文。

## 选举投票

- `election-vote` 统一承载普选、互选的提案、选民/候选快照、投票、计票、结果快照和清理。
- `term_start`、`term_end` 使用自纪元起 `u32` 天，不使用区块高度表达法定任期。
- 普选终态映射为 `PopularElection`，互选终态映射为 `MutualElection`；每位当选人的任职都以 `proposal_id` 的 SCALE 编码写入 `assignment_source_ref`。
- `election-vote` 只把单个目标岗位的完整当选集合封装为 `InstitutionGovernanceResult`；runtime 只按机构码路由，entity 负责校验并从全部有效任职派生 admins 钱包集合。
- `election-vote` 不直接写 `InstitutionRoles`、`InstitutionRoleAssignments` 或 admins storage，不保存第二份岗位/管理员真源。
- 底层创建 extrinsic 仍由 RuntimeCallFilter 禁止；真实创建规则必须由 `election-campaign` 业务壳提供。

## 清理

提案完成后统一进入投票引擎清理状态机，清理内部投票记录、联合投票记录、联合公投记录、立法投票记录、选举投票记录、提案对象和反向索引。

## 验收

- `cargo test -p votingengine`
- `cargo test -p joint-vote`
- `cargo test -p legislation-vote`
- `cargo test -p internal-vote`
- `cargo test -p election-vote`

2026-07-13 第四步 B1 验收：runtime 37、`legislation-vote` 32、`legislation-yuan` 30、
OnChina 120 项测试全部通过；node、runtime `no_std` 和 OnChina 生产构建通过。当前源码 production
WASM 的 fresh 临时节点正常启动，block#0 为
`0xf5f7bb30535ead9b5cd5b0159b61124dd0116635ebe78b6b550eb3aa7dc169fe`；真实 metadata
已确认新代表机构存储与 `cast_representative_vote` 生效，被替换的旧存储和旧调用名不存在。

2026-07-13 第四步 B2：`election-vote` 已改用 entity 的通用 `InstitutionGovernanceResult`，
只提交单个目标岗位的完整当选集合；每位当选人的任期、来源和引用独立编码。该改造没有改变
选举提案、资格、计票或通过规则，也没有新增业务模块、外部调用或投票 kind。
