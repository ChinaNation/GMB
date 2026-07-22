# 任务卡：公民主体、选举快照与字段统一

状态：已完成。2026-07-22 第 1 至第 9 步全部完成。

## 最终需求

- 公民授权主体统一为 `CitizenSubject { cid_number, wallet_account }`；公民 CID 证明身份与资格，钱包签名证明操作授权，二者缺一不可。
- 机构授权主体继续使用机构 CID + 岗位码 + 任职钱包；公民主体不增加岗位码，也不得退化为裸钱包。
- `citizen-identity` 是公民身份、CID 与钱包绑定、资格历史及全国/省/市/镇四级人口数据的唯一真源；它不创建、不编号、不保存提案快照。
- 投票引擎只消费 `citizen-identity` 的规范化人口数据，在具体提案内生成和保存人口快照；不得自行制造人口数据或复制完整公民名单。
- 具体选举业务模块就是该类选举的规则真源。未来每种公权选举业务在 `citizenchain/runtime/public/` 下新增独立模块，负责权限、目标岗位、候选条件、选民范围、席位、任期、指定投票引擎和结果写回。
- `election-vote` 只负责通用选举提案、候选冻结、投票票据、计票、结果快照、超时和清理，不解释具体选举规则，也不直接写 entity 任职。
- 每个机构只能发起本机构岗位的选举；选举元数据只保留一个 `actor_cid_number` 和一个现有 `role_code`，不得支持 A 机构发起 B 机构岗位选举。
- 删除选举元数据中重复的目标机构、另造职位编码和无真源规则编号；提案实例只使用投票引擎生成的全链唯一 `proposal_id`，业务类型由 `BusinessActionId` 表达。
- 公民姓名在全仓统一使用 `family_name`、`given_name`；删除合并姓名字段及所有带公民前缀的姓、名别名。身份结构已经限定公民语义，不重复加前缀。
- 普选票据、候选人快照、候选人计票和当选结果都使用完整 `CitizenSubject`，不得继续只保存钱包账户。
- 有效人口分母必须包含状态正常且在快照日期护照有效的公民；护照尚未生效、已经过期、身份吊销或行政区不匹配均 fail-closed。
- 当前正式链尚未创世，不写 migration、不保留旧 storage、旧字段、旧载荷、别名、双读或兼容分支。
- 项目自身版本统一归零：runtime 数字版本、全部 pallet storage version 为 0，CitizenChain workspace/runtime/Node 程序包版本为 `0.0.0`；第三方依赖和 Substrate runtime API trait 协议版本不属于项目升级计数，不修改。

## 最终模块边界

```text
具体选举业务模块(public/*-election)
  ├─ 校验本机构有权岗位
  ├─ 绑定本机构目标 role_code
  ├─ 定义候选条件、选民范围、席位和任期
  ├─ 静态指定 Election 投票引擎
  └─ 复核结果后写 entity 任职
                    │
                    ▼
election-vote / votingengine
  ├─ 从 citizen-identity 取得规范化公民主体和人口数据
  ├─ 生成提案人口快照或岗位任职快照
  ├─ 冻结候选主体、保存票据、计票和形成结果
  └─ 不解释业务、不制造身份、不直接任职
                    │
                    ▼
citizen-identity / entity
  ├─ citizen-identity：公民身份与人口数据真源
  └─ entity：机构岗位与任职真源
```

## 最终核心结构

```rust
CitizenSubject {
    cid_number,
    wallet_account,
}

CandidateIdentity {
    birth_province_code,
    birth_city_code,
    birth_town_code,
    family_name,
    given_name,
    citizen_sex,
    birth_date,
    updated_at,
}

ElectionMeta {
    mode,
    population_scope,
    actor_cid_number,
    role_code,
    seat_count,
    term_start,
    term_end,
}
```

`VotingIdentityByCid`、`CandidateIdentityByCid` 和资格版本历史统一以永久 CID 为 storage key；身份结构不重复保存 CID，也不以钱包搬迁身份资料。`WalletAccountByCid` 与 `CidByWalletAccount` 只保存当前唯一签名钱包的双向绑定。需要授权、候选或票据主体时，必须由永久 CID 与当前双向绑定的钱包构造完整 `CitizenSubject`，任何缺失或错配均 fail-closed。

## 分步骤实施

