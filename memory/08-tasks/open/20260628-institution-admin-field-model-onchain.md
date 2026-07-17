# 任务卡：机构、岗位与管理员链上模型收口

## 当前状态

- 状态：已被 `20260717-institution-minimal-registration.md` 的管理员人员独立模型替代；以下旧执行记录只说明历史，不再定义当前协议
- 当前步骤：停止；当前实现与验收以 2026-07-17 任务卡为准
- 最新业务确认：2026-07-17
- 实施方式：逐步输出技术方案，用户确认后才执行；每步完成后立即更新文档、完善中文注释、删除残留，再输出下一步方案

## 任务目标

将机构信息、机构岗位、机构管理员任职与管理员集合彻底拆分到正确模块：

- 机构信息、机构岗位和机构管理员任职归 `entity`。
- 机构管理员 `admin_name + admin_account` 人员集合及其生命周期归 `admins`，岗位不得反向派生管理员。
- 投票引擎只决定普选、互选、提名任免等任职结果，不保存第二份管理员或岗位真源。
- 所有机构在真实法定代表人任免生效后都必须公开上链；创世没有真实任免资料时不得伪造。
- 个人多签及 `personal-admins` 完全排除在本机构岗位模型之外。

## 强制领域关系

```text
公民 cid_number 1 ─── 1 钱包账户

机构 1 ─── N 机构岗位
机构 N ─── N 管理员钱包账户
机构 + 管理员钱包账户 + 机构岗位 = 机构管理员任职
```

- 一个公民只有一个 `cid_number`，且只能绑定一个钱包账户。
- 一个钱包账户只能绑定一个公民 CID。
- 一个机构管理员就是一个取得机构管理资格的钱包账户，不新建管理员身份 ID。
- 一个管理员钱包账户可在多个机构任职，同一机构可有多个管理员。
- 管理员能否执行具体机构业务，由对应业务模块依据“机构 + 有效岗位 + 有效任职 + 业务动作”的硬规则确定。

## 当前权威字段契约

### 机构信息

`InstitutionInfo` 保留当前机构公开信息。法定代表人真实任免生效后，对所有公权、私权、创世、非创世机构统一公开：

| 字段 | 中文注释 |
|---|---|
| `legal_representative_name` | 法定代表人公开姓名 |
| `legal_representative_cid_number` | 法定代表人唯一公民 CID |
| `legal_representative_account` | 法定代表人唯一钱包账户 |

目标结构废弃 `legal_rep_name` 和 `legal_rep_cid_number`，全仓统一使用 `legal_representative_*`。法定代表人照片、联系方式和原始身份档案不上链。

### 机构岗位

`InstitutionRole` 归 `entity`：

| 字段 | 中文注释 |
|---|---|
| `role_code` | 机构内唯一岗位代码 |
| `role_name` | 岗位名称 |
| `term_required` | 该岗位是否强制任期 |
| `role_status` | 岗位是否有效 |

岗位不保存 `role_permissions`，也不建立通用权限枚举或权限表。机构信息维护、机构账户管理、机构注销、资产管理、宪法和立法等操作继续由各自业务模块的现有硬规则判定；本模型只提供机构、岗位和有效任职事实。

### 机构管理员任职

`InstitutionAdminAssignment` 归 `entity`：

| 字段 | 中文注释 |
|---|---|
| `cid_number` | 任职机构 CID |
| `admin_account` | 管理员唯一钱包账户 |
| `role_code` | 在该机构担任的岗位 |
| `term_start` | 任期开始日期，自纪元以来天数 |
| `term_end` | 任期结束日期，自纪元以来天数 |
| `assignment_source` | 任职制度来源 |
| `assignment_source_ref` | 选举、投票、登记或任免记录 ID |
| `assignment_status` | 任职是否有效 |

`assignment_source` 只允许：

- `Genesis`
- `Registry`
- `PopularElection`
- `MutualElection`
- `NominationAppointment`

