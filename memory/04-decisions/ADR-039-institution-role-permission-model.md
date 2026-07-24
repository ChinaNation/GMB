# ADR-039：机构 CID 与岗位码权限模型

状态：Accepted（2026-07-20；公私权管理员分型、机构阈值解耦、公民链基金会重新创世、岗位席位记票和 ADR-040 runtime 账户字段统一均已实施）。

## 背景

一个机构可以有大量工作人员、多个岗位，并且不同岗位可发起、投票或执行不同业务。管理员只是人员名册；若以管理员账户直接授权或把全体管理员自动作为投票人，会把机构人员、岗位职责、业务权限和投票规则混成同一层，无法表达现实机构结构。

## 决策

### 1. 授权主体

机构业务授权主体唯一表示为：

```text
RoleSubject = (cid_number, role_code)
```

- CID 决定机构能够拥有的顶层业务能力。
- `RoleSubject` 决定该机构内某岗位能够执行的具体业务动作。
- 管理员账户没有固有业务权限；有效权限来自“账户属于 admins + 账户对 RoleSubject 有有效任职 + RoleSubject 拥有目标业务动作权限”。
- 同一账户可在多个机构和多个岗位任职，各 `RoleSubject` 完全隔离。

### 2. 岗位权限

- 稳定业务动作标识统一使用 `BusinessActionId`，岗位权限记录统一使用 `RoleBusinessPermission`。
- 权限操作至少区分 `Propose` 与 `Vote`；不得用“管理员能做什么”或泛化字符串权限替代。
- 岗位权限必须属于该 CID 的顶层能力范围。
- 岗位权限与岗位码绑定且不可修改；改变权限必须删除旧动态岗位并创建新岗位码。

### 3. 业务模块与投票引擎

- 每个状态变更由对应业务模块前置校验发起人的 `RoleSubject` 与 `Propose` 权限。
- 每个业务动作在业务模块代码中静态指定唯一投票引擎；管理员、岗位、客户端和交易参数均不得选择或覆盖。
- 业务模块构造并绑定 `VotePlan`：业务动作、业务对象摘要、提案岗位主体、参与投票岗位主体及引擎所需规则。
- 投票引擎只负责合格任职账户快照、投票资格、阈值、计票、通过/否决、终态和维护。
- 投票引擎不读取岗位权限来决定业务是否合法，不解释业务正文，也不执行转账、发行、升级、任免等具体业务。
- 投票通过后，由已绑定业务模块执行确定性回调；回调不得再次建立投票流程。
- 机构和个人多签对自身的内部治理状态变更必须经过指定投票引擎。只读、创世写入、投票引擎内部维护和已通过提案回调除外。注册局为公民或其他机构办理登记属对外行政业务，依其业务规则校验岗位与必要签名，不能误改为注册局机构内部投票。

### 4. 联合投票

联合业务可以绑定多个 `RoleSubject`。协议升级与决议发行采用相同参与结构：NRC/43 个 PRC 的 `COMMITTEE_MEMBER` 可发起和投票，43 个 PRB 的正式 `DIRECTOR / 董事` 只投票。参与资格按 VotePlan 的完整岗位主体解析，不把参与机构的全部 admins 自动纳入。

### 5. 岗位生命周期

- 所有机构必须永久存在唯一 `LR / 法定代表人` 岗位；岗位允许空缺，但岗位码和岗位名不可修改或删除。
- `LR` 岗位任职人数只能为 0 或 1；法定代表人姓名、个人 CID、账户三字段必须与 `LR` 任职在同一治理结果中一起设置或一起清空。
- 所有创世固定岗位的岗位码、岗位名和岗位权限永久固定。非营利法人“公民链技术发展基金会”固定包含 `LR`、`GENESIS_PRODUCT_MANAGER`、`GENESIS_PROGRAMMER`，并允许同一账户分别任职三岗。
- 创世机构可以依法增加、改名和删除普通动态岗位；NodeGuard 只保护固定岗位，不禁止额外动态岗位。
- 动态岗位码由 runtime 生成，机构内唯一、不可修改，删除后永不复用；调用方不得指定岗位码。
- 动态岗位码格式：`R_<32 位大写十六进制>`。
- 生成材料：`blake2_256(SCALE(MODULE_TAG, cid_number, institution_role_nonce, proposal_id))`，取前 16 字节并转为大写十六进制。
- `InstitutionRoleNonce[cid_number]` 单调递增；`UsedRoleCodes[(cid_number, role_code)] = true` 永久保留，删除岗位不删除占用记录。
- `role_name` 在机构内唯一；同名多人必须表达为同一岗位码下的多个任职席位，不能复制成多个同名岗位。动态岗位名可以依法修改；岗位码、岗位权限及固定属性不可修改。
- 同一管理员可以在同一机构担任多个不同岗位；任职去重边界是“同一岗位内同一账户不得重复占席”，不是“一个账户在机构内只能有一个岗位”。
- 岗位只定义席位与权限，不保存岗位阈值；投票阈值属于机构/具体投票计划。例如 NRC 是一个 `COMMITTEE_MEMBER` 岗位、19 个任职席位、机构阈值 13。
- 机构阈值由 public/private entity 按 CID 独立保存，不得从 `admins` 账户数推导。投票引擎只在建案时消费该阈值并冻结提案快照。
- 机构投票票据唯一键是 `InstitutionVoteTicket { role_subject, voter_account_id }`。同一账户兼任多个有效投票岗位时，每个岗位各有一张票；同一岗位和账户组合只能投一次。个人多签仍使用独立的账户票据。
- `InstitutionTicketCountSnapshot[(proposal_id, cid_number)]` 冻结该机构在本提案中的岗位席位票据总数，只用于可达票数和阈值判定，不建立岗位阈值，也不按账户去重。

