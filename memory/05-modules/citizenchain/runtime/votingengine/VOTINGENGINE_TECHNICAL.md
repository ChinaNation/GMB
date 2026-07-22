# votingengine 技术说明

## 定位

`votingengine` 是链上中国 runtime 的统一投票引擎。

业务模块只提交提案语义，不能自行实现投票流程、人口快照、投票资格、计票、通过判定或清理状态机。

ADR-039 已于 2026-07-19 冻结机构岗位主体目标。任务卡第 5A、5B、5C 已依次完成联合、内部、立法和选举投票迁移；全部机构 Track 都按 `VotePlan` 中的完整岗位主体冻结资格，不再以 CID 全体 admins 作为发起或投票资格。个人多签仍使用独立管理员主体。

## 内部投票与业务权限

- `internal-vote` 是机构岗位主体与个人多签共用的投票程序；两类授权主体必须用 `AuthorizationSubject` 强类型分离。
- 共享 `VotePlan` 的 SCALE 字段顺序固定为 `business_action_id`、`proposal_owner`、`proposer_subject`、`voter_subjects`、`voting_engine`、`business_object_hash`；最多绑定 256 个投票主体。
- `VotingEngineKind` discriminant 固定为 `Internal = 0`、`Joint = 1`、`Election = 2`、`Legislation = 3`。构造器强制 owner 与 module tag 相同、投票主体不重复，并禁止机构岗位与个人多签主体混用；只有 Popular 选举允许 voter subjects 为空，因为其选民资格来自人口快照，其他引擎一律要求非空岗位/个人主体。
- 机构内部投票和联合投票创建时都必须携带完整 `VotePlan`；核心引擎一次性写入 `ProposalVotePlans`，要求 `ProposalOwner` 与 plan 相同，拒绝重复绑定。个人多签使用独立 personal 创建接口和 `AdminSnapshot`，不得伪造机构岗位主体。
- 目标机构提案由业务模块先校验 `RoleSubject(cid_number, role_code)` 的 `Propose` 权限，并静态选择唯一投票引擎、绑定 `VotePlan`。引擎不得接受调用方选择引擎，也不得把“属于 admins”当业务准入。
- 机构投票资格按 VotePlan 中一个或多个 voter `RoleSubject` 的有效任职账户建立不可变快照；不得自动快照该 CID 的全体 admins。个人多签仍按 personal_account 的管理员集合快照。
- `VoterSnapshot[(proposal_id, RoleSubject)]` 保存每个岗位主体的有效任职账户；`InstitutionTicketCountSnapshot[(proposal_id, cid_number)]` 冻结该机构岗位席位票据总数。同一钱包兼任多岗时按 `RoleSubject + admin_account` 分别行使各岗位票权，不按钱包合并，也不改变机构阈值。
- 投票引擎只负责快照、资格、票据、阈值、计票、终态、重试和清理；业务合法性、业务对象绑定及通过后的具体执行归对应业务 pallet。
- `multisig` 转账允许已登记机构账户和个人多签账户；机构调用显式携带 `actor_cid_number + proposer_role_code + institution_account`，反向索引只校验账户归属，业务模块再校验完整岗位主体权限。个人多签必须同时携带 `actor_cid_number=None`、`proposer_role_code=None`。
- `resolution-destroy` 只允许 NRC、PRC、PRB 对应固定岗位；`grandpakey-change` 只允许 NRC、PRC 委员岗位。业务限制和业务动作权限不得下沉到 `internal-vote`。
- FRG 是一个 CID 机构并拥有多个协议账户和 215 名管理员；省域 5 人岗位组属于注册业务权限，目标投票和发起资格必须解析对应省专员 `RoleSubject`，不能把 FRG 全体 admins 纳入。
- 协议升级与决议发行都固定使用联合投票：NRC/43 个 PRC 委员岗位可发起和投票，43 个 PRB `DIRECTOR / 董事` 岗位只投票；岗位只提供资格主体，不能选择引擎或自带岗位阈值。
- 联合投票引擎还会校验 proposer 是 plan 中 NRC/PRC 委员主体的有效任职账户，并要求 plan 精确覆盖 44 个 CHINA_CB 委员主体和 43 个 CHINA_CH 董事主体；缺失、多出、重复或跨类型主体均 fail-closed。

### 机构阈值与提案快照