任职不保存 `creator`；来源由 `assignment_source + assignment_source_ref` 唯一表达。

### 管理员集合

`public-admins` 和 `private-admins` 中的目标字段为：

```text
admins: BoundedVec<AccountId>
```

- 不再内嵌 `AdminProfile`。
- 管理员集合目标记录不保存 `creator`、`created_at`、`updated_at`；链上来源和时间由对应任职关系、事件及区块确定。
- 不保存 `admin_name`、`admin_cid_number`、`role_code`、`role_name`、`term_start`、`term_end`、`admin_source`、`admin_source_ref`。
- runtime、Node、OnChina、CitizenApp 和公民钱包中的机构 `AdminProfile` 协议及机构管理员直接变更入口均已删除。

## 信任与隐私边界

- 普通公民的原始实名档案、护照号、出生日期、住址等非公开信息不上链。
- 机构法定代表人、机构岗位任职和竞选资料属于依法公开或主动公开的身份事实，可以上链。
- 所有机构的真实法定代表人任免生效后，三个统一字段必须公开上链。
- 创世时没有真实法定代表人任免资料的机构保持“尚未任命”；不得伪造姓名、CID、账户，也不得使用 `admins[0]` 作为回退值。

## 实施步骤

1. 纠正文档和任务卡中的通用岗位权限、创世法定代表人错误口径。
2. 将法定代表人三个公开字段迁移到 `entity`，并完成全端契约对齐。
3. 在 `entity` 建立机构岗位和任职；2026-07-17 更正为 `admins` 独立保存姓名与账户，不由任职派生。
4. 接通创世、注册局和现有普选/互选来源，建立可供未来业务调用的通用机构治理结果底座；不提前实现具体业务细则。
5. 改造 OnChina、CitizenApp 和公民钱包的管理与展示。
6. Node 模型改造、全仓残留清理与真实运行态验收；重新创世单独执行。

## 各步确认规则

- 每一步必须先输出完整技术方案和预计修改目录。
- 用户确认后才能执行该步骤。
- 涉及 `citizenchain/runtime/` 的每一步都必须单独获得 runtime 二次确认。
- 每步代码执行完成后，必须立即更新文档、完善中文注释、删除旧代码、旧字段、旧注释、旧协议和旧文档口径。

## 第一步执行记录

- [x] 修正“链上不保存真实身份”过宽口径，区分普通公民隐私与依法公开身份。
- [x] 确认真实法定代表人任免生效后三个字段必须公开上链，创世不得伪造或回退到 `admins[0]`。
- [x] 确认机构岗位和任职关系归 `entity`；2026-07-17 更正为管理员姓名与账户人员集合归 `admins`。
- [x] 删除无代码依据的 `role_permissions` 和通用岗位权限口径。
- [x] 确认任职只记录制度来源，不存在 `creator`。
- [x] 确认个人多签完全排除在本任务的机构岗位模型之外。
- [x] 统一登记新字段命名和目标协议。

## 第二步执行记录

- [x] `InstitutionInfo` 新增 `legal_representative_name`、`legal_representative_cid_number`、`legal_representative_account`，公权与私权机构使用同一字段顺序。
- [x] 运行期机构创建强制三字段非空，并将三字段纳入 call index 5 注册局签名域；原 call index 2 登记凭证保持自身现行字段契约，不建立兼容分支。
- [x] 法定代表人不是创世必填项；创世阶段三字段允许全部为 `None`，不准备法代资料常量，
  不从 `admins[0]`、机构主账户或其它现有钱包推导占位值。后续依法任命时再原子写入三字段。
