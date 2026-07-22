# 任务卡：机构岗位权限与投票职责统一

状态：已完成。2026-07-21 第 1 至第 8 步全部实施并通过全端真实验收。投票引擎按 `VotePlan` 冻结岗位任职账户，并以 CID + 岗位码 + 钱包形成独立机构票据；同一钱包兼任多岗可分别记票且机构阈值不变。正式既有治理、发行、立法和机构业务入口已审计收口，runtime、Node、OnChina、CitizenApp、CitizenWallet、QR、公开白皮书和生成文档均已统一到最终模型。

## 最终需求

- 机构授权主体唯一表示为 `RoleSubject { cid_number, role_code }`，即“机构 CID + 机构内岗位码”。
- `admins` 只是机构可任职人员集合。管理员账户本身没有业务权限；只有该账户对某个 `RoleSubject` 存在有效任职时，才取得该岗位被授予的业务动作权限。
- 机构 CID 决定机构可拥有的顶层业务能力；岗位权限必须落在该 CID 的顶层能力范围内。具体业务发起、投票等权限由 `RoleBusinessPermission` 绑定 `RoleSubject + BusinessActionId + operation`。
- 业务模块负责前置校验 `RoleSubject` 权限、静态选择唯一投票引擎、绑定业务对象和参与主体，并在投票通过后执行具体业务。投票引擎不得由管理员、岗位或调用参数选择，也不得解释或执行具体业务。
- 全部机构与个人多签状态变更必须经过指定投票引擎；只读、创世写入、投票引擎自身维护及已通过提案的确定性回调不重新发起投票。
- 协议升级和决议发行采用相同联合权限：NRC/43 个 PRC 委员岗位可发起并投票，43 个 PRB 正式 `DIRECTOR / 董事` 岗位只参与投票；资格按完整 `RoleSubject` 判断，不按机构全体 admins 判断。
- 所有机构必须永久存在唯一 `LR / 法定代表人` 岗位，允许空缺且任职只能为 0 或 1 人，但不得改名、删除或替换岗位码；法定代表人原子结构必须与 LR 任职一致。
- 所有创世固定岗位的岗位码、岗位名和岗位权限永久固定，包括“公民链技术发展基金会”的 `LR`、`GENESIS_PRODUCT_MANAGER`、`GENESIS_PROGRAMMER`。创世机构仍可依法增加普通动态岗位。
- 除固定岗位外，动态 `role_code` 由 runtime 生成，在机构内唯一、不可修改、删除后永不复用；`role_name` 可依法新增、修改和删除。岗位权限随岗位码固定，改变权限必须删除旧岗位并生成新岗位码。
- `role_name` 同样在机构内唯一；同名多人属于同一个岗位的多个席位。一个管理员可以担任多个不同岗位，同一岗位内不得重复占席。岗位没有独立阈值，阈值属于机构或业务投票计划。
- 机构阈值与 `admins` 人数独立：一个人可以用同一钱包在同一机构兼任多个岗位，不能因为钱包去重而降低机构阈值。最终票据去重主体必须是“提案 + 机构 CID + 岗位码 + 任职钱包”，不是裸钱包。
- 动态岗位码格式为 `R_<32 位大写十六进制>`。随机材料统一为 `blake2_256(SCALE(GMB_ROLE_V1, cid_number, institution_role_nonce, proposal_id))` 的前 16 字节；调用方不得提交岗位码，`UsedRoleCodes[(cid_number, role_code)]` 永久保留已用记录。
- 个人多签使用独立 `AuthorizationSubject::PersonalMultisig`，不混入机构岗位授权模型。
- 公权机构管理员统一为 `PublicAdmin { admin_account, cid_number, family_name, given_name }`；当前允许公民 CID、姓、名为空，非空 CID 必须是 CTZN 且与 `citizen-identity` 的钱包绑定完全一致。私权机构和个人多签继续使用 `Admin { admin_account, family_name, given_name }`。
- 私权创世机构统一为非营利法人“公民链技术发展基金会”（简称“公民链基金会”）；CID `GZ018-SFGYR-201206100-2026`，主账户 `0xe86aa3cd794651257dea9b7cad1abc4f0ce05940c1aecccd2ed8dd2fc9907023`，费用账户 `0xaa23304c7b663ba25a9d3a2fb1efafdd650ecf2504a2caedc228fe81b46b4333`。程伟以同一钱包同时任职 `LR`、`GENESIS_PRODUCT_MANAGER`、`GENESIS_PROGRAMMER`，管理员人员名册只保存一条，机构阈值保持 2。

开发期无正式用户数据：按最终结构重新创世，不保留旧授权、旧载荷、旧 storage、旧命名或双轨兼容。

## 职责边界

```text
管理员账户
  └─ 有效任职 ─> RoleSubject(cid_number, role_code)
                    ├─ 业务模块校验 Propose / Vote 权限
                    ├─ 业务模块静态选择投票引擎并绑定 VotePlan
                    └─ 投票引擎快照合格任职账户、计票、判定终态
                                         └─ 业务模块执行已绑定业务
```

- `admins`：人员名册和任职候选范围，不是业务授权真源、投票主体或费用回落付款人。
- `entity`：岗位、岗位权限、岗位码 nonce/永久占用、任职、法定代表人及机构治理阈值真源；阈值与 admins 人数解耦。
- 业务模块：业务动作权限、指定投票引擎、业务对象绑定和通过后执行真源。
- `votingengine`：资格快照、提案阈值快照、票据、计票、终态和维护真源；只消费机构阈值，不保存第二份机构配置，不执行具体业务。
- `NodeGuard`：只永久保护强制 `LR`、创世固定岗位及其固定权限；不得禁止创世机构新增普通动态岗位。

## 分步骤实施

1. **架构、命名与协议冻结（已完成）**：新增 ADR-039，修订 ADR-023 与统一命名/协议/模块文档，冻结职责边界、动态岗位码算法和分步计划。
2. **共享授权类型与跨端 SCALE 契约（已完成）**：实现 `RoleSubject`、`BusinessActionId`、`RoleBusinessPermission`、`AuthorizationSubject`、`VotePlan` 等强类型，并在 runtime、Node、OnChina、CitizenApp、CitizenWallet 同步字段序和 fixture。
3. **岗位、权限、任职与生命周期（已完成）**：在 public/private entity 实现岗位权限、动态码生成、永久不复用、岗位增删改名、任职有效性、强制 LR 保护和统一授权查询；关闭无法满足原子初始化要求的旧机构直接创建 call 5。
4. **创世固定岗位与 NodeGuard**：第 4A 只读盘点并确认完整矩阵；第 4B 固化全部创世岗位及权限，纳入公民链基金会三个固定岗位，并允许保护机构增加普通动态岗位。
5. **投票引擎按岗位主体快照（已完成）**：机构投票从全体 admins 快照改为 VotePlan 指定的一个或多个 `RoleSubject` 有效任职账户快照；个人多签保持独立主体。第 5A 完成 joint-vote，第 5B 完成 internal-vote，第 5C 完成 legislation-vote、election-vote 及法律业务入口。
6. **基础权限路径收口（已完成）**：复核岗位维护、任职、管理员更换、账户关闭、机构转账和个人多签边界；删除投票引擎旧机构管理员快照辅助函数与 provider 接口，确保 `AdminSnapshot` 只服务个人多签。本步不实施机构登记或机构 CRUD。
6A. **管理员类型、机构阈值与基金会校正（已完成）**：拆分公权四字段和私权/个人三字段管理员；机构阈值迁入 public/private entity；同一程伟钱包任职基金会三个固定岗位，不改阈值。
6B. **岗位席位记票（已完成）**：机构票据唯一键已改为岗位任职席位；同一钱包兼任多个岗位时使用同一私钥分别行使各岗位票权，机构阈值未修改。
7. **剩余治理、发行和公共业务权限收口（已完成）**：已审计现有协议升级、GRANDPA、销毁、决议发行、选举、立法及已经存在的公共业务入口；正式可达入口均按岗位权限和指定投票引擎收口。机构登记、机构 CRUD 与 OnChina 机构管理不属于本任务。
8. **全端真实验收与残留清理**：完成 UI/QR、真实 fresh runtime、真实服务和页面验收，删除全部旧管理员授权与旧投票主体口径，回写最终文档。