- 岗位没有阈值 storage，也不得把岗位数或 admins 钱包数误作阈值；`VotePlan` 只决定哪些岗位任职席位进入快照。
- public/private entity 的 `InstitutionGovernanceThresholds[cid_number]` 是机构治理阈值真源。`internal-vote` 在建案事务中通过 runtime provider 读取并写入 `InternalThresholdSnapshot[proposal_id]`；建案后机构配置变化不得改变既有提案阈值。
- 投票引擎不再保存机构 Active/Pending 阈值表，也不通过管理员更换隐式改阈值。个人多签继续使用 `ActivePersonalThresholds[personal_account]`，不受机构阈值重构影响。
- 固定创世机构的阈值由 genesis 写入 entity，不是岗位阈值。公民链基金会只有一个程伟管理员钱包，但三个固定岗位各一席、机构阈值仍为 2；同一钱包的三项岗位任职已形成三张独立岗位票据，并通过内部投票阈值验收。

## 公民身份真源

投票资格和参选资格统一通过 `CitizenIdentityReader` 读取 `citizen-identity`：

- `voting_subject(who, scope)`：账户在当前作用域内有投票资格时返回完整 `CitizenSubject`。
- `candidate_subject(who, scope)`：账户在当前作用域内有参选资格时返回完整 `CitizenSubject`。
- `population_data(scope)`：人口日期完整就绪时返回作用域、人口分母、资格 revision 和
  护照判定日期；日期尚未推进完成或身份模块处于人口维护故障时返回 `None`。该数据只能
  由 `citizen-identity` 产生，裸 `population_count()` 接口已经删除。
- `voting_subject_at(who, population_data)`：按人口数据中的 revision 和日期查询永久 CID 身份历史，并返回 CID + 当前签名钱包的完整主体。

`citizen-identity` 是人口数据和身份历史唯一真源，但不生成、不编号、不保存提案快照。
`votingengine::create_population_snapshot(proposal_id, scope)` 遇到 `None` 返回
`PopulationDataNotReady`，不写入任何快照；取得完整数据后才写入
`ProposalPopulationSnapshots[proposal_id]`。投票、分母读取和 90 天清理都只围绕该
proposal_id 快照进行。

OnChina 本地数据库只能用于注册局录入和界面提示，不能作为链上投票资格真源。

2026-07-22 资格读取已统一返回 `CitizenSubject { cid_number, wallet_account }`。联合公投、立法公投和 Popular 选举均以 `(proposal_id, cid_number)` 唯一去重，票据值保存完整主体；同一永久 CID 更换钱包后不能再投一票。候选快照、候选计票和当选结果也已统一为完整主体。人口快照仍只保存作用域、有效总数、资格 revision、判定日期和创建区块，不枚举公民名单。

## 人口作用域

`PopulationScope` 支持四级：

- `Country`
- `Province(province_code)`
- `City(province_code, city_code)`
- `Town(province_code, city_code, town_code)`

联合公投和立法特别案在创建提案的同一存储事务内调用
`votingengine::create_population_snapshot(proposal_id, scope)`；引擎从 `citizen-identity`
读取人口数据并在自身 storage 形成不可变提案快照。联合公投固定使用
全国作用域；立法特别案只从 `actor_cid_number` 的合法 CID 和法定机构码推导国家、
省或市作用域。任一后续写入失败时快照与提案一起回滚，不存在公开准备入口、待消费
快照、客户端作用域参数、独立 snapshot_id 或第二人口真源。

## 联合投票

- 内部阶段：`JointVote::cast_admin(proposal_id, institution, approve)`。
- 联合公投阶段：`JointVote::cast_referendum(proposal_id, approve)`。
- 联合公投按 `proposal_id + cid_number` 去重，`CitizenReferendumTicket` 保存完整公民主体和票值。
- 联合公投资格由 `CitizenIdentityReader::voting_subject_at(who, population_data)` 返回；提案创建后新增、迁居或被撤销的当前身份均不能改变已有提案的成员集合。
- 公投分母与成员资格来自同一个投票引擎提案快照，快照数据只由 `citizen-identity` 提供；累计票数达到该分母后拒绝继续写票，参与率不得超过 100%。
- 联合业务回调必须同时绑定 `ProposalOwner`、联合 proposal kind、`STAGE_JOINT/STAGE_REFERENDUM`、业务摘要和对象摘要；联合阶段直接通过与转入公投后通过都必须执行同一项已绑定业务。