- [x] 删除 `public-admins`、`private-admins` 中法定代表人 storage、setter、getter 和个人多签占位实现；立法签署改为读取 entity 唯一真源。
- [x] 对齐 node、OnChina、CitizenApp、公民钱包的 SCALE 解码、DTO、数据库字段、签名构造和公开展示字段。
- [x] 删除目标代码中的 `legal_rep_*` 旧命名；OnChina 仅保留启动时删除旧数据库列的清理 SQL，不读取或兼容旧列。
- [x] 验证：runtime 相关 148 项单元测试通过；node CID 生命周期守卫 14 项测试通过；OnChina 131 项测试通过；OnChina 前端构建通过；CitizenApp 目标 10 项测试通过；CitizenWallet 69 项测试通过；node、runtime、OnChina 编译通过。
- [x] 真实运行态：使用当前源码重建 WASM 并启动 `citizenchain-fresh`，节点守卫与 RPC 正常；RPC 读取 NRC `InstitutionInfo` 确认法定代表人三字段全部为 `None`。临时 PostgreSQL 完成 49,593 个机构和 99,186 个账户的真实链投影，旧 `legal_rep_*` 列为 0；真实 HTTP 接口返回三字段，前端首页返回 200。验收后已停止进程并删除临时数据库。
- [x] 整 runtime lib 测试被仓库既有 `runtime_upgrade::Proposal` 测试缺少 `expected_pow_params_hash/new_pow_params` 阻断，该错误不属于本步骤，未越界修改。

## 第三步执行记录

- [x] 在 `entity-primitives` 建立 `InstitutionRole`、`InstitutionAdminAssignment`、岗位/任职状态和五类任职来源的统一 SCALE 类型。
- [x] 公权与私权 `entity` 新增按“机构 CID + 岗位代码”存储的岗位目录和任职关系；初始管理员钱包集合由有效任职去重派生，不再由调用方重复提交第二份名单。
- [x] 此旧签名域已于 2026-07-17 删除；当前创建凭证覆盖最小身份与 `admins(admin_name + admin_account)`，不携带岗位或任职。
- [x] `public-admins`、`private-admins` 的机构记录收口为 `cid_number + institution_code + admins + status`，其中 `admins` 只编码钱包账户；删除 `AdminProfile`、机构管理员创建人、创建/更新时间、岗位资料和任职来源副本。
- [x] 新增不含 `creator` 的机构管理员生命周期接口；个人多签继续使用其独立账户模型和管理员变更流程，不与机构管理员接口混用。
- [x] 删除公权/私权 admins 中旧机构管理员集合变更 extrinsic、投票回调、Pending 创建路径、旧事件和错误；机构管理员变更必须在第四步由任职结果驱动。
- [x] 五类固定创世机构由 `runtime/genesis/src/institution/seeder.rs` 实际写入岗位、任职和纯管理员账户：国家/省储委会为委员，省储行为董事，国家司法院为 7 护宪大法官、1 首席、2 次席、5 大法官，联邦注册局为 43 个省专员岗位且每岗 5 人。
- [x] 联邦注册局不再保存 43 个虚拟管理员组；省级权限统一按 FRG 机构 CID、稳定省专员岗位代码和有效任职查询。
- [x] Node Guard 与创世共用 `primitives/governance_skeleton.rs` 中的固定机构、岗位代码和席位协议清单；
  `runtime/genesis/src/institution/fixed_roles.rs` 负责五类岗位、席位与既有钱包索引映射，
  `runtime/genesis/src/institution/seeder.rs` 是岗位/任职/管理员账户的唯一实际写入方；清单和映射均不写 storage。
- [x] Node Guard 当前按 `InstitutionAdmins`（storage key 为 CID，value 为机构码与姓名/钱包人员集合）共享类型校验；
  删除旧 `AdminProfile`、creator/时间字段和不存在的 FRG 虚拟省组规则。FRG 省专员席位仍由 entity 任职真源表达。