每一步必须先输出完整技术方案并等待确认；任何涉及 `citizenchain/runtime/` 的步骤还必须列出完整 runtime 路径，取得该步单独二次确认后才能执行。

## 预计修改目录

- `memory/04-decisions/`：记录 ADR-039 并修订 ADR-023；仅文档，不涉及代码。
- `memory/01-architecture/`、`memory/05-modules/`、`memory/07-ai/`、`memory/08-tasks/`：统一架构、模块、命名、协议和任务状态；包含文档修订与旧口径清理。
- `citizenchain/runtime/entity/`：已实现岗位、岗位权限、任职、动态岗位码、统一授权查询和旧直接创建入口关闭；后续只按已确认步骤接入创世与业务模块。
- `citizenchain/runtime/admins/`：已收口 admins 为人员名册并移除管理员即业务授权语义；涉及 runtime 代码、测试和残留清理。
- `citizenchain/runtime/votingengine/`：已完成全部现有 Track 的 VotePlan、岗位资格快照和岗位席位票据；涉及 runtime 代码、测试和残留清理。
- `citizenchain/runtime/{governance,issuance,public,transaction,misc}/`：正式既有业务已接入准确岗位权限与固定投票引擎；禁用业务骨架保持禁用，未改变其业务规则。
- `citizenchain/runtime/primitives/`、`citizenchain/runtime/genesis/`、`citizenchain/runtime/src/`：已承载共享常量、创世固定岗位/权限、runtime 接线与禁用入口过滤；涉及 runtime 代码、测试和残留清理。
- `citizenchain/node/`：已同步 NodeGuard、交易构造和解码；第 8 步只做最终真实验收及发现问题后的已确认修正。
- `citizenchain/onchina/`：已同步机构工作台授权、链读写和岗位管理，并完成真实服务、临时数据库、链投影与页面验收。
- `citizenapp/`、`citizenwallet/`：已同步提案、投票、QR/SCALE 解码和展示，并完成全量测试、静态分析与离线签名验收。
- `citizenweb/`：公开白皮书与治理页面统一机构岗位权限、个人多签管理员快照和 citizen-identity 人口数据边界；涉及现有文档页面、生产构建和旧口径清理。

## 总体验收

- 任一管理员未担任有效岗位时，不能发起或参与任何机构业务；任一权限判定都能追溯到完整 `RoleSubject`。
- 同一管理员可在多个机构、多个岗位任职，各权限互不串用；同机构不同岗位不能继承彼此权限。
- 每个业务动作只使用其代码静态指定的投票引擎，调用方无法改选；投票引擎不包含业务执行逻辑。
- 联合投票按 VotePlan 中的岗位主体分别快照和计票，不把参与机构全部 admins 自动纳入。
- `LR` 永久存在并允许空缺；创世固定岗位不可变；普通动态岗位码在机构内唯一、删除后永不复用，域分隔符精确为 `GMB_ROLE_V1`。
- runtime、Node、OnChina、CitizenApp、CitizenWallet 的 SCALE/QR/字段命名逐字节一致。
- 完成相关编译、单测、no_std、clippy、fresh 链、NodeGuard、真实 HTTP/页面/签名和投票执行验收；最终无旧模型残留。

## 进度

- [x] 第 1 步：架构、命名与协议冻结
- [x] 第 2 步：共享授权类型与跨端 SCALE 契约
- [x] 第 3 步：岗位、权限、任职与生命周期
- [x] 第 4A 步：创世固定岗位权限只读盘点与推荐矩阵
- [x] 第 4B 步：创世固定岗位权限与 NodeGuard 实施
- [x] 第 5A 步：联合投票 VotePlan/岗位快照及协议升级、决议发行接入
- [x] 第 5B 步：内部投票 VotePlan/岗位快照及现有内部投票业务接入
- [x] 第 5 步：投票引擎按岗位主体快照
- [x] 第 6 步：基础权限路径收口
- [x] 第 6A 步：公私权管理员分型、机构阈值解耦与公民链基金会重新创世
- [x] 第 6B 步：岗位席位记票
- [x] 第 7A 步：剩余业务权限只读盘点
- [x] 第 7 步：治理、发行和公共业务接入
- [x] 第 8 步：全端真实验收与残留清理

## 第 2 步完成记录（2026-07-19）

- runtime 共享类型已落在 `entity-primitives`，`VotePlan`、`VotingEngineKind` 与构造期主体组合校验已落在共享 `votingengine` crate；未写入现有提案 storage，未改变 pallet/call/storage index。
- `RolePermissionOperation`、`AuthorizationSubject`、`VotingEngineKind` 的 discriminant 和所有结构字段序均由 Rust 单测锁定。
- Node 使用共享 Rust 类型读取统一金标；OnChina、CitizenApp、CitizenWallet 对同一金标严格解码。CitizenApp 与 CitizenWallet 拒绝非法主体组合和尾随字段。
- 唯一跨端金标为 `memory/06-quality/fixtures/institution_role_permission_v1.json`，不建立端侧私有协议副本。
- 验收通过：`entity-primitives` 6 项、`votingengine` 4 项、Node fixture 1 项、OnChina fixture 1 项；两个共享 crate 的 `no_std` 检查通过；CitizenApp 6 项、CitizenWallet 97 项目标测试全部通过，目标 Dart 文件 analyze 无问题。
- 本步骤目标 crate 在仅屏蔽既存 lint（`too_many_arguments`、`unnecessary_lazy_evaluations`、`manual_is_multiple_of`、`type_complexity`）后通过 `clippy -D warnings`；不屏蔽时还会被既存 `primitives` 数字分组等 45 项和上述既存 lint 拦截，本步骤未越界修改这些旧文件。
- `citizenchain/Cargo.lock` 仅为 `votingengine` 增加已确认的 `entity-primitives` 依赖；Square Post 的既有未提交依赖改动保持不动。

## 第 3 步完成记录（2026-07-19）