1. **架构、字段和版本规则冻结**：新增本任务卡并更新有效架构、模块、命名、协议和旧选举骨架任务文档；不修改代码。
2. **项目版本全面归零**：runtime 自身数字版本、19 个已有 pallet storage version 和 workspace/Node 程序包版本归零；第三方依赖与 runtime API trait 版本不动。
3. **公民身份姓名与主体规范化**：在 citizen-identity 实现完整公民主体，竞选身份改用 `family_name`、`given_name`，清除三套旧姓名表达。
4. **四级有效人口数据**：修正护照生效、到期、身份状态和迁居对全国/省/市/镇人口计数的影响，以有界日期推进保证人口分母就绪。
5. **投票引擎公民主体快照接口**：资格查询返回完整 `CitizenSubject`，人口快照继续只保存作用域、有效总数、资格 revision、判定日期和创建区块。
6. **选举投票模型收口**：删除三个错误元数据字段，统一使用本机构 `actor_cid_number + role_code`，候选、普选票据、计票和结果改用完整公民主体。
7. **删除通用选举业务壳**：删除 `runtime/public/election-campaign` crate、runtime 接线和错误文档；pallet index 留空，本任务不新增具体选举业务模块。
8. **OnChina、CitizenApp、CitizenWallet、QR 全端统一**：数据库、API、前端、SCALE、离线签名展示和 QR 字段统一到最终协议，不兼容旧载荷。
9. **权重、文档、残留与真实验收**：重算受影响权重，完成全端测试、fresh runtime/Node、真实 OnChina/数据库/HTTP/页面/签名验收及全仓旧字段清零。

每一步必须先输出完整技术方案并等待确认。任何涉及 `citizenchain/runtime/` 的修改都必须另列完整 runtime 路径并取得该步二次确认；执行中遇到无法从仓库确认的业务规则必须立即停止沟通。每步执行完成后更新文档、完善中文注释、清理残留，再输出下一步方案。

## 预计修改目录

- `memory/01-architecture/`、`memory/04-decisions/`、`memory/05-modules/`、`memory/07-ai/`、`memory/08-tasks/`：冻结架构、协议、命名、任务进度和最终验收；文档及旧口径清理。
- `citizenchain/runtime/misc/citizen-identity/`：公民主体、竞选姓名、资格历史和有效人口数据；runtime 代码、测试、权重和残留清理。
- `citizenchain/runtime/votingengine/`：完整公民主体资格、人口快照消费和通用清理；runtime 代码、测试、权重和残留清理。
- `citizenchain/runtime/votingengine/election-vote/`：选举元数据、候选主体、票据、计票和结果；runtime 代码、测试、权重和残留清理。
- `citizenchain/runtime/public/election-campaign/`：删除无具体规则的通用业务壳及其错误边界。
- `citizenchain/runtime/src/`、`citizenchain/runtime/Cargo.toml`、`citizenchain/Cargo.toml`：runtime 接线、项目版本归零和通用业务壳移除；代码、配置和残留清理。
- `citizenchain/crates/qr-protocol/`：身份与选举 QR/SCALE 字段契约；代码、生成源、测试和残留清理。
- `citizenchain/onchina/`：公民数据库、DTO、API、前端、交易构造和真实服务验收；代码、生成物和残留清理。
- `citizenapp/`、`citizenwallet/`：身份签名、选举投票、离线解码和 QR 展示；代码、生成物、测试和残留清理。
- `citizenchain/node/`、`citizenweb/`：Node 程序版本、metadata、公开文档和生成文档；代码、生成物、文档和残留清理。

## 非目标

- 本任务不实现参议员、众议员或其他具体选举业务。
- 不修改机构岗位权限、机构阈值或个人多签模型。
- 不在投票引擎实现任何具体选举法规则。
- 不生成正式 chainspec，不部署，不提交或推送 Git。

## 进度

- [x] 第 1 步：架构、字段和版本规则冻结
- [x] 第 2 步：项目版本全面归零
- [x] 第 3 步：公民身份姓名与主体规范化
- [x] 第 4 步：四级有效人口数据
- [x] 第 5 步：投票引擎公民主体快照接口
- [x] 第 6 步：选举投票模型收口
- [x] 第 7 步：删除通用选举业务壳
- [x] 第 8 步：全端协议统一
- [x] 第 9 步：权重、文档、残留与真实验收

## 第 1 步完成记录（2026-07-21）

- 用户确认通用 `election-campaign` 直接删除，不改名；未来每种公权选举业务在 `runtime/public/` 下新增独立业务模块。
- 用户确认当前正式链尚未创世，项目自身 runtime、pallet storage 和 workspace/Node 程序版本统一归零；第三方依赖及 Substrate runtime API trait 协议版本不动。
- 最终字段、职责、九步顺序、逐步确认、runtime 二次确认和无兼容边界已写入有效架构、模块、命名与协议文档。
- 本步只修改 `memory/` 文档，没有修改 runtime、Node、OnChina、CitizenApp、CitizenWallet、QR 或生成代码，没有创建代码文件或目录。
- 本步文档 `git diff --check` 与行尾空白检查通过；任务卡文件名为 51 个 UTF-8 字节，符合不超过 160 字节的规则。
- 冲突扫描确认：旧非零版本只保留在明确标注为“历史验收事实”的记录中，`election-campaign` 只保留为待删除对象；有效目标协议不再采用旧姓名字段、跨机构目标字段、职位编码或通用规则编号。