- [x] runtime 新增创世逐项验收，核对固定岗位席位、`Genesis` 来源及每个常量钱包账户；公权/私权 entity、admins、multisig 和 runtime 测试已恢复编译并通过目标测试。
- [x] 创世模块已拆分为 `institution/mod.rs + fixed_roles.rs + seeder.rs`；构建前断言钱包数量等于席位总数、固定钱包无重复，创世法定代表人三字段保持全空。
- [x] 第三步创世收口验收：补齐 `no_std` 的 `alloc::vec` 宏导入；固定岗位映射 4 项、管理员 SCALE 契约 2 项、协议清单 4 项、真实 runtime 创世 1 项测试通过，runtime/node 全目标编译通过。
- [x] 第三步真实运行态：使用当前源码 production WASM 启动 `citizenchain-fresh` headless 临时节点，NodeGuard 通过、RPC 正常、block#0 可查询；创世哈希为 `0x1a3de5fdfdf75f37480b1964d7339ec7a7d38cd0716abf672dbf3ae7a4ed257e`，验收后节点正常退出。
- [x] Node Guard 已接入 `PublicManage::InstitutionRoles` 与 `InstitutionRoleAssignments`：固定机构岗位目录、NJD 7/1/2/5、FRG 43×5、任职状态/任期及任职钱包与 `admins` 集合一致性均纳入启动、普通区块、`:code` 和完整状态导入校验。
- [x] Node Guard 允许管理员、任职来源、来源引用和合法任期原子轮换；禁止固定岗位缺失、改名、停用、额外岗位、席位变化、重复占席、畸形 RAW key 和 SCALE 尾随字节；不读取法定代表人。
- [x] 固定岗位 Node Guard 验收：纯策略 8/8、`entity-primitives` 5/5、真实 block#0 完整状态和缺失/额外岗位拒绝测试通过，runtime/node 全目标编译及 production WASM 构建通过。
- [x] 固定岗位 Node Guard 真实运行态：fresh headless 临时节点启动和 RPC 正常，block#0 为 `0x1a3de5fdfdf75f37480b1964d7339ec7a7d38cd0716abf672dbf3ae7a4ed257e`，验收后节点正常退出。
- [x] `public-admins/src/weights.rs`、`private-admins/src/weights.rs`、旧 benchmark 及对应 `WeightInfo`/runtime benchmark 注册已删除；机构 admins 不再暴露可计权的管理员集合变更 extrinsic，`runtime-benchmarks` 特性编译通过。
- [x] OnChina、CitizenApp、公民钱包的机构管理员协议和界面已在第五步改为“管理员钱包集合 + entity 岗位任职”。

## 第四步 A 执行记录

- [x] `entity-primitives` 首先建立单岗位任职结果和唯一结果处理边界，第四步 B2 再彻底收口为通用复合治理结果协议。
- [x] `election-vote` 的普选、互选元数据任期统一为自纪元起 `u32` 天；终态当选结果分别映射为 `PopularElection`、`MutualElection`，以 `proposal_id` SCALE 编码作为 `assignment_source_ref`。
- [x] runtime 新增结果路由：公权机构交 `public-manage`，私权机构交 `private-manage`；未知机构码关闭失败，不建立第三份任职或管理员真源。
- [x] entity 在写入前校验目标机构、主账户、岗位状态、任期、结果账户唯一性；固定创世岗位额外按治理骨架强制法定席位数。
- [x] 此旧实现已于 2026-07-17 删除：目标岗位任职整体替换只更新岗位/任职，任职必须引用既有管理员，不再派生 admins。
- [x] 动态机构保持既有 Active 多签阈值，结果无权修改阈值；固定治理机构继续使用代码级固定阈值且不创建动态阈值 storage。
- [x] `public-admins`、`private-admins` 只保留 entity 内部同步入口，不恢复旧机构管理员变更 call、投票回调、Pending 或兼容分支。
- [x] `NominationAppointment` 仍只有强类型来源，仓库当前没有合法任免流程生产者和制度规则，本步骤未伪造任免 workflow。
- [x] 单元与 runtime 路由测试覆盖公权/私权结果替换、阈值保持、失败原子回滚、固定岗位席位拒绝、固定治理无动态阈值。
- [x] 第四步 A 验收：`entity-primitives` 5、`public-admins` 6、`private-admins` 5、`public-manage` 41、`private-manage` 39、`election-vote` 3 项测试全部通过；runtime 37/37 通过。
- [x] 第四步 A 真实运行态：使用当前源码重建 production WASM，fresh headless 临时节点成功通过 Node Guard 并启动 RPC；block#0 为 `0x2fe0183ac10abe7574c9821fa17490c5114d591df56f36d985edac358893205f`，`system_health.isSyncing=false`，验收后节点正常退出。