- public/private entity 新增 `InstitutionRolePermissions`、`InstitutionRoleNonce`、`UsedRoleCodes`，storage version 直接提升为 2；当前链无创世、无数据，因此不保留 migration 或旧 storage 兼容。
- 动态岗位创建不接收岗位码，由 runtime 使用精确域分隔符 `GMB_ROLE_V1`、CID、单调 nonce 和真实 `proposal_id` 生成 `R_<32 位大写十六进制>`；删除后只清理当前岗位、权限和任职，永久占用记录不删除。
- `InstitutionRoleMutation::{Create,Rename,Delete}` 已替代旧岗位变更布局。Create 原子写入不可变权限和初始任职；Rename 只改动态岗位名；Delete 禁止删除 LR 和创世固定岗位。保护创世机构仍可增加普通动态岗位。
- 任职有效性统一为 UTC 日：有任期岗位使用闭区间 `[term_start, term_end]`，允许同日起止；无任期岗位必须精确为 `0/0`。授权查询同时校验 CID 存在、origin 属于 admins、有效任职、完整权限主体、业务动作、操作类型和 CID 顶层能力。
- PublicManage/PrivateManage 的旧 call index 5 已从 runtime metadata、权重、费用路由、OnChina 构造器、QR registry 和 CitizenWallet decoder 全部删除并永久留洞。OnChina 创建 API 在第 6 步新业务模块落地前固定返回 501，禁止恢复旧直接创建路径。
- Node 已读取岗位权限 storage 并按 runtime 相同 UTC 日规则过滤任职；OnChina 已同步新治理载荷；CitizenApp/CitizenWallet 已同步严格 SCALE 解码，旧岗位布局和两个 call 5 动作均拒绝。
- 验收通过：共享/entity 目标测试 37 项、runtime 46 项、Node 岗位管理 6 项、NodeGuard 当前骨架 11 项、OnChina 5 项、QR registry 6 项、CitizenApp 8 项、CitizenWallet 107 项；目标 no_std、目标 clippy（仅屏蔽既存跨依赖 lint）、Node/OnChina check、OnChina production build 和 `git diff --check` 均通过。
- fresh `--dev --tmp` 节点真实初始化 genesis、开放 RPC、返回 metadata 并运行到 best #9；同时复现 NodeGuard 启动自检 `AdminAccountDecodeFailed(NRC)`。启动守卫按既定边界记录错误后继续服务，问题属于第 4 步真实创世/NodeGuard 对拍，不能在第 3 步伪装为已通过。
- OnChina 新前端已把创建按钮固定禁用并明确显示原子创建前置条件，后端创建 handler 固定返回 501；生产构建通过。带真实管理员会话的 HTTP/页面回归统一留到第 8 步，当前不使用旧载荷做兼容验收。
- 全仓 `cargo fmt --all -- --check` 仍被用户既有 Square Post 测试文件的单处换行差异拦截；第 3 步全部目标 Rust 包格式检查通过，未越界格式化该文件。
- 第 4B 已取得矩阵确认、新文件确认和 runtime 单独二次确认；实现严格采用重新创世，不保留旧机构身份、旧权限或旧管理员布局兼容。

## 第 4A 步盘点结果与固定矩阵（2026-07-19，已确认并实施）

### 盘点结论

- 第 4B 已把 `InstitutionRolePermissions` 写入当前源码生成的创世态，并把 `RuntimeInstitutionCapabilityPolicy` 改为准确 CID/机构身份的显式白名单；未知组合继续 fail-closed。第 5 至第 6 步已完成现有治理、发行、立法、机构转账和本机构治理入口切换。`citizen-identity` 与 `address-registry` 属注册局对外业务，不能因为由机构岗位操作就误改为机构内部投票。
- 用户最终确认协议升级与决议发行相同，由 NRC 和 43 个 PRC 的 `COMMITTEE_MEMBER` 共同拥有 `Propose + Vote`；不再采用早期 NRC 单方草案。
- 决议发行按已确认目标：NRC/PRC `COMMITTEE_MEMBER` 可发起和投票，PRB `DIRECTOR` 只投票。
- FRG 是一个 CID、215 名 admins、43 个 `PROVINCE_COMMISSIONER_<省码>` 岗位；省级业务只能绑定目标省岗位的 5 人快照，禁止退回全体 admins。机构级动作若未来需要跨省共同决定，必须由业务 `VotePlan` 明确列出参与岗位，不能复用当前 FRG `3/5` 省组阈值代表全机构。
- `address-registry`、`citizen-identity` 的直接写入是对外行政办理路径，不是机构内部决策。本任务只冻结稳定业务动作标识和岗位权限，不改写实际签名规则，不接入机构内部投票。
- 第 3 步记录的 `AdminAccountDecodeFailed(NRC)` 已只读定位：当前运行节点使用的冻结 block#0 中，`PublicAdmins::AdminAccounts[NRC]` RAW 值仍是旧的 19 个纯账户数组（机构码后直接为 `Compact(19) + 19 * AccountId32`，共 613 字节），没有当前 `Admin { admin_account, family_name, given_name }`。`node/src/core/command.rs` 又把 `--dev` 映射到冻结 chainspec，所以该错误是旧冻结创世与当前代码不一致，不是 NodeGuard 当前共享解码器错误。禁止增加旧布局兼容；第 4B 真实验收必须以当前源码/WASM 生成隔离 `citizenchain-fresh` 临时创世，正式 chainspec 重烤仍放在最终创世流程。
- 固定岗位权限永久不可修改，因此采用最小授权：没有已确认固定职责的能力不授予。未来业务应由机构依法新建动态岗位承接，不能借 runtime 升级给既有固定岗位追加权限。
- 除法律行政签署/三人会签机构和公民链基金会外，受保护创世机构的 `LR` 固定为空权限。法律相关 LR 只拥有 `leg-yuan/0,1,2 Vote`，不得发起；公民链基金会 `LR` 仍是该准确 CID 的单独固定规格。

### 稳定业务动作目录

`BusinessActionId.action_code` 是独立稳定业务码，不以 pallet call index 作为鉴权真源；下列数值为便于审计而与现有稳定 call/action 号对齐，后续即使交易入口调整也不得复用或改义。

| `module_tag/action_code` | 业务语义 | 当前状态 |
|---|---|---|
| `pub-mgmt/3`、`pri-mgmt/3` | 本机构管理员、岗位、任职和法定代表人治理 | 第 5B 已按提案岗位权限发起、岗位有效选民快照投票 |
| `rt-upg/0` | 协议升级 | 第 5A 已按 NRC/PRC 委员发起、87 个岗位主体联合投票 |
| `res-iss/0` | 决议发行 | 第 5A 已按 NRC/PRC 委员发起，PRB 董事只投票 |
| `res-dst/0` | 决议销毁 | 第 5B 已按提案岗位权限发起、岗位有效选民快照投票 |
| `gra-key/0` | GRANDPA 密钥更换 | 第 5B 已按委员岗位权限发起、岗位有效选民快照投票 |
| `multisig/0` | 机构普通转账 | 第 5B 已按提案岗位权限发起、岗位有效选民快照投票；个人多签保持独立管理员主体 |
| `multisig/1` | NRC 安全基金转账 | 第 5B 已按 NRC 提案岗位权限发起、岗位有效选民快照投票 |
| `multisig/2` | 费用账户划转主账户 | 第 5B 已按 NRC/PRB 提案岗位权限发起、岗位有效选民快照投票 |
| `onc-iss/10..14` | NRC 冻结、解冻、扣押、强制划转、整币封禁监管 | 只冻结了稳定动作与固定岗位权限；runtime call filter 禁用整个 pallet，当前没有投票提案或终态回调，不能视为可用业务 |
| `leg-yuan/0,1,2` | 新法、修法、废法；LR 承担行政签署/三人会签，NJD 护宪岗位只参与修宪终审 | 第 5C 已按岗位权限发起并冻结代表、签署、护宪资格 |
| `sqr-sub/5` | 公民链基金会平台会员价格调整 | 第 5B 已按产品经理岗位发起、三个固定岗位有效任职投票 |
| `cit-id/0..4,6..8` | 公民投票身份登记、升级、更新、注销及 CID 占号/批量占号/吊销 | 注册局对外办理公民身份的业务；按业务规则校验注册局岗位和公民本人签名，不接入机构内部投票 |
| `addr-reg/0..4` | 地址库版本、地址名称和完整地址的增删改 | 注册局对外登记业务；不因操作人属于机构就自动转成机构内部投票，具体签名规则不在本任务改写 |
| `ins-reg/0` | 历史冻结的机构登记预留动作 | 当前没有独立业务模块；机构登记、机构 CRUD 与 OnChina 机构管理已明确排除出本任务，不作为第 7 步完成条件 |