### 6. 机构创建与个人多签

- 普通机构创建必须原子建立 admins、强制 LR、至少一个初始治理岗位及其权限、初始任职和初始投票规则。
- 不存在临时管理员权限、超级管理员或先创建再补岗位的授权窗口。
- 个人多签使用 `AuthorizationSubject::PersonalMultisig`，继续按个人多签管理员集合治理，不复用机构 `RoleSubject`。

## 唯一真源与边界

- admins 人员名册：`runtime/admins`。公权、私权机构统一使用 `Admin { account_id, cid_number, family_name, given_name }`，非空公民 CID 只能引用 `citizen-identity` 的 `AccountIdByCid` / `CidByAccountId` 双向绑定。个人多签复用同一 SCALE 结构，但不属于机构管理员并按个人多签规则处理字段完整性。
- 岗位、岗位权限、岗位码 nonce/占用和任职：`runtime/entity`。
- CID 顶层能力、创世固定岗位与权限：runtime 共享常量与创世规范。
- 业务动作权限要求、指定投票引擎、VotePlan 和通过后执行：对应业务模块。
- 机构治理阈值：`runtime/entity/public-manage` / `private-manage`；提案阈值快照、资格快照、票据、计票和终态：`runtime/votingengine`。
- 永久固定岗位保护：NodeGuard；不得扩大为一般机构业务授权真源。

## 后果

- 现有“admins 即授权”和“机构投票快照全体 admins”的实现必须删除，不能保留兼容分支。
- 所有受影响 SCALE、storage、QR 和客户端解码必须在同一步骤跨端同步。
- 普通机构创建载荷会发生 breaking change，开发链重新创世，不做历史迁移。
- 每个业务模块都必须显式登记业务动作、可发起/投票岗位主体和唯一投票引擎；未登记则 fail-closed。

## 第 4A 固定权限盘点

完整动作目录、逐岗位 `Propose/Vote` 固定矩阵、实现记录和验收记录在任务卡 `memory/08-tasks/20260719-institution-role-permission-unify.md` 的“第 4A 步盘点结果与固定矩阵”和“第 4B 步完成记录”。

固定矩阵已经确认并实施：协议升级与决议发行均由 NRC/PRC `COMMITTEE_MEMBER` 发起和投票，两个业务中的 PRB `DIRECTOR` 均只投票；FRG 按准确省专员岗位隔离；公民链基金会平台调价由 `GENESIS_PRODUCT_MANAGER` 发起、三个固定岗位投票。没有明确固定职责的转账、普通资产发行等能力不授予固定岗位，后续由动态岗位承接。

## 实施与验收

实施顺序及完整验收以 `memory/08-tasks/20260719-institution-role-permission-unify.md` 为准。2026-07-19 已完成共享授权类型、跨端 SCALE 契约、public/private entity 岗位权限生命周期、创世固定权限、准确 CID 顶层能力和 NodeGuard 固定权限保护。旧机构直接创建 call 5 已永久关闭。

联合、内部、立法和互选 Track 均按每个 `RoleSubject` 建立 `VoterSnapshot`，机构票据按 `RoleSubject + voter_account_id` 分别保存，已删除旧的按 CID 合并账户快照。同一账户仍只需一把私钥，但可依法分别行使其每个有效岗位席位；机构阈值未改变。普选和立法公投继续使用 citizen-identity 人口数据生成的提案人口快照，个人多签继续按管理员账户票据记票。

不得恢复 admins 直接授权、机构全体 admins 快照、按账户合并岗位票权或旧账户票 storage。跨端调用必须显式携带岗位码，并以提案冻结的 VotePlan 和 VoterSnapshot 筛选可投岗位。