## 第 2 步完成记录（2026-07-21）

- runtime 的 `authoring_version`、`spec_version`、`impl_version`、`transaction_version`、`system_version` 已全部归零；runtime 版本专项测试和完整 46 项测试通过。
- 19 个既有项目 pallet 的 `StorageVersion` 已全部归零；`pow-difficulty` 的开发期 `on_runtime_upgrade` 已删除，未新增 migration、兼容分支或双读。
- CitizenChain workspace 版本已改为 `0.0.0`，`Cargo.lock` 仅把 38 个本地 workspace 包从 `1.0.0` 改为 `0.0.0`；第三方依赖版本和 Substrate runtime API trait 版本未修改。
- WASM CI 已删除 SSH 查询开发链和临时抬升 `spec_version` 的步骤，改为按源码版本原样编译；相关有效架构、Node、runtime-upgrade、CI 路由及历史任务文档已同步。
- 用户复核后进一步确认版本发布边界：只有公民控制台「运行 WASM CI」读取已配置正式目标链，在源码版本严格等于链上版本时把 `spec_version` 与现有测试断言同步加一；没有正式目标链或发现版本漂移就停止。GitHub workflow 仍只按源码原样编译，其他入口不增加版本，且不恢复 SSH。
- 受影响 19 个 pallet 共 350 项测试、19 crate `no_std`、runtime benchmark/try-runtime 编译、release WASM 与 release Node 构建通过。
- NodeGuard 并发回归 79 项直接通过，2 项因共享临时数据库锁竞争失败后分别串行通过，实际覆盖 81/81；未为测试夹具竞争修改生产代码。
- 当前源码 WASM 的 `runtime_version` 自定义段确认为全零；显式嵌入该 WASM 后使用 `--chain citizenchain-fresh --tmp` 真实启动，RPC 返回 runtime 六项展示值均为 `0`、block `#0`、`isSyncing=false`，metadata 正常返回 430,412 字节。最终 genesis hash 为 `0x3a23218ebfb3c3b0052b53ba8ffd6866935bd0209c18f674959ed10505a6f39a`，state root 为 `0x4040ff06497ea571d57fc5916bdfb29435ff10b39e7ac5739cac2bc012a4eb2e`；验收节点已停止，临时数据由 `--tmp` 清理。
- 本步未生成或修改正式 chainspec，未部署、提交或推送 Git。

### 第 2 步版本发布规则复核补充（2026-07-21）

- `pow-difficulty` 删除的是开发期旧存储布局的一次性 `on_runtime_upgrade`，不是 Runtime 升级能力；治理升级、开发期直升、`System.set_code` 和 try-runtime 升级检查均保留。
- 公民控制台 WASM 卡片已移除无效的 `GMB_SSH_KEY` 就绪依赖。「运行 WASM CI」现在读取充值发币页配置的目标链 `NODE_WS`，并强制 RPC genesis hash 与协议升级区明确保存的 `CHAIN_GENESIS_HASH` 一致；没有正式链指纹、目标链不可达、指纹不匹配或响应无效时直接停止。
- 控制台只在源码版本、现有版本测试断言和链上版本三者严格一致时，把 `citizenchain/runtime/src/lib.rs` 的 `spec_version` 与 `runtime/src/tests/cases.rs` 的精确断言同步提高 1；任何版本漂移都 fail-closed，且不会机械修改其他四个 Runtime 版本字段。
- `citizenchain-wasm.yml` 新增普通源码构建/控制台升级构建分流。两者都按提交源码原样编译；升级构建只校验“源码版本 = 目标链版本 + 1”并在 CI 摘要记录 genesis hash、升级前后版本和提交 SHA，其他手动触发不提高版本。
- 隔离临时副本已真实验证 `0 → 1` 同步更新及第二次以链上 `0` 调用时因版本漂移被拒；仓库当前 `spec_version` 和测试断言均仍为 `0`。`actionlint`、`shellcheck`、CitizenConsole `npm run check`、相关文件 `git diff --check` 均通过。
- CitizenConsole 全量 `npm test` 的 22 项中 16 项通过、6 项 `settle.test.mjs` 失败；失败均位于未被本次修改的充值结算测试/实现，首项表现为预期 200 实得 400，其余为连锁断言。本次未跨边界修改充值业务逻辑。
- 本次没有点击按钮、没有连接或认定任何正式目标链，没有改动当前 Runtime 版本值，没有触发提交、推送或 GitHub CI。

## 第 3 步完成记录（2026-07-21）