## 判定与业务执行

- 投票门槛一旦命中，只提交 `STATUS_PASSED`、释放相应活跃名额并写入 `PendingProposalExecutions`；最后一票不再同步执行转账、销毁或 `set_code` 等业务。
- `on_initialize` 按 `MaxExecutionWeightPerBlock` 与 `MaxAutoFinalizePerBlock` 双重上限消费执行队列，每项按包含 `SystemWeightInfo::set_code()` 的最重成本预留。
- 业务回调返回 `DispatchError` 时只回滚本次执行尝试，不撤销已成立的投票结果；失败按指数退避重试，达到上限转 `STATUS_EXECUTION_FAILED` 并发出 dead-letter 事件。
- 回调 `Err`、`Ignored`、结果应用错误和 Track 后处理错误统一进入同一失败处理器；每次递增 attempts，孤儿或状态不匹配的队列项立即删除。
- 达到自动执行上限后，业务执行队列永久停止；`PendingTerminalFinalizations` 只补终态副作用并拥有独立退避/dead-letter，绝不重新调用业务执行回调。
- `ProposalExecutionOutcome::RetryableFailed` 继续进入手动重试宽限期；机构提案只有创建时任一 `VoterSnapshot` 中的有效岗位选民可重试/取消，个人多签才读取 `AdminSnapshot`。执行成功或失败终态仍统一触发互斥释放、业务终态通知和 90 天延迟清理。
- 自动超时 finalizer 自身返回错误时使用独立有限退避状态；达到上限或重试桶已满后写入 `AutoFinalizeDeadLetters`，不会反复阻塞同一区块的其余维护管线，公开 `finalize_proposal` 仍可在修复数据后人工恢复。

## 立法投票

- 人口快照：仅特别案在创建事务内按 `actor_cid_number` 内联创建并绑定；普通案和
  重大案不创建。
- 代表机构表决：`cast_representative_vote(proposal_id, approve)`。
- 特别案公投：`cast_referendum_vote(proposal_id, approve)`。
- `legislation-yuan` 的新法、修法、废法动作分别固定为 `leg-yuan/0,1,2`；发起调用必须显式携带 `proposer_role_code`，业务模块按 `actor CID + 岗位码 + action + Propose` 校验后固定选择立法投票引擎。
- 当前创世只强制创建各机构唯一 LR；NRP/NSN/NED/PRP/PSN/CLEG/CEDU/CSLF 的成员岗位不在创世预造，必须由机构以后依法创建。建案时业务模块从链上岗位权限中解析准确代表岗位并写入 VotePlan。
- 行政签署和国家/省级三人会签固定使用相关机构 LR，护宪终审固定使用 NJD `CONSTITUTION_GUARD`；全部按建案时岗位任职快照判定，不按机构全体管理员快照判定。护宪岗位不是 LR，且必须在建案时恰好冻结 7 个不重复账户。
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
- `RepresentativeVotesByTicket[proposal_id][(body_index, InstitutionVoteTicket)]` 按完整机构岗位席位去重，同一钱包可在不同代表岗位分别投票。
- 终局回调由各业务模块依据 `ProposalOwner/MODULE_TAG` 认领；投票引擎不解析法律、任免职书或预算正文。

## 选举投票

- `election-vote` 统一承载普选、互选的提案、选民/候选快照、投票、计票、结果快照和清理。
- `term_start`、`term_end` 使用自纪元起 `u32` 天，不使用区块高度表达法定任期。
- `election-vote` 只产生不可变当选结果快照，不解释职位、席位、任期或目标机构业务规则，也不得构造 `InstitutionGovernanceResult` 直写 entity。
- 普选/互选底层创建 extrinsic 已物理删除；当前外部只保留 `cast_popular_vote` 与 `cast_mutual_vote`。
- 真实创建必须由 `runtime/public/` 下对应的具体选举业务模块校验本机构发起岗位、目标 `role_code`、候选人、选民范围、席位和任期后调用引擎；结果也必须先回到原具体业务模块复核，再由业务模块调用 entity 任职入口。
- 无具体规则的开发期通用选举业务壳已经删除，原 pallet index 32 永久留空；不得恢复、改名或扩展成所有选举规则的集中模块。具体业务模块本身就是该类选举的规则真源。
- 机构只能发起本机构岗位选举。最终元数据只保留 `actor_cid_number + role_code`，发起岗位、互选岗位和被选举岗位的 CID 必须相同。
- 提案实例只使用投票引擎生成的全链唯一 `proposal_id`，业务类型由 `BusinessActionId` 表达；不得保留无权威规则表支撑的通用规则编号。

