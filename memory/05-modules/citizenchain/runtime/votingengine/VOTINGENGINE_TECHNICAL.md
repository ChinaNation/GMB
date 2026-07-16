# votingengine 技术说明

## 定位

`votingengine` 是链上中国 runtime 的统一投票引擎。

业务模块只提交提案语义，不能自行实现投票流程、人口快照、投票资格、计票、通过判定或清理状态机。

## 内部投票与业务权限

- `internal-vote` 是所有机构与个人多签共用的管理员投票程序，负责内部投票模式准入、CID/个人账户主体、管理员快照、计票、阈值快照和终态。
- “机构可以使用内部投票”不等于“机构可以发起所有接入内部投票的业务”。有效准入 = 投票引擎模式准入 + 业务 pallet 具体权限，两层任一拒绝都不能创建或执行提案。
- `multisig` 转账允许已登记机构账户和个人多签账户；机构调用显式携带 `actor_cid_number + institution_account`，反向索引只校验两者归属，不反推或回落机构身份。
- `resolution-destroy` 只允许 NRC、PRC、PRB；`grandpakey-change` 只允许 NRC、PRC。业务限制不得下沉到 `internal-vote`。
- FRG 是一个 CID 机构并拥有多个协议账户和 215 名管理员；省域 5 人岗位组属于注册业务权限，通用内部投票只校验 FRG CID、管理员授权和 CID 快照。

### 内部投票阈值

- NRC、PRC、PRB、NJD、FRG 使用代码级永久固定阈值，不写账户级动态阈值。
- PRS、NLG、NSN、NRP、NSP、NED 六个国家单例没有机构级动态阈值；普通内部事项在创建提案时按当前 admins 快照计算 `floor(N/2)+1`，只写 `InternalThresholdSnapshot`。
- 六个国家单例禁止写入 `ActiveInstitutionThresholds` 或待变更阈值。
- 普通注册机构使用 `ActiveInstitutionThresholds[cid_number]`，个人多签使用
  `ActivePersonalThresholds[personal_account]`；生命周期关闭使用全员快照，具体业务权限仍由业务 pallet 校验。

## 公民身份真源

投票资格和参选资格统一通过 `CitizenIdentityReader` 读取 `citizen-identity`：

- `can_vote(who, scope)`：判断账户在作用域内是否有投票资格。
- `can_be_candidate(who, scope)`：判断账户在作用域内是否有参选资格。
- `population_count(scope)`：读取链上人口分母。
- `create_population_snapshot(scope)`：由身份真源同时冻结人口分母、资格 revision 和护照判定日期。
- `can_vote_at(who, snapshot_id)`：按账户创建时的身份历史校验资格，不读取投票时的当前身份。
- `release_population_snapshot(snapshot_id)`：提案 90 天历史清理完成后释放快照元数据。

OnChina 本地数据库只能用于注册局录入和界面提示，不能作为链上投票资格真源。

## 人口作用域

`PopulationScope` 支持四级：

- `Country`
- `Province(province_code)`
- `City(province_code, city_code)`
- `Town(province_code, city_code, town_code)`

联合公投和立法特别案在创建提案前先调用对应的 `prepare_*_population_snapshot(scope)`。runtime 在当前区块要求 `citizen-identity` 创建不可变快照，消费端只缓存 `snapshot_id + eligible_total + prepared_at`，不得复制作用域或选民名单作为第二真源。

## 联合投票

- 内部阶段：`JointVote::cast_admin(proposal_id, institution, approve)`。
- 联合公投阶段：`JointVote::cast_referendum(proposal_id, approve)`。
- 联合公投按 `proposal_id + who` 去重。
- 联合公投资格由 `CitizenIdentityReader::can_vote_at(who, snapshot_id)` 判定；提案创建后新增、迁居或被撤销的当前身份均不能改变已有提案的成员集合。
- 公投分母与成员资格来自同一个 `citizen-identity` 快照；累计票数达到该分母后拒绝继续写票，参与率不得超过 100%。
- 联合业务回调必须同时绑定 `ProposalOwner`、联合 proposal kind、`STAGE_JOINT/STAGE_REFERENDUM`、业务摘要和对象摘要；联合阶段直接通过与转入公投后通过都必须执行同一项已绑定业务。

## 判定与业务执行

- 投票门槛一旦命中，只提交 `STATUS_PASSED`、释放相应活跃名额并写入 `PendingProposalExecutions`；最后一票不再同步执行转账、销毁或 `set_code` 等业务。
- `on_initialize` 按 `MaxExecutionWeightPerBlock` 与 `MaxAutoFinalizePerBlock` 双重上限消费执行队列，每项按包含 `SystemWeightInfo::set_code()` 的最重成本预留。
- 业务回调返回 `DispatchError` 时只回滚本次执行尝试，不撤销已成立的投票结果；失败按指数退避重试，达到上限转 `STATUS_EXECUTION_FAILED` 并发出 dead-letter 事件。
- 回调 `Err`、`Ignored`、结果应用错误和 Track 后处理错误统一进入同一失败处理器；每次递增 attempts，孤儿或状态不匹配的队列项立即删除。
- 达到自动执行上限后，业务执行队列永久停止；`PendingTerminalFinalizations` 只补终态副作用并拥有独立退避/dead-letter，绝不重新调用业务执行回调。
- `ProposalExecutionOutcome::RetryableFailed` 继续进入既有管理员手动重试宽限期；执行成功或失败终态仍统一触发互斥释放、业务终态通知和 90 天延迟清理。
- 自动超时 finalizer 自身返回错误时使用独立有限退避状态；达到上限或重试桶已满后写入 `AutoFinalizeDeadLetters`，不会反复阻塞同一区块的其余维护管线，公开 `finalize_proposal` 仍可在修复数据后人工恢复。

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