- `citizen-identity` 新增只读 `CitizenSubject { cid_number, wallet_account }`。身份和资格历史以永久 CID 为主键，钱包只通过 `WalletAccountByCid` 与 `CidByWalletAccount` 表达当前签名绑定；读取时必须同时验证双向绑定、正常身份状态和 Active CID，任何缺失、吊销或错配均返回 `None`。
- 公民 CID 不提供修改、替换、删除或复用路径。资料更新和迁居只能更新同一个 CID 下的身份版本；传入另一 CID 会被当作无权修改并拒绝，不能把原身份迁移到新 CID。
- `CandidateIdentityPayload` 与 `CandidateIdentity` 已删除 `citizen_full_name`，统一拆分为 `family_name`、`given_name`；姓、名分别限定最多 128 字节且分别校验非空，没有旧字段、别名、双读或 migration。
- `can_vote` 已先校验完整公民主体，避免在 CID↔钱包反向绑定损坏时退化为裸钱包授权；投票引擎将于第 5、6 步把现有裸钱包快照和票据切换到完整主体。
- 已为三个目标结构补齐中文字段注释，并增加完整主体正常读取、绑定错配 fail-closed、吊销 fail-closed、姓/名分别必填及 runtime 集成覆盖。
- NodeGuard 直接读取 `VotingIdentityByCid`、`WalletAccountByCid` 与 `CidByWalletAccount`，逐块和完整状态都要求永久 CID 身份及当前钱包双向绑定闭环；公民首次认证发行同样以该闭环为依据。
- 验证通过：`citizen-identity` 30 项、runtime 46 项、NodeGuard 公民发行 8 项、NodeGuard CID 生命周期 3 项；`citizen-identity --no-default-features`、runtime `wasm32v1-none`、`runtime-benchmarks`、`try-runtime` 和 Rust 1.94.0 固定工具链 release Node 构建全部通过。
- 当前源码以 `citizenchain-fresh --tmp` 启动全新隔离链，NodeGuard 与创世装载通过；RPC 返回 `peers=0`、`isSyncing=false`、runtime 六项项目版本均为 `0`，metadata 二进制 215,796 字节。block #0 / genesis hash 为 `0x45144d74a7af61bb25cc08a803a19af1cdc946b007d22c774ce3acdeeebd7db4`，state root 为 `0xe916b283c7cd017aa87d2bfda2b835298195d2cbfc53c19536d0fddeae9874ea`；验收节点已停止，临时数据由 `--tmp` 清理。
- 本轮首次完成记录时未修改投票引擎、OnChina、CitizenApp、CitizenWallet 或 QR；随后用户明确要求全仓字段和机构授权协议立即统一，并补充确认全部相关 runtime 路径，具体补充结果见下节。

### 第 3 步跨端补充收口（2026-07-21）

- 所有公民姓名和法定代表人姓名统一使用 `family_name`、`given_name`；runtime 法定代表人收口为原子 `legal_representative: Option<{ family_name, given_name, cid_number, account }>`，不再保存拼接姓名或主体前缀别名。
- 公权/私权机构管理、公民身份、地址、runtime 升级、链上发行及链下交易等已确认入口统一显式携带机构 CID、岗位码并以当前签名管理员钱包完成三项联合授权；管理员账户本身不产生业务权限。
- 公民身份上链仍保持注册局管理员与目标公民钱包双签：公民签署身份内容，注册局任职管理员签署最终链交易；本轮没有扩展护照续期、居住地更新或钱包更换业务。
- OnChina PostgreSQL 目标模型、API、前端与链交易编码，CitizenApp、CitizenWallet、QR 注册表和 Node 消费端已同步最终字段顺序；数据库启动清理只删除旧列，不读取、不回填、不兼容旧格式。
- 验证通过：QR 注册表 6 项一致性/仓库守卫测试；OnChina 后端 134 项测试与前端生产构建；CitizenApp 静态检查及 37 项定向测试；CitizenWallet 静态检查及 signer 目录 146 项测试；runtime 46 项全量测试；runtime-upgrade 20 项测试；Node 链下批次编码 7 项、runtime-upgrade 调用编码 1 项和提案解码组 10 项测试。
- 全 runtime 机构业务入口复查发现的最后一处偏差已经收口：`resolution-issuance::propose_issuance` 显式接收 `proposer_role_code`，权限校验、`VotePlan.proposer_subject`、业务数据和事件均保存 CID + 岗位码 + 签名钱包主体；无岗位码旧载荷直接拒绝。决议发行的 NRC/PRC 委员发起与 NRC/PRC 委员、43 PRB 董事投票规则未改变。
- 强制从当前源码重编译 release WASM 与 Node 后，以 `citizenchain-fresh --tmp` 启动全新隔离链完成真实运行态验收：RPC 返回 `peers=0`、`isSyncing=false`，`authoring/spec/impl/transaction/system/state` 六项项目 Runtime 版本均为 `0`，metadata 二进制为 217,659 字节。block #0 / genesis hash 为 `0xea6bf4639f6f810f87f299a07fc38d5665839e8bfd60c785dde9c3e687fa58d0`，state root 为 `0x7fc3e650d6651a7b4a1af9e60629dab2aff305abbac367f55ba18628d5b29a7e`；验收节点已正常停止，临时数据由 `--tmp` 清理。
- 未修改投票引擎人口快照或有效人口分母；这两项仍分别属于第 4 至第 6 步。未生成正式 chainspec，未部署、提交、推送或触发 GitHub CI。