## 第四步 B1 执行记录

- [x] 投票引擎保持唯一 `legislation-vote` 模块；其内部固定拆为 `representative/`（单机构、顺序多机构、计票）和 `legislation/`（公投、签署、护宪）两个程序边界。
- [x] 代表机构路线使用 `RepresentativeRoute::Single/Sequential`，数学规则使用 `RepresentativeVoteRule::Regular/Major/Special`，后续程序使用 `VoteProcedure::RepresentativeOnly/Legislation`；删除引擎中的五类业务 `u8` 常量。
- [x] `legislation-yuan` 继续保存常规、常规教育、重要、重要教育、特别五类法律业务类型；教育属性只决定提案机构和代表机构路线，映射到投票引擎时复用常规/重要数学规则。
- [x] 代表元数据与法律专属元数据分离为 `RepresentativeMetas`、`LegislationMetas`；任免和预算只能创建代表表决，不写法律签署、公投和护宪元数据。
- [x] 代表计票按 `(proposal_id, body_index)` 保存，票据按 `(proposal_id, (body_index, account))` 去重；同一管理员钱包在多个机构任职时，可按各机构席位分别投票。
- [x] 业务结果继续使用同一个 `PROPOSAL_KIND_LEGISLATION` 生命周期，但 runtime 回调改为可扩展元组；法律、任免、预算分别依据 `ProposalOwner/MODULE_TAG` 认领，不新增 PERSONNEL/BUDGET 投票 kind。
- [x] call index 1 和二维码动作码 `0x1A01` 保持字节不变，业务名统一为 `cast_representative_vote`；不保留旧 API、旧动作名或旧存储兼容。
- [x] OnChina、CitizenApp、CitizenWallet 已同步新存储镜像、双 Map 键和动作名称；OnChina 大屏只读取当前 `body_index` 的票据，避免跨机构同钱包票据覆盖。
- [x] 完整回归通过：runtime 37、`legislation-vote` 32、`legislation-yuan` 30、OnChina 120、CitizenWallet 71 项测试均为 0 失败；node、runtime `no_std`、OnChina 生产构建通过，CitizenWallet 静态检查为 0 问题，CitizenApp 仅保留 2 个与本步无关的既有 info lint。
- [x] 文档和残留清理完成：固定框架已写入投票引擎技术文档与 ADR；旧调用、旧存储、旧阶段、旧 API、旧二维码动作和旧界面组件标识全仓搜索为 0，不保留兼容分支。
- [x] fresh runtime 真实运行态验收通过：使用当前源码 production WASM 启动 headless 临时节点，Node Guard 与 RPC 正常，`system_health.isSyncing=false`，block#0 为 `0xf5f7bb30535ead9b5cd5b0159b61124dd0116635ebe78b6b550eb3aa7dc169fe`；真实 metadata 包含 `RepresentativeMetas`、`LegislationMetas`、`RepresentativeTallies`、`RepresentativeVotesByAccount` 和 `cast_representative_vote`，且旧标识不存在。验收后节点正常退出，`--tmp` 临时数据已清理。

## 第四步 B2 执行记录

