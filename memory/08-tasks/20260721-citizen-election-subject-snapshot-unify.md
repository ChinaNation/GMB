# 任务卡：公民主体、选举快照与字段统一

状态：执行中。2026-07-21 已完成第 1 至第 3 步及第 3 步跨端补充收口；后续人口与投票引擎步骤尚未执行。

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
- [ ] 第 4 步：四级有效人口数据
- [ ] 第 5 步：投票引擎公民主体快照接口
- [ ] 第 6 步：选举投票模型收口
- [ ] 第 7 步：删除通用选举业务壳
- [ ] 第 8 步：全端协议统一
- [ ] 第 9 步：权重、文档、残留与真实验收

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