## 第 4 步完成记录（2026-07-22）

- `citizen-identity` 已成为全国、省、市、镇四级有效人口的唯一真源。最终分母只统计永久 CID 有效、CID↔钱包双向绑定完整、身份状态正常、护照在判定日期有效且居住作用域匹配的公民；未生效、已过期、已吊销或迁移离开作用域的身份不再计入。
- 新增 `PopulationReadyDate` 及按日期、顺序号存储的护照生效/失效转换队列。`on_idle` 每块最多推进 366 个日期、处理 2,048 个转换项，并受最大区块权重 `1/8` 的独立预算限制。某日未完整处理时不发布部分分母，也不接受会改变人口的身份写入。
- 护照有效区间为闭区间：`valid_from` 当日加入，`valid_until` 当日仍有效，下一自然日退出。日期采用严格公历校验，覆盖大小月、闰年、跨年和时间倒退故障。身份修订号使旧转换项自动失效，不会重复加减人口。
- 人口 provider 已收口为 `population_data(scope) -> Option<PopulationData>`；只有当前 UTC+8 日期全部就绪且无维护故障时才返回数据。裸 `population_count()` 接口已删除，投票引擎建立人口快照时如果真源未就绪，以 `PopulationDataNotReady` 原子回滚，不会遗留提案或快照。
- 不可恢复故障记录覆盖日期倒退、计数溢出/下溢和转换队列缺损；故障后身份人口变更与新快照统一 fail-closed。本步没有在 `citizen-identity` 生成提案快照，没有复制公民名单，也没有修改具体业务模块的投票规则。
- 验证通过：`citizen-identity` 36 项、`election-vote` 14 项、`citizen-issuance` 14 项单元测试与 5 项集成测试、runtime 46 项及受影响投票/业务 crate 测试；全 workspace 测试目标编译；`citizen-identity --no-default-features`、runtime benchmark/try-runtime、runtime `wasm32v1-none --no-default-features` 和当前源码 release Node 构建。
- 从当前源码嵌入 WASM 后，以 `citizenchain-fresh --tmp` 启动全新隔离链完成真实运行态验收：RPC 返回 `peers=0`、`isSyncing=false`，`authoring/spec/impl/transaction/system/state` 六项项目 Runtime 版本均为 `0`，metadata 二进制为 219,646 字节。block #0 / genesis hash 为 `0x1eee16b152f0e8e50a84bf38a3ccda8f91458bcc8d843ac51640818c1dfeb560`，state root 为 `0x94374f3eea81371fb810889ecaf22ceb740b6d0029ccd27986cd8b42c63a1dc6`；验收节点已停止，临时数据由 `--tmp` 清理。
- 项目 runtime、storage 和程序包版本保持为 0/`0.0.0`，未新增 migration、兼容分支或双读；未修改 OnChina、CitizenApp、CitizenWallet 或 QR 协议，未生成正式 chainspec，未部署、提交、推送或触发 GitHub CI。

## 第 5 步完成记录（2026-07-22）