### 固定岗位权限

表中 `P` = `Propose`，`V` = `Vote`；未列出的动作一律无权限。

| 固定机构/岗位 | 推荐固定权限 |
|---|---|
| NRC `COMMITTEE_MEMBER` | `pub-mgmt/3 P+V`；`rt-upg/0 P+V`；`res-iss/0 P+V`；`res-dst/0 P+V`；`gra-key/0 P+V`；`multisig/0,1,2 P+V`；`onc-iss/10..14 P+V` |
| 每个 PRC `COMMITTEE_MEMBER` | `pub-mgmt/3 P+V`；`rt-upg/0 P+V`；`res-iss/0 P+V`；`res-dst/0 P+V`；`gra-key/0 P+V`；`multisig/0 P+V` |
| 每个 PRB `DIRECTOR` | `pub-mgmt/3 P+V`；`rt-upg/0 V`；`res-iss/0 V`；`res-dst/0 P+V`；`multisig/0,2 P+V` |
| NJD `CHIEF_JUSTICE` | `pub-mgmt/3 P+V`；只由首席大法官发起司法院本机构治理 |
| NJD `DEPUTY_CHIEF_JUSTICE`、`JUSTICE` | `pub-mgmt/3 V` |
| NJD `CONSTITUTION_GUARD` | `pub-mgmt/3 V`；`leg-yuan/1 V`，后者只用于修宪护宪终审 |
| PRS/NLG/NRP/NSN/NED、PGV/PLG/PRP/PSN、CGOV 的 `LR` | `leg-yuan/0,1,2 V`，仅承担行政签署或国家/省级三人会签，不得发起法律提案 |
| FRG 每个 `PROVINCE_COMMISSIONER_<省码>` | `pub-mgmt/3 P+V`（仅本省岗位任职治理）；`cit-id/0..4,6..8 P+V`；`addr-reg/0..4 P+V`；当前创世还含历史 `ins-reg/0 P+V` 预留，但本任务不建设或接入机构登记模块；实际省级业务必须按目标省码绑定同一个省岗位 |
| 公民链基金会 `LR` | `pri-mgmt/3 P+V`；`sqr-sub/5 V` |
| 公民链基金会 `GENESIS_PRODUCT_MANAGER` | `pri-mgmt/3 P+V`；`sqr-sub/5 P+V` |
| 公民链基金会 `GENESIS_PROGRAMMER` | `pri-mgmt/3 P+V`；`sqr-sub/5 V` |
| 其他公权创世机构的 `LR` | 空权限；岗位永久存在且允许空缺，但不继承委员、董事、司法或注册权限 |

### 明确不授予的能力

- 公民链基金会三个固定岗位不取得协议升级权限；协议升级由 NRC/PRC 委员岗位发起和投票，PRB 董事岗位只投票。
- NRC/PRC/PRB/NJD/FRG/公民链基金会固定岗位均不默认取得普通链上资产发行权限；确有需要时新建动态岗位。
- NJD、FRG 和公民链基金会固定岗位不默认取得机构转账权限；应先通过本机构治理新建财务动态岗位，避免把司法、注册、产品或程序岗位永久变成财务岗位。
- `developer_direct_upgrade` 使用协议升级同一业务权限并显式携带 NRC CID、`COMMITTEE_MEMBER` 岗位码和签名钱包；正式创世前必须保持关闭，后续不得用开发入口绕过投票。
- entity 当前 call 6/7/9 的注册局直接更新信息、加账户、登记 admins 不作为固定权限继续保留；第 6/7 步应由原子登记或本机构治理业务替代并清理旧直写路径。

### 第 4B 实施确认

- 用户已确认本节动作 tag/code 与完整岗位矩阵，并最终修正协议升级与决议发行完全一致：NRC/PRC 委员 `P+V`、PRB 董事 `V`。
- 第 4B 全部 `citizenchain/runtime/` 路径、新增 `business_action.rs` 及 Git 跟踪均已取得明确确认和 runtime 单独二次确认。
- 创世私权机构后续已按最终确认重新定义为非营利法人“公民链技术发展基金会”；最终值见第 6A 完成记录，不做 migration、旧名兼容或双读。

## 第 4B 步完成记录（2026-07-19）

- `entity-primitives/business_action.rs` 成为稳定业务动作和创世固定岗位权限唯一目录；固定权限以完整 CID + 岗位码生成，不向管理员账户直接授权。协议升级与决议发行均为 NRC/PRC `P+V`、PRB `V`。
- public/private entity 创世入口把每个固定岗位权限写入 `InstitutionRolePermissions`，包括公权固定机构永久空权限的 `LR`；`UsedRoleCodes` 同步永久占用。FRG 43 个省专员岗位各自保留同省边界。
- runtime CID 顶层能力先核对机构实际存在于唯一公权或私权 storage，再按准确受保护身份开放固定白名单；普通机构仍只开放自身岗位治理，未知、跨 CID 或超范围组合拒绝。
- NodeGuard 精确校验每个固定岗位的完整权限数组、主体 CID/岗位码、动作和 `Propose/Vote`，并校验受保护私权创世机构的固定身份；固定权限缺失、解码失败或变化均拒绝。创世机构新增普通动态岗位不再被误判为额外固定岗位。
- 该私权创世机构的最终法人类型、CID、账户和管理员/岗位布局已在第 6A 重新创世，以第 6A 完成记录为唯一当前口径。
- 验收通过：`entity-primitives` 10 项单测、runtime 两项创世权限目标测试、NodeGuard 12 项目标单测；当前源码完整 block#0 经 NodeGuard 全策略一次扫描通过，删除固定岗位被拒绝且动态岗位 key 被接受。最新源码 WASM 通过 `citizenchain-fresh --tmp` 隔离节点真实启动，RPC `isSyncing=false`，创世哈希 `0x0b0d3f1a601660d8a884cb732e82f6ee7e1403a5dd981e2f11d47a819ec93bda`、state root `0x297e29646bad2d1c24397294f17bfb9d7528a16c67245b660c917487f88727d6`；验收节点已停止。正式冻结 chainspec 未在本步骤重烤，继续由最终创世流程唯一生成。