### 资格真源与快照边界

- 普选必须使用 `citizen-identity` 的 `PopulationScope`、`can_be_candidate`、`create_population_snapshot` 和 `can_vote_at`；只保存 snapshot_id，不接收、不枚举、不保存全国/省/市/镇完整选民列表。
- 互选属于机构内部互选，必须由对应 admins provider 的 `get_institution_admins(institution_code, cid_number)` 提供 CID 管理员快照；调用方提交的选民集合必须与完整 admins 快照等长且逐成员一致，不得删减或夹带账户。
- `election-vote` 创建入口按 `ElectionMode` 强制检查资格来源：Popular 必须有人口作用域，Mutual 必须取得目标机构 admins 快照。
- 普选人口作用域写入 `ElectionMeta`，资格引用写入核心 `ProposalPopulationSnapshotIds`；互选不写公民作用域，选民存于 `MutualVoters` 并按机构管理员集合校验候选人和选民。
- 多席位计票允许完整落入剩余席位的并列组共同当选；并列组跨越席位边界时拒绝结果。

## 清理

提案完成后统一进入投票引擎清理状态机，清理所属 Track 的投票记录、提案对象和反向索引。

- `ScheduledCleanups + ScheduledCleanupHead/Tail` 是 90 天保留期的延迟 FIFO；固定保留期保证写入顺序就是到期顺序，不再使用有界区块桶或向后扫描候选桶。
- 到期任务转入 `PendingCleanupQueue + PendingCleanupQueueHead/Tail` 就绪 FIFO；每个提案每轮只执行一个有界步骤，未完成任务排回队尾。
- 清理阶段固定为 `AdminSnapshots → TrackData → ProposalObject → FinalCleanup`；`TrackData` 只派发到提案所属 Track，不再空扫四类 sub-pallet。
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
- `joint-vote` 本 crate 现有 10 项直属测试，除纯函数边界外直接覆盖 `cast_admin`、`cast_referendum`、105 票全票、机构否决和超时转公投；`internal-vote` 继续提供跨 pallet 回归覆盖。
- `legislation-vote` 的 signing、guard、referendum、result、cleanup 文件已承载实际规则或清理辅助，不再是纯注释残桩。
- 正式链重新创世采用最终布局：五个投票 pallet 以及本次触达的 `public-admins`、`public-manage`、`private-manage` 的 `StorageVersion` 全部为 1；开发期 storage alias、升级翻译、迁移类型和迁移测试已删除，runtime 全仓不存在高于 1 的 storage version。

## 2026-07-14 Track 与维护调度收口

- Runtime 以 `(InternalVote, (JointVote, (LegislationVote, (ElectionVote, ()))))` 注册递归 Track tuple；手动超时、自动超时、模式清理和内部阈值副作用走同一类型路由。
- 自动终结、异步业务执行、历史清理分别使用 `MaxAutoFinalizeWeightPerBlock`、`MaxExecutionWeightPerBlock`、`MaxCleanupWeightPerBlock` 独立预算；生产配置当前分别为最大区块权重的 `1/4`、`1/4`、`1/8`。
- 延迟 FIFO 与就绪 FIFO 均以单调 `u64` 序号键控，不存在单区块桶容量、顺延窗口或严格头部反复处理。
- 新增公平轮转测试和 Track 隔离清理测试：大任务存在时小任务仍推进，内部提案清理不会删除联合投票账本。

## 2026-07-14 资格快照与执行重试加固

- `citizen-identity` 以全局 `eligibility_revision` 和每账户不可变版本历史冻结创建时资格；同一区块多次身份写入也能确定顺序，账户查询按版本数二分定位。
- 联合公投、立法公投和 Popular 选举统一绑定 `ProposalPopulationSnapshotIds`；旧的 `ReferendumScopes`、`LegislationMeta.referendum_scope`、Popular 全量选民表及 `MaxElectionVoters` 已删除。
- Popular 不再受完整选区人数的 `BoundedVec` 限制；Mutual 的 `MaxMutualVoters` 与 Runtime `MaxAdminsPerInstitution` 使用同一上限。
- `joint-vote` crate 直属测试由 3 个纯函数测试扩展到 10 项，直接覆盖 `cast_admin`、`cast_referendum`、105 票全票执行、机构否决转公投、超时转公投和创建后新增选民拒绝。
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
- election-vote 现有测试文件内建立完整 mock runtime，覆盖普选/互选创建、人口/管理员快照、资格拒绝、写票、超时、结果回调与分块清理；当前为 13 项。

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

2026-07-14 治理职责收口第 2 步：六个国家单例删除账户级动态阈值，普通内部事项改为按提案管理员快照派生严格过半；首次组成只原子写岗位、任职和 admins。`internal-vote` 89、`public-admins` 8、`public-manage` 42、runtime 集成 40 项测试通过。

2026-07-14 治理职责收口第 3 步：业务执行端新增 owner/kind/stage/code/account/CID/action 全绑定；联合业务接受联合阶段或公投阶段的合法通过终态；立法路由改为链端双重复校验；选举引擎删除外部创建入口和直写 entity 路径。投票引擎继续只负责投票流程，业务权限与执行前复核留在业务 pallet。