- `CitizenIdentityProvider` 已删除只返回 bool 的 `can_vote`、`can_be_candidate`、`can_vote_at`，统一改为 `voting_subject`、`candidate_subject`、`voting_subject_at`，成功时返回 `CitizenSubject { cid_number, wallet_account }`，任何 CID、钱包、身份、护照、作用域或历史 revision 错配均 fail-closed。
- 联合公投与立法公投票据已从 `(proposal_id, wallet_account)` 收口为 `(proposal_id, cid_number)`，票据值保存完整公民主体和票值；同一永久 CID 更换绑定钱包后仍不能重复投票。事件同步输出完整主体，不再把裸钱包表达为公民投票身份。
- 人口快照边界未扩大：`ProposalPopulationSnapshots` 仍只保存作用域、有效人口总数、资格 revision、判定日期和创建区块，不保存全量公民名单。`citizen-identity` 继续是人口与身份数据唯一真源，投票引擎只在提案内消费并冻结。
- `election-vote` 本步只完成新主体 provider 的临时接线，没有提前修改选举模型。现存 `target_cid_number`、`office_code`、`rule_id`、裸钱包候选快照和 Popular 票据已复查确认，统一留给第 6 步一次性删除或替换。
- 验证通过：`citizen-identity` 36 项、`election-vote` 14 项、`internal-vote` 95 项、`joint-vote` 13 项、`legislation-vote` 35 项、runtime 46 项，以及受影响机构、治理、发行、立法和转账模块测试；全 workspace 测试目标编译通过。
- `citizen-identity --no-default-features`、runtime benchmark/try-runtime、runtime `wasm32v1-none --no-default-features`、第 5 步文件定向 rustfmt、`git diff --check` 和当前源码 release Node 构建通过。全仓 `cargo fmt --check` 仍会报告本步范围外的既有格式差异，本步没有越界格式化这些文件。
- 从当前源码嵌入 WASM 后，以 `citizenchain-fresh --tmp` 启动全新隔离链完成真实运行态验收：RPC 返回 `peers=0`、`isSyncing=false`，`authoring/spec/impl/transaction/system/state` 六项项目 Runtime 版本均为 `0`，metadata 二进制为 220,197 字节。block #0 / genesis hash 为 `0x69b4a0025356d050004cff3ef176167a6520b59c9086c9ac6b9a45c4b9e9c0e6`，state root 为 `0x0b066c3567ed25c15cfa96b7d249b6235df4746a253144db21c87dfd2ed2333e`；验收节点已停止，端口已释放，临时数据由 `--tmp` 清理。
- 项目 runtime、storage 和程序包版本保持为 0/`0.0.0`，未新增 migration、兼容分支或双读；未修改 OnChina、CitizenApp、CitizenWallet 或 QR 协议，未生成正式 chainspec，未部署、提交、推送或触发 GitHub CI。

## 第 6 步完成记录（2026-07-22）

- `ElectionMeta` 已删除 `target_cid_number`、`office_code`、`rule_id`，最终只保存 `mode + population_scope + actor_cid_number + role_code + seat_count + term_start + term_end`。发起机构就是拟任职机构，内部机构码只从唯一 actor CID 推导，互选 VotePlan 的全部岗位选民主体也必须属于该 CID。
- `ElectionCandidates`、`ElectionCandidateTallies` 和 `ElectionResults` 已从裸钱包改为完整 `CitizenSubject`；当选结果项保存 `candidate_subject + votes + seat_index`。候选快照按永久 CID 去重，并通过新增的只读 `CitizenIdentityReader::citizen_subject` 校验 CID↔钱包真源闭环。
- 普选票据已改为 `PopularElectionVotesByCid[proposal_id][cid_number] -> { voter_subject, candidate_subject }`；同一永久 CID 更换绑定钱包后不能再投第二票。互选继续按完整 `InstitutionVoteTicket { cid_number, role_code, voter_account }` 去重，票据值改为完整候选主体。
- 同一管理员担任同一机构多个岗位时，每个冻结的“机构 CID + 岗位码 + 钱包”票据仍可各投一票；同一岗位票据不能重复使用。未新增岗位阈值，也未修改机构阈值。
- 通用引擎不再要求互选候选人同时属于选民岗位。具体候选条件、选举规则和结果写回继续由未来 `runtime/public/*-election` 业务模块负责；引擎只校验业务模块交付的完整公民主体来自 citizen-identity，不直接任职。
- `ElectionCreated` 事件改为输出唯一 `actor_cid_number + role_code`；投票事件输出完整公民主体或机构岗位票据以及完整候选主体。清理逻辑、benchmark 夹具和权重 storage 名已同步；第 9 步正式重算前，proof 上界已按新 MaxEncodedLen 保守提高，未继续使用偏小旧值。
- 验证通过：`election-vote` 17 项、votingengine 4 项、runtime 46 项；全 workspace 测试目标、两个目标 crate `no_std`、runtime 普通/benchmark/try-runtime、runtime `wasm32v1-none --no-default-features`、定向 rustfmt 和 `git diff --check` 全部通过。
- 从当前源码嵌入 WASM 后，以 `citizenchain-fresh --tmp` 启动全新隔离链完成真实运行态验收：RPC 返回 `peers=0`、`isSyncing=false`，`authoring/spec/impl/transaction/system/state` 六项项目 Runtime 版本均为 `0`，metadata 二进制为 220,398 字节。block #0 / genesis hash 为 `0x285ca7f4ab0f24771baff6a6fc10141ee281fbbd6ce1a8f9dcd1d7676501a41b`，state root 为 `0x27ecdc5b73ce195df4bdfe6c05fe68ef0b682c58751f5f145868a69a1f4672bd`；验收节点已停止，端口已释放，临时数据由 `--tmp` 清理。
- `election-vote` 中三个错误字段及旧钱包票据名已清零；全仓只剩第 7 步将整体删除的 `runtime/public/election-campaign` 壳仍含这些字段。项目 runtime、storage 和程序包版本保持为 0/`0.0.0`，未新增 migration、兼容分支或双读；未修改客户端、QR 或 OnChina，未生成正式 chainspec，未部署、提交、推送或触发 GitHub CI。