### 资格真源与快照边界

- 普选必须使用 `citizen-identity` 的 `PopulationScope`、完整公民主体资格、`population_data` 和快照时资格查询；投票引擎按 proposal_id 保存不可变人口数据，不接收、不枚举、不保存全国/省/市/镇完整选民列表。
- 互选属于机构岗位业务；目标选民必须来自业务模块 VotePlan 指定的 voter `RoleSubject` 有效任职快照。调用方不得提交或删减选民集合。
- `election-vote` 创建入口按 `ElectionMode` 强制检查资格来源：Popular 必须有人口作用域，Mutual 必须取得已绑定岗位主体快照。
- 普选人口作用域写入 `ElectionMeta`，完整人口数据写入核心 `ProposalPopulationSnapshots`；互选不写公民作用域，按 VotePlan 中属于唯一 `actor_cid_number` 的一个或多个 `RoleSubject` 写入核心 `VoterSnapshot`，并以 `MutualElectionVotesByTicket` 保存完整岗位票据。`MutualVoters`、调用方选民参数和 `MaxMutualVoters` 已删除。
- 多席位计票允许完整落入剩余席位的并列组共同当选；并列组跨越席位边界时拒绝结果。
- 候选快照、Popular 票据、候选计票和当选结果全部使用 `CitizenSubject`；钱包只提供签名身份，CID 只提供公民身份，二者缺一不可。

## 清理

提案否决、超时、执行成功或执行失败终态都统一进入投票引擎维护管线。引擎先调用
所属业务 Track 的终态回调清除业务 pending 锁，再按保留期调度清理投票记录、人口
快照、提案对象和反向索引；业务 pallet 不暴露人工拒绝清理交易。

- `ScheduledCleanups + ScheduledCleanupHead/Tail` 是 90 天保留期的延迟 FIFO；固定保留期保证写入顺序就是到期顺序，不再使用有界区块桶或向后扫描候选桶。
- 到期任务转入 `PendingCleanupQueue + PendingCleanupQueueHead/Tail` 就绪 FIFO；每个提案每轮只执行一个有界步骤，未完成任务排回队尾。
- 清理阶段固定为 `AdminSnapshots → VoterSnapshots → InstitutionTicketCounts → TrackData → ProposalObject → FinalCleanup`；`TrackData` 只派发到提案所属 Track，不再空扫四类 sub-pallet。`FinalCleanup` 同步删除 `ProposalVotePlans`。
- 激活数、清理步骤数和 `MaxCleanupWeightPerBlock` 同时限流；单个大型公投不能阻塞后续提案，也不能挤占自动终结或业务执行的独立预算。

## 生产代码职责边界

- `votingengine/src/lib.rs` 只保留 pallet 配置、存储、事件、错误、hooks 和 lifecycle extrinsic。
- `expiry.rs` 承载到期索引、自动终结、有限退避和 dead-letter。
- `execution.rs` 承载异步业务执行、重试期限与管理员恢复入口。
- `lifecycle.rs` 承载状态迁移、终态副作用、回调作用域和统一事件。
- `maintenance.rs` 承载有界票据清理和提案对象清理步骤。
- `tracks.rs` 定义单 Track 生命周期接口和递归 tuple 派发；核心不匹配具体 mode/stage。
- `traits.rs` 只作为稳定 re-export 门面；实际 trait 按 engines、providers、callbacks、finalizers、cleanup 分组。
- `internal-vote` 的 proposal、threshold、vote、cleanup 各自独立；`legislation-vote` 的代表表决、公投、签署、护宪、结果和清理实现均位于对应真实模块。
- 生产源码单文件不得超过 800 行，不得用纯注释文件或空实现伪造职责边界。

## 2026-07-14 第一步安全修复

- 立法签署、三人会签和护宪终审的公开超时入口统一要求当前区块严格大于提案截止区块，防止任意账户提前终结提案。
- 自动到期结算失败不再回插原到期桶；第三步已补充独立计数、指数退避和 dead-letter，确定性错误不会永久阻塞执行重试和历史清理管线。
- 立法特别案公投统一调用 `primitives::constitution::referendum_passed`，不再保留重复数学实现。
- 注册多签待激活动态阈值按 `proposal_id` 隔离，避免同机构并发注册提案互相覆盖。