## 第 4B 确认后校正记录（2026-07-19）

- 按最终确认把协议升级补齐为与决议发行完全相同的联合权限矩阵：NRC/43 个 PRC `COMMITTEE_MEMBER` 为 `Propose + Vote`，43 个 PRB 正式 `DIRECTOR / 董事` 为 `Vote`，PRB 不得发起两类业务。
- 机构内岗位码和岗位名分别唯一；同名多人统一写入一个岗位的多个任职席位。管理员可以兼任不同岗位，同一岗位内同一账户不得重复占席。
- 所有机构唯一 `LR` 岗位的任职区间统一为 0..=1；法定代表人姓名、个人 CID、账户三字段与 `LR` 任职必须原子设置或原子清空。公民链基金会的三个固定岗位各一席，并允许同一管理员钱包跨岗兼任。
- NodeGuard 不再要求固定岗位任职并集等于 admins，也不再禁止跨岗位兼任；仍要求每个固定岗位精确满足自身席位边界、任职账户属于本机构 admins、同一岗位内不重复，并保护固定权限数组不变。
- 校正专项已通过：`entity-primitives` 10 项、`primitives::governance_skeleton` 7 项、`public-manage` 14 项、`private-manage` 14 项、NodeGuard 治理骨架 12 项，以及 runtime 法定代表人原子一致性、创世固定权限、固定席位路由 3 项测试；四个目标 crate 的 `no_std` 检查与 `git diff --check` 均通过。
- 当前源码强制重建 WASM/Node 后，`citizenchain-fresh --tmp` 隔离链真实启动且 RPC `isSyncing=false`；新创世哈希为 `0xa96d68d95c9f05f6ad893c93e4738e993bf031eb9eebb64506f0d7174418c805`，state root 为 `0x630708630485316c372a182a0ede4670c3d487353f8462719f58dc03ec4ec5c6`。19944 临时进程已停止，遗留 `substratePvmy8J` 临时目录已移入废纸篓；正式 chainspec 未重烤。

## 第 5A 步完成记录（2026-07-19）

- 共享 `votingengine` 当时新增 `ProposalVotePlans` 和按完整 `RoleSubject` 存储的 `VoterSnapshot`，并使用按 CID 合并钱包的过渡快照；第 6B 已删除该过渡结构。当前链无正式数据，不保留 migration、旧提案兼容或联合投票双轨 storage。
- `JointVoteEngine` 创建接口必须携带 `VotePlan`，旧的裸创建入口已删除。联合引擎要求 proposer 是 plan 中 NRC/PRC 委员主体的有效任职账户，并要求选民主体精确覆盖 NRC + 43 PRC 委员和 43 PRB 董事。
- 第 5A 当时同一账户在同一 CID 的多个参与岗位会被过渡快照合并；第 6B 已改为每个 `RoleSubject + admin_account` 一张独立票据。机构联合提案始终不写 `AdminSnapshot`。
- 协议升级和决议发行调用均显式携带岗位码，并在业务模块前置校验 `RoleSubject(actor_cid_number, proposer_role_code)` 的对应 `Propose` 权限；当前权限目录只允许 NRC/PRC `COMMITTEE_MEMBER`。两者均构造 87 个岗位主体的固定联合 `VotePlan`；PRB `DIRECTOR` 只在 voter subjects 中，不能发起。普通 staff 即使属于 admins，没有委员岗位有效任职与权限时也被拒绝。
- benchmark fixture 已改为构建真实创世机构、岗位、任职和固定权限，不再用 admins 替代业务授权。正式权重已用当前 benchmark runtime WASM、50 steps / 20 repeats 重新生成：`joint-vote::cast_admin` 为 6 reads / 4 writes；决议发行提案为 368 reads / 280 writes / 1.977 s；协议升级提案为 367 reads / 281 writes / 12.483 s。
- 验收通过：`internal-vote` 96、`joint-vote` 12、`resolution-issuance` 19、`runtime-upgrade` 20、`votingengine` 4，runtime 全量 47 项单测；四个目标 crate `no_std`、runtime benchmark feature check、`git diff --check` 通过。目标 Clippy 在仅屏蔽已有跨依赖 lint 后以 `-D warnings` 通过，未越界修改旧 lint 所在文件。
- 当前源码强制重建 WASM/Node 后，`citizenchain-fresh` 隔离链真实启动，RPC `isSyncing=false`，runtime `specVersion=2`；block#0/genesis hash 为 `0x474e2b7870041f940143cd039c9a27cd0693c3518723b45c6c860cb99ce1114e`，state root 为 `0xdf1e13803e4c7a21b2301f7bcbc4cab5708bdaa960651d6cd0104eb44d2b7b64`。19945 验收进程已停止，临时 base path 已移入废纸篓；正式冻结 chainspec 未重烤。
- 最终残留扫描发现 `onchain-issuance` 两处 TODO 仍把 admins 写成业务授权条件；该额外 runtime 路径取得单独二次确认后已只清理注释，统一为业务模块校验 `RoleSubject + BusinessActionId + Propose` 并构造指定引擎 `VotePlan`；未改动该模块占位业务逻辑。
- 第 5A 当时不代表整体第 5 步完成；`internal-vote` 已在第 5B 完成，`legislation-vote`、`election-vote` 和其他相关业务入口继续由第 5C 迁移。

## 第 5B 步完成记录（2026-07-19）

- `internal-vote` 的机构提案统一保存 `VotePlan`，按完整 `RoleSubject` 生成岗位任职快照；第 6B 后按岗位票据投票，并由任一冻结岗位成员执行重试/取消。个人多签继续使用独立 `AdminSnapshot`。机构阈值仍读取机构投票规则，不新增岗位阈值。
- public/private 机构治理与关闭、决议销毁、GRANDPA 密钥更换、机构普通转账、NRC 安全基金转账、费用账户划转主账户、公民链基金会平台会员价格调整均新增独立 `proposer_role_code`，由业务模块前置校验 `CID + 岗位码 + BusinessActionId + Propose` 后构造固定内部投票计划。个人多签转账明确编码为无机构 CID、无岗位码，不混入机构授权模型。
- OnChina、Node、CitizenApp、CitizenWallet 和 QR action registry 已同步同一字段名与 SCALE 顺序；CitizenApp 的内部投票详情和协议升级联合投票详情均按机构/个人主体分别读取岗位有效选民快照或管理员快照，禁止用当前 admins 回算历史提案资格；CitizenWallet 严格拒绝 CID 与岗位码仅出现一项的机构转账载荷。
- 正式权重基于当前源码 fresh spec、50 steps / 20 repeats 生成：决议销毁提案 `239 ms / proof 584308 / 25 reads / 23 writes`，GRANDPA 密钥更换提案 `244 ms / proof 584308 / 25 reads / 23 writes`，机构转账提案 `289 ms / proof 584308 / 31 reads / 23 writes`；`internal-vote` 和 `votingengine` 权重同步重算。public/private 完整治理及关闭、Square Post 价格调整因现有 benchmark 夹具不能覆盖真实完整调用，采用 `400 ms / proof 700000 / 35 reads / 30 writes` 的独立保守上界，没有用不完整 benchmark 覆盖正式权重。
- 验收通过：`internal-vote` 96、`entity-primitives` 10、public/private manage 各 14、决议销毁 16、GRANDPA 密钥更换 17、多签 25、Square Post 23、`votingengine` 4、runtime 47 项；OnChina 134 项、Node 多签 4 项、QR registry 6 项、CitizenApp 目标 11 项、CitizenWallet 解码 97 项。目标 `no_std`、runtime-benchmarks、try-runtime、跨端 check/build/analyze 和仅屏蔽既存跨依赖 lint 类别的目标 `clippy -D warnings` 均通过。
- benchmark 脚本改为从当前源码二进制导出一次性 `citizenchain-fresh` spec，并显式使用 `spec-genesis`，退出后删除，禁止继续借用冻结 chainspec 或裸 WASM 创世。正式冻结 chainspec 本步骤未重烤。
- 最终残留清理后再次从当前源码构建 normal release WASM/Node，以一次性 fresh spec 和 `--tmp` 真实启动；RPC 返回 `CitizenChain`、`isSyncing=false`、`peers=0`、runtime `specVersion=2`。最终 block#0/genesis hash 为 `0xb9458b14ae014479cc483a0a6ec473fba2f4c4539df0855e5a118a1aaf8b06da`，state root 为 `0x9bee6fc682e3857ed7d5316496456ad20a5ec39b9e3aaed17748684bcc656939`；验收节点已正常停止，一次性 chainspec 已删除。
- 第 5B 当时不代表整体第 5 步完成；第 5C 已在后续确认后完成立法、选举及法律业务入口迁移。