## 第 7 步完成记录（2026-07-22）

- 开发期无具体规则的通用选举业务 crate 已从 `runtime/public/` 物理删除；workspace 成员、Runtime 依赖、`std`/benchmark/try-runtime feature、Config 实现和 `construct_runtime` 接线全部删除，`Cargo.lock` 不再包含该包。
- 原 pallet index 32 已从 Runtime metadata 移除并永久留空，不在本任务复用。`election-vote` 仍只负责选举投票、计票和结果快照；注释与测试业务标签均改为“创建提案的具体选举业务模块”，没有保留旧模块名、模块标识或错误变体。
- 已删除错误模块文档；旧骨架任务卡移动到 `memory/08-tasks/done/` 并明确标记为退役删除。有效架构、跨模块边界、投票引擎文档、白皮书及节点内置生成文档已同步，未来每一种公权选举必须建立自己的独立业务模块并定义规则。
- 验证通过：`election-vote` 17 项、runtime 46 项；全 workspace 测试目标编译；`election-vote --no-default-features`、runtime 普通/benchmark/try-runtime、runtime `wasm32v1-none --no-default-features`、release Node 构建及 `git diff --check` 全部通过。
- 从当前源码嵌入 WASM 后，以无头 `citizenchain-fresh --tmp` 启动全新隔离链完成真实运行态验收：RPC 返回 `peers=0`、`isSyncing=false`，`authoring/spec/impl/transaction/system/state` 六项项目 Runtime 版本均为 `0`，metadata 二进制为 220,247 字节。block #0 / genesis hash 为 `0x9dd176890773edfbd5ef543a5710e2115d05cdf6dd36c91cc230eb53feab5fbe`，state root 为 `0x0757b0cd5b1923328bd10ac9ab26e86ee6fd6363677799281e7dceda389327bb`；metadata 不含已删除壳的模块标识、业务标签或错误名，验收节点已停止且临时数据由 `--tmp` 清理。
- 节点前端生产构建通过，并通过本地 Vite preview 的真实页面验收：白皮书正确渲染“具体公权选举业务模块 / Specific Public Election Business Modules”，不再显示旧通用模块名或旧集中式规则表述；验收服务和浏览器标签均已关闭。
- 项目 runtime、storage 和程序包版本保持为 0/`0.0.0`，未新增 migration、兼容分支、双读、正式 chainspec 或具体选举业务模块；未修改 OnChina、CitizenApp、CitizenWallet 或 QR 协议，未部署、提交、推送或触发 GitHub CI。

## 第 8 步完成记录（2026-07-22）

- CitizenApp 的“我的身份”和广场身份统一通过永久 CID 闭环读取：同一 finalized 区块内依次校验 `CidByWalletAccount`、Active `CidRegistry`、`WalletAccountByCid` 反向绑定和 CID 主键身份；CID 只作 storage key，不再从身份值重复解码。截断、尾随、状态矛盾或钱包错配全部 fail-closed。
- Cloudflare 身份投影采用同一 finalized head 的五项读取与同一 SCALE 布局，护照有效期按 UTC+8 判定；竞选身份只使用 `family_name`、`given_name`，不保留合并姓名或带公民前缀的姓名字段。
- QR 唯一注册表新增 `ElectionVote.cast_popular_vote = 0x1602` 与 `cast_mutual_vote = 0x1603`，两端生成表已同步。CitizenWallet 按最终 call data 严格展示 `proposal_id + cid_number + wallet_account`，互选另展示 `voter_role_code`，旧裸钱包、截断 CID 和尾随载荷一律拒签。
- CitizenApp 未虚构通用选举业务入口；“选举”和“发起选举”继续禁用，并明确只有未来具体公权选举业务模块接入后才能开放。OnChina 仅清理数据库启动期旧列兼容语句，保持注册局管理员与公民双签、机构登记和权限业务流程不变。
- 验证通过：CitizenApp 全量 763 项通过、5 项跳过并通过静态分析；Cloudflare 29 个测试文件 172 项与类型检查通过；CitizenWallet 全量 185 项与静态分析通过；QR registry 6 项一致性/仓库守卫测试、OnChina 后端 134 项和前端生产构建通过；`git diff --check` 通过。
- 真实验收使用临时 PostgreSQL 初始化 OnChina 最终 schema，确认 `family_name`、`given_name`、护照有效期和行政区字段存在，7 个旧列为 0；生产前端经 Vite preview 真实渲染管理员扫码登录页。临时数据库和服务均已停止，临时数据已移入系统废纸篓。
- 当前 release Node 的 fresh 与既有链启动均在节点护宪守卫基准派生处报 `LawDecodeFailed`；同源码定向测试 `real_runtime_genesis_satisfies_full_constitution_guard` 通过。该问题不来自第 8 步客户端、QR 或 OnChina 改动，本步未绕过护宪守卫，留在第 9 步从当前源码、嵌入 WASM 与节点解码结构三方做真实收口。
- 第 8 步没有修改任何 `citizenchain/runtime/` 文件，项目 runtime、storage 和程序包版本继续保持 0/`0.0.0`；未新增 migration、兼容分支、具体选举业务模块、正式 chainspec，未部署、提交、推送或触发 GitHub CI。