## 2026-07-14 第三步安全收口

- `on_initialize` 在到期桶尚未排空时不再提前返回，执行重试、终态清理和 90 天清理管线每块都能继续获得有界处理机会。
- 投票判定与业务执行已通过 `PendingProposalExecutions` 解耦；队列按 weight 预算执行，错误指数退避并在三次失败后 dead-letter，既有手动重试和终态清理契约已完成适配。
- `finalize_proposal` 只承担投票判定与执行入队，不再叠加 `set_code`；`set_code` 最重成本只归入 `process_pending_execution` 异步执行预算。五个投票 crate 的正式 benchmark 已生成并写入生产权重。
- `joint-vote` 本 crate 现有 12 项直属测试，除原有 `cast_admin`、`cast_referendum`、105 票全票、机构否决和超时转公投外，直接覆盖 `VotePlan`/岗位快照绑定、同 CID 去重和跨 CID 独立投票；`internal-vote` 继续提供跨 pallet 回归覆盖。
- `legislation-vote` 的 signing、guard、referendum、result、cleanup 文件已承载实际规则或清理辅助，不再是纯注释残桩。
- 该步骤当时曾把五个投票 pallet 及触达模块设为开发期 `StorageVersion = 1`；2026-07-21 最终创世决策已取代该版本口径，当前五个投票 pallet 及全部项目 pallet storage version 已统一为 `0`，不保留开发期 migration。

## 2026-07-14 Track 与维护调度收口

- Runtime 以 `(InternalVote, (JointVote, (LegislationVote, (ElectionVote, ()))))` 注册递归 Track tuple；手动超时、自动超时、模式清理和内部阈值副作用走同一类型路由。
- 自动终结、异步业务执行、历史清理分别使用 `MaxAutoFinalizeWeightPerBlock`、`MaxExecutionWeightPerBlock`、`MaxCleanupWeightPerBlock` 独立预算；生产配置当前分别为最大区块权重的 `1/4`、`1/4`、`1/8`。
- 延迟 FIFO 与就绪 FIFO 均以单调 `u64` 序号键控，不存在单区块桶容量、顺延窗口或严格头部反复处理。
- 新增公平轮转测试和 Track 隔离清理测试：大任务存在时小任务仍推进，内部提案清理不会删除联合投票账本。

## 2026-07-14 资格快照与执行重试加固

- `citizen-identity` 以全局 `eligibility_revision` 和每账户不可变版本历史冻结创建时资格；同一区块多次身份写入也能确定顺序，账户查询按版本数二分定位。
- 联合公投、立法公投和 Popular 选举统一写入 `ProposalPopulationSnapshots[proposal_id]`；`citizen-identity::PopulationSnapshots/NextSnapshotId`、核心 `ProposalPopulationSnapshotIds`、`Proposal.citizen_eligible_total`、`ReferendumScopes`、`LegislationMeta.referendum_scope`、Popular 全量选民表及 `MaxElectionVoters` 已删除。
- Popular 不受完整选区人数的 `BoundedVec` 限制；Mutual 只使用核心岗位快照，`MutualVoters` 和 `MaxMutualVoters` 已删除。
- `joint-vote` crate 直属测试当前为 12 项，直接覆盖 `cast_admin`、`cast_referendum`、105 票全票执行、机构否决转公投、超时转公投、创建后新增选民拒绝、岗位快照绑定和有效选民去重。
- 自动执行结果应用阶段的确定性错误不再每块无限重排；R2 回归用例覆盖 `Ignored` 三次退避后 dead-letter，以及孤儿执行队列立即删除。

## 2026-07-14 生产 Benchmark 与动态权重

