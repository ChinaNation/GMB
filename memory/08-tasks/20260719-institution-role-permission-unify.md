# 任务卡：机构岗位权限模型统一（岗位=权限载体·治理岗/自治岗·法人岗必存·创世技术公司）

## 任务需求

把机构权限模型统一为「岗位是权限唯一载体、任职即授权、全机构统一」，落地以下已确认口径：

- 权限两级：**机构权限（顶层，由 CID 内机构码派生封顶）+ 岗位权限（键 `cid+role_code`，必须 ⊆ 机构权限）**。
- 岗位两类：**治理岗（系统内置固定码/名，创世机构定死）/ 自治岗（机构后期自由增删、只可改岗位名）**。
- **所有 `role_code` 一律不可改，只增删；唯一可改字段 = `role_name`（仅自治岗）。**
- **法定代表人岗（`LR`）全机构创世必存、不可删、可零任职**；给 `FRG` 补齐 `LR`。
- 岗位任期由对应投票引擎（普选/互选/提名/注册局/创世）写入，**鉴权新增 `now ∈ [term_start, term_end]` 时点窗口**。
- 阈值/投票主体沿用既有投票引擎（最低过半、`admins_len ≥ 2`）；**投票引擎与业务模块不重写，只在机构层接入**。
- 新增**创世技术公司**（私权 `SFGQ`）三个治理岗：`LR` + `GENESIS_PRODUCT_MANAGER`（创世产品经理，1 席）+ `GENESIS_PROGRAMMER`（创世程序员，1 席）。
  - CID：`GZ018-SFGQ1-201206100-2026`
  - 主账户：`0x7a20b8b7b1147abfdb24615222e3c9d77f1ff9a85d2a509fcf51dc89a64d1712`
  - 费用账户：`0x4bc5b8dd3770b1230c79fb8e048f27ae4f4ccf6d6890de0399123a617ccf305f`

链开发期零用户：彻底改、零残留、重新创世，不做迁移/兼容。

## 预计修改目录

- `citizenchain/runtime/entity/entity-primitives/`
  - 用途：`InstitutionRole` 加 `permissions`(位标志) + `role_class`(治理岗/自治岗)，删 `term_required`；新增 `CapabilitySet` 能力位与「岗位权限 ⊆ 机构权限」校验 trait；补 SCALE 字段序契约测试。
  - 边界：只放共用类型与 trait，不持有 storage、不写业务规则。
  - 类型：runtime 共用原语，breaking，需用户二次确认。

- `citizenchain/runtime/entity/public-manage/`、`private-manage/`
  - 用途：岗位 CRUD（增/删/改名）；`LR` 必存不可删；任期时点窗口接入 `is_active_assignment`/`active_accounts_for_role`；治理结果校验更新（保留固定席位与 member_composition 不变量）。
  - 边界：只改机构岗位/任职/权限校验，不扩展业务规则、不改投票引擎职责。
  - 类型：runtime 代码与测试。

- `citizenchain/runtime/admins/`
  - 用途：把「机构 admins 集合 = 治理岗投票主体有效任职者」口径对齐，保证投票引擎读到的分母只含治理岗；自治岗工作人员不入 admins。
  - 边界：只改口径与派生一致性，不改投票引擎接口。
  - 类型：runtime 代码与测试。

- `citizenchain/runtime/primitives/`
  - 用途：`governance_skeleton` 增两个新岗位码/名（`GENESIS_PRODUCT_MANAGER`/`GENESIS_PROGRAMMER`）与技术公司固定规格；`institution_constraints` 增「机构权限（顶层）派生表」与全机构 `LR` 必存约束；`count_const` 增技术公司席位常量。
  - 边界：结构性协议单源；业务能力位常量放 entity 侧不进 primitives 核心（遵守 no-business-types-in-primitives）。
  - 类型：runtime 协议常量，breaking。

- `citizenchain/runtime/genesis/`
  - 用途：技术公司常量入库（CID/主账户/费用账户）与三岗创世播种；`FRG` 补 `LR` 空岗；`fixed_roles` 钱包→岗位映射更新。
  - 边界：只写创世，不写运行期逻辑；重新创世无迁移。
  - 类型：创世播种代码与测试。

- `citizenchain/runtime/src/configs.rs`
  - 用途：新增机构 capability 查询入口，把既有机构鉴权 gate 切到「持岗位权限的有效在任者」口径；省专员/护宪大法官两条主权路径统一为「∈ admins ∩ 有效岗位任职」双校验。
  - 边界：只接线机构层授权解析，不新增/改写业务模块与投票引擎。
  - 类型：runtime 装配代码与测试。

- `citizenchain/node/src/core/node_guard/governance_skeleton.rs`
  - 用途：节点守卫同步新骨架（技术公司/两新岗/`LR` 必存/权限位字段序）。
  - 边界：只测守卫行为与 RAW 解码，不改业务。
  - 类型：节点守卫代码与测试。

- `citizenchain/onchina/`
  - 用途：机构/岗位/权限 SCALE 读写与展示同步（含新岗位码/名、权限位展示、技术公司）。
  - 类型：CID 控制台前后端，需与链端字段序逐字节对齐。

- `citizenchain/wallet`（CitizenWallet）、CitizenApp
  - 用途：岗位/任职/权限 SCALE decode 同步（两色识别、护照/机构详情展示）。
  - 类型：移动端解码同步。

- `memory/04-decisions/ADR-039-institution-role-permission-model.md`
  - 用途：新增机构岗位权限模型 ADR（两级权限、治理岗/自治岗、LR 必存、任期窗口、投票主体=治理岗）。
  - 类型：架构决策文档。

- `memory/05-modules/`、`memory/08-tasks/`
  - 用途：模块技术文档与本任务卡状态回写。
  - 类型：文档。

## 分步

- 第 1 步（类型/权限模型）：entity-primitives 类型改造 + capability 位 + 岗位权限⊆机构权限校验 + 契约测试。
- 第 2 步（创世）：技术公司常量入库 + 两新岗位码/名 + `FRG` 补 `LR` + 全机构 `LR` 必存 + genesis 播种。
- 第 3 步（岗位 CRUD + 任期窗口）：public/private-manage 增删改名 + 任期时点鉴权 + admins=治理岗投票主体口径。
- 第 4 步（鉴权接入）：configs.rs capability 查询接入既有 gate，主权双校验统一。
- 第 5 步（节点守卫 + 四端 SCALE 同步）：node_guard / onchina / CitizenWallet / CitizenApp。
- 第 6 步（测试与验收 + ADR-039 + 文档回写）。

## 验收要求

- `cargo test` 覆盖 entity/admins/primitives/genesis/configs 受影响 crate（含字段序契约测试）。
- `cargo check --no-default-features`（wasm 侧）通过。
- `cargo fmt --check` 通过；`cargo clippy -- -D warnings` 通过。
- 重新创世启动开发链，技术公司三岗、`FRG` `LR`、各创世机构岗位任职按骨架写入且节点守卫逐块通过。
- onchina / CitizenWallet / CitizenApp 对新 SCALE 结构 decode 无红、无 decodeFailed。
- 主权路径（省专员登记 / 护宪修宪终审）双校验一致，越权/漂移 fail-closed。

## 进度

- [ ] 第 1 步 类型/权限模型
- [ ] 第 2 步 创世常量与播种
- [ ] 第 3 步 岗位 CRUD 与任期窗口
- [ ] 第 4 步 鉴权接入
- [ ] 第 5 步 节点守卫与四端同步
- [ ] 第 6 步 测试验收与 ADR/文档