## 第 5C 步完成记录（2026-07-19）

- 用户最终澄清：NRP/NSN/NED/PRP/PSN/CLEG/CEDU/CSLF 当前创世不预造法定成员岗位，创世仍只有强制且可空缺的 LR；成员岗位由各机构以后依法创建。因此本步没有修改 genesis seeder，也没有新增上述岗位或岗位阈值。
- `legislation-yuan` 把新法、修法、废法冻结为 `leg-yuan/0,1,2`，三个提案入口都显式携带 `proposer_role_code`，由业务模块按准确 `CID + 岗位码 + action + Propose` 前置校验并固定构造 `VotingEngineKind::Legislation` 的 VotePlan。代表机构岗位从 entity 实际 Vote 权限解析，管理员账户本身不产生资格。
- 代表表决、行政签署、国家/省级三人会签和 NJD 护宪终审全部在建案时按完整岗位主体冻结。相关 LR 只取得三个法律动作的 Vote 权限；NJD `CONSTITUTION_GUARD` 只取得修法 Vote，且建案时必须恰好冻结 7 个不重复账户。
- `election-vote` 的 Popular 允许空 voter subjects，由投票引擎消费 citizen-identity 的 `PopulationData` 生成 proposal 快照；Mutual 必须携带目标机构一个或多个 voter `RoleSubject`，候选人与选民都来自当前有效任职，调用方不能提交或删减选民集合。旧 `MutualVoters` 和 `MaxMutualVoters` 已删除。
- 人口职责重新收口：citizen-identity 只保存四级人口计数、资格 revision、判定日期和身份历史，是人口数据唯一真源；不再生成、编号或保存提案快照。votingengine 读取 `PopulationData` 后写入自己的 `ProposalPopulationSnapshots[proposal_id]`。旧 `PopulationSnapshots`、`NextSnapshotId`、`ProposalPopulationSnapshotIds` 和 `Proposal.citizen_eligible_total` 均已删除，不保留兼容。
- OnChina 法律三入口、前端岗位码输入、CitizenWallet 严格 SCALE 解码、CitizenApp/OnChina Proposal 解码和 QR 字段目录已同步；法律载荷中的岗位码固定紧随 `actor_cid_number`，旧尾字段和无岗位码载荷直接拒绝。
- `votingengine`、联合投票、立法投票、选举投票、决议发行和协议升级正式权重均由当前源码的 fresh benchmark runtime 以 50 steps / 20 repeats 重新生成；联合公投、立法公投和 Popular 选举的权重证明已包含 `ProposalPopulationSnapshots`，Mutual 选举只读取岗位有效选民快照。
- 验收通过：受影响 runtime crate 共 282 项单测、runtime 全量 47 项、NodeGuard 国家机构组成 9 项和真实 runtime 创世宪法守卫 1 项；runtime-benchmarks、try-runtime、目标 no_std、OnChina 134 项及生产 build、CitizenWallet 97 项、CitizenApp 11 项均通过，`git diff --check` 无格式错误。
- 当前源码强制重建 WASM/Node 后，以 `CITIZENCHAIN_HEADLESS=1` 和 `citizenchain-fresh --tmp` 启动全新隔离链；RPC 返回 `isSyncing=false`、`peers=0`、runtime `specVersion=2`。最终 block#0/genesis hash 为 `0x23bef606c4e991286dcb2a9b59e4a5e843b8f1e33b8fc400cd71c952e7d4701b`，state root 为 `0x719787bd66d3da8501079b02f93ccd123105279c76cb77bb8e6418a284a1cc7b`；验收节点已正常停止，正式冻结 chainspec 未重烤。

## 第 6 步完成记录（2026-07-19）

- 用户将本步收窄为机构权限路径收口，不新增机构登记 pallet、不实现机构新增/修改/删除、不扩大创世机构保护，也不改 OnChina 机构管理流程。
- 只读审计确认 public/private 机构治理与自定义账户关闭、机构转账均要求签名账户属于 `admins` 人员名册，并进一步校验准确 `RoleSubject + BusinessActionId + Propose`；机构投票只读取岗位有效任职快照。个人多签继续使用独立管理员快照。
- 删除旧机构管理员快照入口及对应测试；同步清理 runtime 生产接线和各 crate 测试 mock。个人管理员快照不再存在机构写入入口，只服务个人多签。
- 现状与修改后验证通过：`cargo check -p citizenchain`、`cargo check -p votingengine -p internal-vote --no-default-features`；runtime 全量 47 项通过；votingengine 4、internal-vote 94、public/private manage 各 14、multisig 25、grandpakey-change 17、resolution-destroy 16、legislation-yuan 32、election-vote 13、joint-vote 12、legislation-vote 34，共 275 项相关测试全部通过。
- 本步未触碰用户同时进行的 OnChina 权限分级改动，未修改 NodeGuard、机构登记、其他创世公权机构、冻结 chainspec、部署或 Git 远端。

## 第 7A 步只读盘点记录（2026-07-19）