- 目标环境：Apple M5 Pro / arm64、Rust 1.94.0、FRAME Benchmark CLI 53.0.0；WASM compiled，`steps=50`，`repeat=20`。
- Runtime registry 共注册 19 条：核心 4、内部 2、联合 5、立法 6、选举 2。
- 核心权重为 35/24/10/22 ms，其中公开终结 35 ms 是保守调度包络并另叠加实际 Track 权重；`process_pending_execution` 另显式叠加 `SystemWeightInfo::set_code()`，runtime 升级不再被普通回调静态权重掩盖。
- 内部权重沿用既有实测；联合和立法已按资格历史存储重新生成，公民公投写票均真实计入 snapshot 绑定、快照元数据和账户资格版本读取。
- 选举最后一票按候选人数 `c` 线性计费：Popular 基础约 `47.6m ps`、Mutual 基础约 `38.7m ps`，两者每候选人约增加 `1.56m ps`；Popular 读为 `10+c`，Mutual 读为 `7+c`，写均为 9。
- joint 人口准备和立法签署类 benchmark 无法在通用基准创世态完整构造生产 provider 权限，调用注解使用“实测主体与生产 provider 保守上界取 max”，重生生成文件不会移除安全上界。
- `ProposalTrackHandler` 同时返回 stage timeout、Track chunk cleanup、Track terminal cleanup 权重；手动终结、自动终结和清理维护均按具体 Track 实际值计账。
- 三条维护预算维持最大区块权重的 `1/4`、`1/4`、`1/8`，合计 62.5%；在 60 秒最大计算区块下，每条管线均可容纳至少一个最重任务。

### 覆盖率口径

- 原生 LLVM coverage 排除测试、benchmark、weights 和纯声明 `traits/types/data` 后，可执行业务代码共 4,324 行，命中 3,537 行，行覆盖率 81.80%。
- 若把纯接口与类型声明也计入，五个投票 crate 全源码为 71.60%。文档同时保留两项，80% 门禁只使用可执行业务代码口径。
- election-vote 现有测试文件内建立完整 mock runtime，覆盖普选/互选创建、人口/岗位快照、人口未就绪原子回滚、资格拒绝、写票、超时、结果回调与分块清理；当前为 14 项。

## 2026-07-19 第 5B 岗位主体内部投票收口

- `InternalVoteEngine` 的机构创建接口强制接收业务模块构造的 `VotePlan`；创建事务按 plan 中每个 `RoleSubject` 读取有效任职、写入 `VoterSnapshot`，并按 CID 累加 `InstitutionTicketCountSnapshot`。机构路径不再写 `AdminSnapshot`。
- `internal-vote::cast` 的机构调用必须显式声明 `voter_role_code`，以 `InternalVoteTicket::Institution(RoleSubject + account)` 校验和防双投；个人多签使用 `InternalVoteTicket::Personal(account)` 并读取 `AdminSnapshot`。两类主体没有兼容回落。
- 核心投票引擎不再暴露把机构人员名册写入个人多签快照的入口。机构 `admins` 查询只允许业务模块和 entity 用于确认人员名册归属，不能生成投票资格。
- votingengine Config、runtime 接线和测试 runtime 不再存在机构管理员人数 provider。机构阈值只由 entity 提供，岗位席位数只来自提案冻结的票据快照；不得恢复通过管理员人数推导机构计票的第二路径。
- 机构阈值继续来自机构固定阈值或机构动态阈值；岗位只决定选民集合，不新增岗位阈值。创建时若机构阈值无法由本次有效岗位快照达到，整笔提案回滚。
- 手动重试和取消同样按提案主体分流：机构读取有效岗位选民快照，个人多签读取个人管理员快照。
- 已接入业务为 public/private 本机构治理与关闭、决议销毁、GRANDPA 密钥更换、机构普通转账、NRC 安全基金转账、费用账户划转主账户和公民链基金会平台调价。每个业务自己校验 `RoleSubject + BusinessActionId + Propose`、枚举拥有 `Vote` 权限的岗位并固定使用内部投票引擎。
- 正式 FRAME benchmark 使用当前源码导出的临时 `citizenchain-fresh` spec、50 steps / 20 repeats。`resolution-destroy` 为 25 reads / 23 writes，`grandpakey-change` 为 25/23，`multisig::propose_transfer` 为 31/23；`internal-vote` 与核心 `votingengine` 已按机构有效岗位快照路径重算。public/private 完整凭证治理与 square 调价尚无可执行全调用夹具，生产权重使用 400 ms、700 KB proof、35 reads / 30 writes 的显式保守上界。
- `scripts/benchmark.sh` 每次从当前 benchmark 二进制导出一次性 fresh spec，退出即删除；不再用与当前 storage 布局不一致的冻结 spec 或裸 WASM 空创世态。

## 验收