- [x] 单岗位整体同任期结果协议已删除，统一替换为 `InstitutionGovernanceResult`；不保留旧 trait、旧事件或兼容路由。
- [x] 单个治理结果可同时包含动态岗位定义变化、多个岗位的完整目标任职集合，以及可选的法定代表人姓名、CID、钱包账户三字段整体更新。
- [x] 每条任职独立携带任期、制度来源、来源引用和状态；不再把一个岗位内所有管理员错误压成同一任期或同一来源。
- [x] 2026-07-17 更正：公权、私权 entity 原子写岗位、任职与法定代表人，但不写独立 admins 人员集合。
- [x] 动态机构岗位允许新增、名称/任期要求变化、停用和暂时空缺；停用岗位必须同时清空任职。岗位代码作为稳定键，不提供改码路径。
- [x] 五类固定创世机构的岗位定义不可由运行期业务修改，岗位任职可以依法轮换但必须保持治理骨架法定席位。
- [x] runtime 只按机构码路由结果；`election-vote` 已改为通用协议的现有生产者，不改变普选/互选业务规则，也不新增提名任免等具体业务模块。
- [x] 治理结果没有外部 extrinsic，不改变 call index、二维码动作或现有 storage SCALE 布局；法定代表人仍不要求等于管理员。
- [x] 2026-07-17 测试已改为覆盖动态岗位、多任职、空缺岗位、法定代表人更新、admins 不随岗位变化及固定岗位保护。
- [x] B2 回归验收：entity/admins/选举/立法/runtime 共 200 项测试全部通过；node、runtime `no_std`、`runtime-benchmarks` 和 OnChina 编译通过。benchmark 检查仅保留仓库既有 `resolution-issuance` 未使用 `Hash` 警告。
- [x] B2 真实运行态：当前源码 production WASM 构建成功，压缩产物 SHA-256 为 `ce23906e713ff629d7d777f0f9905e834c49e444f6de08f8b8a722b78a5e465e`；嵌入该 WASM 的 fresh headless 节点通过 Node Guard 并启动 RPC，`system_health.isSyncing=false`，block#0 为 `0xc6d08c02c14c77305680e024e66a7226804e2cda5bb9dfd718e18868ea61c104`。真实 metadata 含 `InstitutionGovernanceApplied`、`InstitutionRoles`、`InstitutionRoleAssignments`，旧任职结果事件不存在；验收后节点和临时数据均已清理。

## 第五步执行记录

- [x] 此旧创建协议已于 2026-07-17 物理删除；当前只提交最小身份与 `admins(admin_name + admin_account)`，不提交 roles/assignments。
- [x] OnChina 链读改为联合读取 `PublicAdmins/PrivateAdmins::AdminAccounts` 钱包集合与 `PublicManage/PrivateManage` 岗位、任职；联邦注册局省域从稳定的 `PROVINCE_COMMISSIONER_<省码>` 岗位取得，不再读取虚拟省组。
- [x] OnChina 管理员展示统一改为 `InstitutionAssignmentCard`，删除旧资料卡、旧姓名/CID/岗位内嵌投影和本地管理员姓名编辑依赖；同一钱包可展示多个有效任职。
- [x] 删除 OnChina 旧 `REPLACE_GOVERNING_REGISTRY` 本地替换动作及其后端预检/写库、scope 迁移、错误码和前端按钮；FRG 岗位任职目录只读，禁止用本地投影冒充换届结果。
- [x] CitizenApp 新增机构岗位、任职强类型模型和严格 SCALE 解码；机构管理员页面联合展示钱包、岗位、任期和来源，个人多签继续使用独立账户布局。
- [x] CitizenApp 与公民钱包删除公权/私权机构管理员集合变更入口、动作码和解码分支；管理员集合变更只保留个人多签，机构管理员变化必须由 entity 治理结果原子派生。
- [x] 公民钱包机构创建冷签复核已按岗位和任职字段解码并校验岗位引用、任期、来源、状态及管理员钱包去重数量。
- [x] 验证：OnChina Rust 130/130、前端生产构建、CitizenApp 目标 15/15、公民钱包 77/77 全部通过；公民钱包静态分析 0 问题，CitizenApp 无新增问题，仅保留仓库既有 2 条 info lint。
- [x] 第五步真实运行态：fresh 无头节点通过 Node Guard 并提供 RPC，`system_health.isSyncing=false`，block#0 为 `0xc6d08c02c14c77305680e024e66a7226804e2cda5bb9dfd718e18868ea61c104`；真实 metadata 含 `InstitutionRoles` 与 `InstitutionRoleAssignments` 且不含旧 FRG 虚拟省组。OnChina 生产包经本地预览返回 HTTP 200，实际 JS 产物包含岗位码、任职来源、来源引用和管理员账户字段，且旧 FRG 本地替换动作标识不存在。验收后临时节点和预览服务均已停止。
- [x] 第五步未修改 `citizenchain/runtime/`；Node 桌面端迁移和最终残留清理已在第六步完成。