## 第 9 步完成记录（2026-07-22）

- `citizen-identity` 新增正式 FRAME benchmark，覆盖投票身份注册、升级竞选身份、更新投票/竞选身份、吊销身份、单个/批量 CID 占号、CID 吊销和四条人口维护路径；批量占号按 `1..=10,000` 线性计费。benchmark 使用真实创世注册局机构、专员岗位、任职和权限目录，不增加“管理员即有权”的旁路。
- runtime benchmark registry 已登记 `citizen_identity`。`scripts/benchmark.sh` 每次强制从当前源码构建 WASM，使用当前源码导出的临时 fresh spec，并在退出时清理临时文件，避免冻结旧 WASM 或旧 storage 布局污染测量。
- 使用 FRAME Benchmark CLI 53.0.0、50 steps、20 repeats，正式重算 `citizen-identity`、核心 `votingengine`、`joint-vote`、`legislation-vote`、`election-vote` 五组生产权重。人口维护权重采用完整生产路径的可组合安全上界；组合时允许重复计入固定开销，但不得低估链上最重路径。
- 第 8 步的 `LawDecodeFailed` 已定位为 macOS 桌面模式没有采用命令行 fresh chain 参数，并非护宪守卫或 runtime 法律数据错误。节点验收统一显式设置 `CITIZENCHAIN_HEADLESS=1`；新增回归测试直接物化 fresh genesis、解码 Law(0)、派生不可变参考并执行完整护宪校验，当前源码嵌入 WASM 时通过，未削弱或绕过守卫。
- 编译与 runtime 验证通过：`citizen-identity`、runtime、Node 的 `runtime-benchmarks` 编译通过；`citizen-identity` 36 项、runtime 46 项、`election-vote` 17 项、`joint-vote` 13 项、`legislation-vote` 35 项、核心 `votingengine` 4 项及 doc tests 全部通过；强制当前源码 WASM 的 release Node 构建通过。
- 全端最终回归通过：CitizenApp 763 项通过、5 项跳过且静态分析无问题；CitizenWallet 185 项及静态分析通过；Cloudflare 29 个测试文件 172 项及类型检查通过；OnChina 后端 134 项、前端生产构建通过；QR registry 6 项通过。第 8 步已用临时 PostgreSQL、真实 HTTP 与页面完成最终 schema 和双签流程验收，本步未修改 OnChina schema 或业务逻辑，并再次通过生产 bundle 的真实 preview：`HTTP 200`、标题“链上中国平台”、废弃姓名字段为 0；服务已停止。
- 最终 release Node 以 `CITIZENCHAIN_HEADLESS=1 citizenchain-fresh --tmp` 真实启动，RPC 返回 block `#0`、`peers=0`、`isSyncing=false`，`authoring/spec/impl/transaction/system/state` 六项项目版本均为 `0`，metadata 二进制 220,247 字节。genesis hash 为 `0x4bd7e3f65f5ad4788e6ac8917abce9b0683f0c93d286766a7512854084ff0dd9`，state root 为 `0xd15b1a20d972f0cc5f64aa9a08a09f6793fe51886f9445c6dc953c0f9d438f7b`；验收节点已停止。bootnode 报告既有部署链 genesis 不同，符合本任务未发布正式统一 chainspec 的边界。
- 全部本地项目包版本为 `0.0.0`，runtime 数字版本和全部项目 pallet storage version 为 0；`node/vendor` 的第三方 GRANDPA 路径依赖版本不属于项目升级计数。有效代码、生成物和现行架构文档中旧姓名字段、旧选举元数据字段、通用选举业务壳、旧裸钱包票据及错误版本注释已清理；历史完成任务卡只保留明确的历史事实。
- 本任务没有生成正式 chainspec，没有部署、提交、推送或触发 GitHub CI；所有临时 Node、页面和 benchmark 资源均已停止或清理。第 9 步为本任务最后一步，不再输出后续实现步骤。
