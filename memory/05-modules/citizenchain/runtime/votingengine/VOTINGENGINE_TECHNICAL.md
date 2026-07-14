# votingengine 技术说明

## 定位

`votingengine` 是链上中国 runtime 的统一投票引擎。

业务模块只提交提案语义，不能自行实现投票流程、人口快照、投票资格、计票、通过判定或清理状态机。

## 内部投票与业务权限

- `internal-vote` 是所有机构与个人多签共用的管理员投票程序，负责内部投票模式准入、有效账户上下文、管理员快照、计票、阈值快照和终态。
- “机构可以使用内部投票”不等于“机构可以发起所有接入内部投票的业务”。有效准入 = 投票引擎模式准入 + 业务 pallet 具体权限，两层任一拒绝都不能创建或执行提案。
- `multisig` 转账允许所有 active 机构账户和个人多签账户；机构身份统一从 entity 生命周期真源解析，不维护 NRC/PRC/PRB 专用转账白名单。
- `resolution-destroy` 只允许 NRC、PRC、PRB；`grandpakey-change` 只允许 NRC、PRC。业务限制不得下沉到 `internal-vote`。
- FRG 是一个机构、一个主账户和 215 名管理员；省域 5 人岗位组属于注册业务权限，通用内部投票只校验 FRG 规范账户身份和管理员快照。

### 内部投票阈值

- NRC、PRC、PRB、NJD、FRG 使用代码级永久固定阈值，不写账户级动态阈值。
- PRS、NLG、NSN、NRP、NSP、NED 六个国家单例没有机构级动态阈值；普通内部事项在创建提案时按当前 admins 快照计算 `floor(N/2)+1`，只写 `InternalThresholdSnapshot`。
- 六个国家单例禁止注册、生命周期和管理员变更通用入口写入 `PendingDynamicThresholds`、`ActiveDynamicThresholds` 或待变更阈值。
- 普通注册机构与个人多签继续使用账户级动态阈值；生命周期关闭使用全员快照，具体业务权限仍由业务 pallet 校验。

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
- 联合业务回调必须同时绑定 `ProposalOwner`、联合 proposal kind、`STAGE_JOINT/STAGE_REFERENDUM`、业务摘要和对象摘要；联合阶段直接通过与转入公投后通过都必须执行同一项已绑定业务。

## 立法投票

- 人口快照：`LegislationVote::prepare_population_snapshot(scope)`。
- 代表机构表决：`cast_representative_vote(proposal_id, approve)`。
- 特别案公投：`cast_referendum_vote(proposal_id, approve)`。
- 行政签署、三人会签、护宪终审继续按账户和机构管理员快照判定。
- `legislation-yuan` 在创建提案和投票通过写入前分别复核一次法定路由：固定院序、发起机构、行政签署机构、会签机构、active 账户和 CID 行政区必须全部一致。客户端携带的路由字段不是授权真源。

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
- `election-vote` 只产生不可变当选结果快照，不解释职位、席位、任期或目标机构业务规则，也不得构造 `InstitutionGovernanceResult` 直写 entity。
- 普选/互选底层创建 extrinsic 已物理删除；当前外部只保留 `cast_popular_vote` 与 `cast_mutual_vote`。
- 真实创建必须由 `election-campaign` 校验组织者、职位、候选人、选民、席位和任期后调用引擎；结果也必须先回到该业务模块复核，再由业务模块调用 entity 任职入口。
- 当前 `election-campaign` 尚未实现真实规则，因此创建和结果写入都保持 fail-closed，不能用 RuntimeCallFilter 或 runtime 结果路由伪装成完整业务。

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

2026-07-13 第四步 B2 曾把 `election-vote` 结果直接封装为 entity 的通用
`InstitutionGovernanceResult`；该过渡实现缺少选举业务层复核，已在 2026-07-14 治理职责第 3 步撤销，
不得恢复为投票引擎直写任职。

2026-07-14 治理职责收口第 1 步：内部投票与业务权限完成分层。FRG 账户上下文不再错误绑定省域 5 人组；多签转账统一从 entity 解析所有机构；销毁仍由业务模块固定 NRC/PRC/PRB，GRANDPA 密钥仍固定 NRC/PRC。专项测试通过：`internal-vote` 88、`multisig` 24、`resolution-destroy` 15、`grandpakey-change` 17，runtime 整体 `cargo check` 通过。

2026-07-14 治理职责收口第 2 步：六个国家单例删除账户级动态阈值，普通内部事项改为按提案管理员快照派生严格过半；首次组成只原子写岗位、任职和 admins。`internal-vote` 89、`public-admins` 8、`public-manage` 42、runtime 集成 40 项测试通过。

2026-07-14 治理职责收口第 3 步：业务执行端新增 owner/kind/stage/code/account/CID/action 全绑定；联合业务接受联合阶段或公投阶段的合法通过终态；立法路由改为链端双重复校验；选举引擎删除外部创建入口和直写 entity 路径。投票引擎继续只负责投票流程，业务权限与执行前复核留在业务 pallet。