- `cargo test -p votingengine`
- `cargo test -p joint-vote`
- `cargo test -p legislation-vote`
- `cargo test -p internal-vote`
- `cargo test -p election-vote`
- `cargo test -p citizenchain`
- `cargo check -p citizenchain --features runtime-benchmarks`
- `cargo check -p citizenchain --features try-runtime`

2026-07-14 执行重试与资格快照最终验收：`citizen-identity` 23、`internal-vote` 96、`joint-vote` 10、`legislation-vote` 33、`election-vote` 13、runtime 40 项专项测试及 `cargo test --workspace` 全部通过；六个相关 crate 的 `no_std`、runtime benchmark/try-runtime 编译和最终 release WASM 构建通过。当前源码以 `citizenchain-fresh --tmp` 真实启动成功，genesis hash 为 `0xd81962210c603a4a0f078b2cc022bac3daab344cd7dce8c6fc3501973d1552ab`，`isSyncing=false`，metadata RPC 响应 418,806 字节，runtime `specVersion/systemVersion/stateVersion` 均为 1；验收节点已停止。

2026-07-14 第 3 步最终运行态：`WASM_BUILD_FROM_SOURCE=1` release 构建通过；当前源码 WASM 以全新 base path 启动隔离 `citizenchain-fresh` 节点，NodeGuard 与创世装载通过。block#0/genesis hash 为 `0x8d3fc4c4567796d8056e61a8dbf431f04230126a1023a49ffecde7b5bff25390`，state root 为 `0x51ef488b720c9f049c501367f31e3779dd7a3711c295ce8cc79ddbe7688413ca`，runtime `specVersion=1`，`system_health.isSyncing=false`，metadata RPC 响应 415,442 字节；验收节点已停止。fresh 链无交易且无同 genesis peer 时按“空块不提交 + 离线不挖矿”规则保持 block 0。

2026-07-14 结构收口第 1 步：四个超限生产文件已完成物理拆分，核心 `lib.rs` 785 行、traits 门面 16 行、internal `lib.rs` 496 行、legislation `lib.rs` 572 行；五个投票 crate、runtime 40 项测试和五 crate `no_std` 构建通过。当前源码 `citizenchain-fresh --tmp` 真实启动成功，block#0 为 `0x15b19408800b8ab685b49e8076f861ed76b4713abea54a216a7be2dc0cee41ea`，`isSyncing=false`，验收节点已停止。

2026-07-14 结构收口第 2 步：Track tuple、两级公平清理 FIFO、三条独立 weight 预算和五 pallet `StorageVersion = 1` 最终创世布局已落地。全工作区测试目标检查通过；personal-manage 23、internal 94、joint 3、legislation 33、election 3、runtime 40 项测试通过；五 crate `no_std` 与 runtime 普通/benchmark/try-runtime 构建通过。当前源码 fresh 节点 block#0 为 `0xf20b42ad98756fa464678ab2473abc6f0be089dceae290c587cea80c1ead9ab1`，`isSyncing=false`，metadata RPC 响应 415,442 字节，验收节点已停止。

结构、Track、公平维护、正式 benchmark、覆盖率和 fresh genesis 三步门禁均已完成。正式创世发布仍需统一烘焙冻结 chainspec 并切换同 genesis bootnode；本任务未修改冻结 chainspec、未推送、未部署。

2026-07-13 第四步 B1 验收：runtime 37、`legislation-vote` 32、`legislation-yuan` 30、
OnChina 120 项测试全部通过；node、runtime `no_std` 和 OnChina 生产构建通过。当前源码 production
WASM 的 fresh 临时节点正常启动，block#0 为
`0xf5f7bb30535ead9b5cd5b0159b61124dd0116635ebe78b6b550eb3aa7dc169fe`；真实 metadata
已确认新代表机构存储与 `cast_representative_vote` 生效，被替换的旧存储和旧调用名不存在。

2026-07-13 第四步 B2 曾把 `election-vote` 结果直接封装为 entity 的通用
`InstitutionGovernanceResult`；该过渡实现缺少选举业务层复核，已在 2026-07-14 治理职责第 3 步撤销，
不得恢复为投票引擎直写任职。