- 已正确接入岗位权限和指定投票引擎的正式入口包括：协议升级、决议发行、决议销毁、GRANDPA 密钥更换、public/private 本机构治理与关闭、机构多签三类转账、公民链基金会平台调价和三类立法业务。其提案入口均校验准确 `RoleSubject + BusinessActionId + Propose`，机构投票均消费岗位有效任职快照；个人多签继续消费个人管理员快照。
- 盘点当时把 `citizen-identity` 和 `address-registry` 的直接业务写入误判为“缺少机构内部投票”。现已校正：注册局为公民或其他机构办理业务是对外行政操作，不是注册局机构自身的内部治理。公民办理依业务规则只需注册局有权岗位和公民本人签名；机构登记依规则由注册局有权岗位签名，不追加被服务机构或注册局内部投票。具体业务签名规则不在本权限重构中修改。
- `onchain-issuance` 与 `offchain-transaction` 整个 pallet 均被 `RuntimeCallFilter` 禁用：前者 10 个公开 call 仍是返回空成功的业务壳，后者虽有完整清算代码但当前不可从外部调用。它们的管理员判断不是当前可达业务权限路径，后续启用前必须分别完成业务规则和投票边界设计，不能在本任务中顺手开放。
- `election-campaign` 没有公开 call 且 `is_enabled() == false`，只是业务骨架；`election-vote` 已支持 Popular 人口数据快照和 Mutual 岗位快照，但不能把投票引擎能力误写成选举业务已经启用。
- `developer_direct_upgrade` 只在创世阶段 `DeveloperUpgradeEnabled=true` 时允许 NRC 委员岗位任职管理员走开发直升，运行期关闭；它复用 `runtime-upgrade/0` 的岗位业务权限，但不创建投票提案。正式协议升级入口仍按 NRC/PRC 委员岗位走联合投票。
- public/private 机构登记、注册局直接维护、机构账户 CRUD 和 `RegistryAuthority` 仍有管理员/地域校验，但用户已经明确排除出本任务；本步没有修改或扩展这些路径，也没有新增机构登记 pallet。
- 费用路由中的机构管理员判断只确定机构费用账户能否为当前签名调用付费，不代替业务模块的岗位授权；entity 岗位任职、治理结果落地和候选人校验中的管理员判断属于人员名册结构约束，继续保留。
- 当时发现一项无生产读取的人数接口仍挂在 `votingengine::Config`，但 runtime 没有任何调用；第 7 步已删除该接口、runtime 接线和全部测试 mock，并同步清理 `election-campaign` 等位置的旧互选 admins 快照口径。
- 本步只修改现有任务卡，没有产生 runtime、Node、OnChina、CitizenApp、CitizenWallet、NodeGuard、chainspec、部署或 Git 远端变更。

## 第 6A 步完成记录（2026-07-20）

- `admin-primitives` 新增公权 `PublicAdmin { admin_account, cid_number, family_name, given_name }`；`Admin { admin_account, family_name, given_name }` 只服务私权机构和个人多签。`public-admins` 允许当前公民 CID/姓/名为空，非空 CID 必须是 CTZN 且由 `citizen-identity` 的 CID↔钱包双索引确认一一绑定。
- public/private admins 不再接收、校验或写入机构阈值。public/private entity 新增 `InstitutionGovernanceThresholds[cid_number]`；机构阈值与管理员钱包数、岗位数分别独立。`internal-vote` 建案时通过 runtime provider 从对应 entity 读取并写入提案阈值快照；个人多签的 `ActivePersonalThresholds` 保持不变。
- 私权创世机构已重新定义为 SFGY 非营利法人“公民链技术发展基金会”，简称“公民链基金会”，英文全称 `CitizenChain Technology Development Foundation`，英文简称 `CitizenChain Technology Foundation`。CID 为 `GZ018-SFGYR-201206100-2026`，主/费用账户为 `0xe86aa3cd794651257dea9b7cad1abc4f0ce05940c1aecccd2ed8dd2fc9907023` / `0xaa23304c7b663ba25a9d3a2fb1efafdd650ecf2504a2caedc228fe81b46b4333`，平台会员收款继续指向新费用账户。
- 基金会的 `PrivateAdmins::AdminAccounts` 只保存一条程伟管理员人员记录；同一钱包 `0xd6d73cfd7d6b7c5692749b7c46fd3fe398f16f84283910dbf15f74472e1e3938` 分别任职 `LR`、`GENESIS_PRODUCT_MANAGER`、`GENESIS_PROGRAMMER` 三个固定岗位，每岗一席，机构阈值保持 2。NodeGuard 保持 89 个公权固定治理机构 + 1 个基金会，共 90 个保护对象。
- Node、OnChina、CitizenApp 与 CitizenWallet 已按目标 pallet 分流解码公权四字段/私权三字段 SCALE；OnChina 构造登记和机构治理载荷时同样严格分流。CitizenWallet 同步使用 runtime 当前治理/登记 call 布局，不再接受已删除的内层凭证尾字段。公权空身份字段不再被展示占位值伪造，非空公民 CID 必须为 CTZN 结构并最终由 runtime 对照 `citizen-identity` 校验。
- 第 6A 当时仍存在同一 CID 内按钱包合并的过渡限制；第 6B 已解除。基金会程伟同一钱包的三个固定岗位现在形成三张独立票据，在机构阈值 2 不变的前提下可依法分别投票。
- 代码回归已通过：`entity-primitives` 11、`genesis-pallet` 11、`internal-vote` 94、`public-admins` 10、`private-admins` 6、public/private manage 各 14、OnChina 133、runtime 46、Node 285、`square-post` 23、CitizenWallet 载荷解码 98 + 离线签名服务 9、QR registry 6 项；相关 `no_std`、Node/OnChina 生产编译和 CitizenWallet 全量静态分析通过。
- 当前源码强制重建 WASM/Node 后以隔离 `citizenchain-fresh --tmp` 真实启动，NodeGuard 自检通过；RPC 返回 block#0 `0x9ad703ec20ed91f693e8077075cc27ffbe0d4f1b9b0e0ee32fb917e52009f6fd`、state root `0xdc532c2cfaa75db4ce38530ee2986c138360da8a8ffa5bbeab36b37b66a9c8b1`、`isSyncing=false`、runtime `specVersion=2`。验收节点已停止并由 `--tmp` 清理；未烘焙正式 chainspec、未部署、未推送。

## 第 6B 步完成记录（2026-07-20）