## 第六步执行记录（不含重新创世）

- [x] Node 机构管理员账户严格解码为 `cid_number + institution_code + admins + status`；删除机构姓名、公民 CID、岗位、来源、创建人和创建/更新时间内嵌镜像。
- [x] Node 在同一个 finalized block hash 上联合读取 public/private admins 钱包集合与对应 entity 岗位、任职；非法人按实际命中的 admins pallet 决定 entity 路由，不按机构码猜测归属。
- [x] Node 输出按钱包唯一聚合的 `InstitutionAdminInfo + InstitutionRoleAssignmentInfo[]`；每条任职展示岗位代码、岗位名称、任期、来源及来源引用，二进制来源引用统一显示为 hex。
- [x] Node 新增 `frontend/admins/InstitutionAssignmentCard.tsx`，治理机构、提案投票状态和清算行管理员列表统一复用；同一钱包多个岗位只显示一张卡片，投票仍按钱包唯一计算。
- [x] 删除 Node 机构管理员直接变更后端 call data、校验、签名和提交命令，删除前端集合编辑、差异、钱包选择、签名页以及所有“换管理员”入口。
- [x] Node 不再承接个人多签管理员管理；个人多签继续只由 CitizenApp 的独立 personal-admins 流程处理。
- [x] 更新 Node 管理员技术文档、ADR、MODULE_TAG 注册表、CitizenApp 技术文档和已被新模型替代的活动任务卡；旧机构管理员直接变更和资料内嵌代码在 Node、OnChina、CitizenApp、公民钱包源代码中搜索为 0。
- [x] 验证：Node `cargo check` 通过，Node 全量 273 项测试 0 失败，前端 TypeScript/Vite 生产构建通过；生产预览 HTTP 200，实际 JS 包含管理员账户、岗位任期和来源依据展示。
- [x] 本步骤没有修改 `citizenchain/runtime/`，没有生成或改写链规格，也没有初始化临时 fresh 链。本机当前没有运行中的 9944 RPC，故未伪造链上在线验收结果。
- [ ] 重新创世、真实 RPC/storage/页面联动和全端最终验收按用户要求暂缓，必须在用户另行确认后执行。

## 历史实现事实

2026-06-28 至 2026-06-30 曾经实现机构 `AdminProfile`，将管理员姓名、CID、岗位、任期和来源内嵌到 `AdminAccounts.admins`，并同步实现 OnChina、CitizenApp 和公民钱包解码。该布局已被 2026-07-12 用户确认的目标模型取代，后续步骤必须彻底删除相关代码、协议、缓存、注释和展示残留。

## 完成标准

- runtime 中不再存在机构管理员 `AdminProfile` 内嵌布局。
- 已任命法定代表人的机构都有可查询的链上三字段；尚未任命的创世机构没有占位值或 `admins[0]` 回退值。
- 机构岗位和机构管理员任职关系有唯一 entity 真源；具体业务权限由对应业务模块硬规则判定。
- `admins` 只保存管理员钱包账户集合。
- 一个管理员可在多个机构任职，一个机构可有多个管理员。
- 无有效岗位任职或任期失效的账户不具有对应机构权限。
- 个人多签行为和存储不受本机构岗位改造影响。
- OnChina、CitizenApp、公民钱包与 runtime SCALE 字节完全一致。
- 用户另行确认重新创世后，通过真实节点、真实 PostgreSQL、真实 HTTP、真实页面和真实冷签完成最终验收。