2026-07-14 治理职责收口第 1 步：内部投票与业务权限完成分层。FRG 账户上下文不再错误绑定省域 5 人组；多签转账统一从 entity 解析所有机构；销毁仍由业务模块固定 NRC/PRC/PRB，GRANDPA 密钥仍固定 NRC/PRC。专项测试通过：`internal-vote` 88、`multisig` 24、`resolution-destroy` 15、`grandpakey-change` 17，runtime 整体 `cargo check` 通过。

2026-07-17 的“普通内部事项按提案管理员快照派生严格过半”口径已被 ADR-039 取代；目标按 VotePlan 指定岗位主体的有效任职快照派生。组成结果仍只写岗位和任职，绝不派生 admins。

2026-07-14 治理职责收口第 3 步：业务执行端新增 owner/kind/stage/code/account/CID/action 全绑定；联合业务接受联合阶段或公投阶段的合法通过终态；立法路由改为链端双重复校验；选举引擎删除外部创建入口和直写 entity 路径。投票引擎继续只负责投票流程，业务权限与执行前复核留在业务 pallet。

2026-07-22 公民主体快照接口最终验收：`voting_subject`、`candidate_subject`、`voting_subject_at` 全部返回完整 `CitizenSubject`；联合公投与立法公投按永久 CID 去重，票据值与事件保存 CID + 当前签名钱包，钱包更换不能形成第二票。候选、Popular 票据、候选计票和结果仍是第 6 步边界。五个投票 crate、runtime 46 项及受影响业务模块测试，全 workspace 测试目标，`no_std`、WASM、benchmark/try-runtime 和 release Node 构建均通过。真实 fresh 节点 block #0 为 `0x69b4a0025356d050004cff3ef176167a6520b59c9086c9ac6b9a45c4b9e9c0e6`，state root 为 `0x0b066c3567ed25c15cfa96b7d249b6235df4746a253144db21c87dfd2ed2333e`，metadata 二进制 220,197 字节，runtime 六项项目版本均为 `0`；节点已停止。

2026-07-22 选举投票模型最终验收：`ElectionMeta` 只保留唯一机构 `actor_cid_number + role_code`；候选快照、普选票据、候选计票和当选结果全部使用完整 `CitizenSubject`。普选按永久 CID 去重，互选继续按机构 CID + 岗位码 + 钱包票据去重，同一管理员可按多个不同岗位各投一票。`election-vote` 17 项、votingengine 4 项、runtime 46 项、全 workspace 测试目标、`no_std`、WASM、benchmark/try-runtime 和 release Node 构建通过。真实 fresh 节点 block #0 为 `0x285ca7f4ab0f24771baff6a6fc10141ee281fbbd6ce1a8f9dcd1d7676501a41b`，state root 为 `0x27ecdc5b73ce195df4bdfe6c05fe68ef0b682c58751f5f145868a69a1f4672bd`，metadata 二进制 220,398 字节，runtime 六项项目版本均为 `0`；节点已停止。

2026-07-22 全端协议验收：QR 唯一注册表已登记 `ElectionVote.cast_popular_vote = 0x1602` 和 `cast_mutual_vote = 0x1603`；CitizenWallet 严格解码提案编号、完整候选 `CitizenSubject`，互选另解码选民岗位码，旧裸钱包载荷和任何截断、尾随载荷均拒签。CitizenApp 不直接创建选举，也不开放不存在的通用选举业务入口；未来只能由具体公权选举业务模块创建并绑定本引擎。第 8 步未修改 runtime，正式重算权重和 fresh Node/真实交易验收属于第 9 步。

2026-07-22 第 9 步正式权重与最终验收：使用当前源码 WASM、临时 fresh spec、FRAME Benchmark CLI 53.0.0、50 steps、20 repeats，重新生成核心 `votingengine`、`joint-vote`、`legislation-vote`、`election-vote` 的生产权重；联合投票超时夹具先由 `citizen-identity` 准备人口真源，再由投票引擎生成提案快照。`election-vote` 最后一票继续按候选人数线性计费，范围 `1..=256`。五个相关 crate 与 runtime 测试、benchmark 编译和当前源码 release Node 构建均通过；最终 fresh 节点 block #0 为 `0x4bd7e3f65f5ad4788e6ac8917abce9b0683f0c93d286766a7512854084ff0dd9`，state root 为 `0xd15b1a20d972f0cc5f64aa9a08a09f6793fe51886f9445c6dc953c0f9d438f7b`，六项项目 Runtime 版本均为 `0`，metadata 二进制 220,247 字节；节点已停止。