- 共享 `votingengine` 新增 `InstitutionVoteTicket { role_subject, voter_account }`，岗位有效选民继续按 `VoterSnapshot[(proposal_id, RoleSubject)]` 冻结，机构席位数改由 `InstitutionTicketCountSnapshot[(proposal_id, cid_number)]` 汇总。旧的同 CID 按钱包合并快照和人数 storage 已直接删除，不保留 migration 或双轨兼容。
- `internal-vote`、`joint-vote`、Mutual `election-vote` 和代表 `legislation-vote` 分别改用完整机构岗位票据账本；个人多签、Popular 选举、联合公投和立法公投仍使用其原本的个人账户票据。机构阈值完全未修改，也没有增加岗位阈值。
- 同一钱包兼任多个机构或同一机构多个岗位时，每个 `CID + role_code + wallet` 只能投一票，但不同岗位票据互不覆盖。新增内部投票和联合投票测试锁定“同一钱包跨岗位分别记票、同一岗位不得重复投票”。
- Node、OnChina、CitizenApp、CitizenWallet 与 QR registry 已同步完整岗位码字段、SCALE 顺序、storage key 和签名展示。CitizenApp 删除已无写入方的旧账户级待确认票存储；投票服务必须在入块后按本次完整票据回读 runtime storage，页面不得用钱包级等待状态阻塞该钱包的其他岗位票。
- 正式权重基于当前 fresh benchmark runtime、50 steps / 20 repeats 重算；脚本中 14 个已注册 benchmark pallet 全部成功。`citizen-identity` 当前未注册 FRAME benchmark，继续使用其现有手工保守权重并从自动列表移除，避免脚本伪报可生成权重。
- 验收通过：五个投票引擎 crate 共 158 项、runtime 46 项、Node 285 项、OnChina 134 项、CitizenApp 747 项（既有条件跳过 5 项）、CitizenWallet 181 项、QR registry 6 项；CitizenApp/CitizenWallet analyze、Node/OnChina check、Node/OnChina 前端生产构建、runtime-benchmarks、目标 no_std、try-runtime 和 `git diff --check` 均通过。
- 当前最终源码以 `WASM_BUILD_FROM_SOURCE=1` 重建 normal release Node，并用 `CITIZENCHAIN_HEADLESS=1 citizenchain-fresh --tmp` 启动全新隔离链；NodeGuard 自检通过，RPC 返回 `peers=0`、`isSyncing=false`、runtime `specVersion=2`。block#0/genesis hash 为 `0x21319c02264a559cec31d03cfbb3d0551db2ee354bb2b950a38ecbe0c03dd1a4`，state root 为 `0x69c8bd5e3b9677aa4a8775670e528c4f1d10630861dd32375aeb8411f0aa57e9`；验收节点已正常停止，未重烤正式 chainspec、未部署、未提交或推送 Git。

## 第 7 步完成记录（2026-07-21）

- 删除 votingengine Config 中无生产读取的旧管理员人数 provider、runtime 接线及全部测试 runtime mock。机构阈值仍只由 public/private entity 提供，岗位席位数仍只来自提案冻结的 `CID + role_code + wallet` 票据；未修改机构阈值、岗位阈值、storage、dispatch、权重或业务规则。
- 逐项复核协议升级、GRANDPA 密钥更换、决议销毁、决议发行、public/private 机构治理与关闭、机构转账、公民链基金会平台调价和三类立法入口；正式可达入口均先校验准确岗位业务权限，再调用代码静态指定的投票引擎。未发现需要修改的生产业务入口。
- `onchain-issuance`、`offchain` 继续由 `RuntimeCallFilter` 整体禁用，并新增代表 call 的 runtime 过滤回归；`election-campaign` 继续无公开 call 且 `is_enabled() == false`。这些业务骨架在另行确认完整业务、权限和投票设计前不得开放。
- 清理互选、立法、内部投票、机构约束和跨模块文档中的旧 admins 机构投票口径；`AdminSnapshot` 只用于个人多签，机构投票只使用岗位有效任职快照。同步修正此前未被外围测试编译到的旧三参数测试调用，为每个测试提案显式传入机构岗位或个人多签授权主体；生产逻辑未因此变化。
- 回归通过：17 个受影响 crate 共 356 项、runtime 46 项、NodeGuard 80 项，全部零失败；18 个受影响 crate 的 `no_std`、runtime 的 `runtime-benchmarks` 与 `try-runtime` 特性编译、目标 `cargo fmt --check` 和 `git diff --check` 均通过。删除的接口不参与 dispatch 或存储读写，因此无需重算权重。
- 当前源码已用 `WASM_BUILD_FROM_SOURCE=1` 重建 release Node，并以 `CITIZENCHAIN_HEADLESS=1 citizenchain-fresh --tmp --rpc-port 9945` 启动全新隔离链。RPC 返回 `peers=0`、`isSyncing=false`、runtime `specVersion=2`；block#0/genesis hash 为 `0x0fa3a524db67758bb25c921dede02f823a4f491e118569a4e531d71eb5b34bc5`，state root 为 `0x22724cca8ca3b966e01268a5f60d9f096dd41b273a893bc369d1804abaf37ccc`。验收节点已正常停止并由 `--tmp` 清理；未重烤正式 chainspec、未部署、未提交或推送 Git。

## 第 8 步完成记录（2026-07-21）

- 私权创世机构中文全称最终统一为“公民链技术发展基金会”，英文全称统一为 `CitizenChain Technology Development Foundation`；简称“公民链基金会”、英文简称 `CitizenChain Technology Foundation`、CID、机构码、法人性质、主账户、费用账户、一名程伟管理员、三项固定岗位任职、固定权限和机构阈值 2 全部保持不变。runtime 测试以字面量同时锁定四个中英文全称/简称。
- 旧中文全称、旧英文全称、此前股份公司名称和旧公司类样例已从 runtime、Node、OnChina、公开白皮书、模块文档、有效 ADR、开放任务记录、测试向量、注释和生成文档中清零；不保留 migration、别名、兼容分支或表述性旧注释。
- 公开白皮书和治理页面已彻底改为最终权限模型：admins 只表示管理员人员身份与登录资格；机构业务权限必须同时匹配机构 CID、岗位码、有效任职和签名钱包；机构投票使用岗位票据与机构阈值，个人多签使用个人管理员快照与个人阈值；普选人口数据来自 citizen-identity，提案快照由投票引擎生成；互选只冻结 `VotePlan` 指定岗位任职。Node 本地文档已从该白皮书重新生成。
- 全端回归通过：primitives 75 项、genesis 11 项、runtime 46 项、五个投票引擎 crate 158 项、QR registry 6 项、Node 285 项、OnChina 134 项、CitizenApp 747 项（5 项条件跳过）、CitizenWallet 181 项，全部零失败。OnChina 清除了无引用的测试人口常量，复验不再产生该 warning。
- 编译与静态检查通过：目标 runtime `no_std`、`runtime-benchmarks`、`try-runtime`，OnChina 普通二进制，OnChina/Node 前端生产构建，CitizenWeb 生产构建和 ESLint，CitizenApp/CitizenWallet `flutter analyze`，目标 `cargo fmt --check` 与 `git diff --check`。本步没有 dispatch、存储读写或权重变化，无需重算权重。
- 当前源码以 `WASM_BUILD_FROM_SOURCE=1` 重建 release Node，并以 `citizenchain-fresh --tmp --rpc-port 19944` 启动全新隔离链；RPC 返回 `peers=0`、`isSyncing=false`、runtime `specVersion=2`。block#0/genesis hash 为 `0xb22750e32291c0741119bb640de71e17127dc83095d7c83b09704c7f84d27b73`，state root 为 `0x9a5599dd9716a1e891bb6270cf6a33835aa82085a9cfc562f993a653e7745f27`。
- 使用当前源码显式构建的 OnChina 普通二进制、仓库外临时 PostgreSQL 和上述 fresh 链完成真实服务验收：链投影精确为 49,593 个公权机构和 99,231 个协议账户；启动抽样精确覆盖 32 个派生公权机构、1 个公权常量机构和公民链技术发展基金会，共 `sampled=34`；真实首页与 `/api/v1/health` 均返回 HTTP 200。首次误用旧普通二进制得到的 `sampled=33` 未计入验收，已重建后二次通过。
- 两次验收节点、OnChina、内嵌 PostgreSQL 均已正常停止；`/tmp/gmb-onchina-step8.4gbZ4d` 与 `/tmp/gmb-onchina-step8-final.ts9KC0` 临时数据已删除。未修改正式 chainspec，未部署，未提交或推送 Git。
