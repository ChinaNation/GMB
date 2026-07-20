# entity-primitives 技术说明

模块：`entity-primitives`

职责：实体生命周期共用类型与 trait。该 crate 不含 storage、不含 extrinsic、不保存 CID 登记状态。

ADR-039 已冻结机构岗位权限目标模型。任务卡第 2 步已落地共享授权类型，第 3 步已落地岗位权限 storage、任职有效期和动态岗位生命周期，第 4 步已落地稳定业务动作目录与创世固定权限；投票快照和具体业务接入仍按后续步骤实施。

## 边界

- 定义 `EntityKind`，区分公权机构、私权机构、个人多签。
- 当前定义 `InstitutionMultisigQuery`，供交易、清算、扫码验签等模块统一查询公权/私权机构账户状态和 admins 人员数据；机构内部/联合投票已改由 `VotePlan` 的 `RoleSubject` 有效任职解析，不得把该查询返回的 CID 全体 admins 用作投票主体。
- 定义 `InstitutionCidQuery`，供 `public-manage` 和 `private-manage` 互查 CID 是否已登记，防止同一 CID 在多个生命周期模块重复写入。
- 定义 `CidInstitutionVerifier`，统一 CID 机构登记、注销凭证验签接口。
- 定义 `InstitutionGovernanceAction` 与 `InstitutionGovernanceProposal`，统一表达本机构内部治理中的 `admins` 完整替换、`InstitutionRoleMutation::{Create,Rename,Delete}`、任职变更和法定代表人三字段整体设置或清空；创建岗位不接收 `role_code`，必须原子携带不可变权限和初始任职。
- 定义 `InstitutionGovernanceResult`，作为创世、注册局、投票/选举引擎和本机构内部治理写入 entity 岗位、任职、法定代表人的唯一结果协议。
- 已定义 `RoleSubject { cid_number, role_code }`，作为机构业务授权和机构岗位投票资格的唯一主体。
- 已定义 `BusinessActionId { module_tag, action_code: u32 }`、`RoleBusinessPermission { role_subject, business_action_id, operation }` 和 `AuthorizationSubject` 强类型；个人多签使用 discriminant `1` 的独立 `PersonalMultisig(AccountId)` 变体。
- `RolePermissionOperation` 的 SCALE discriminant 固定为 `Propose = 0`、`Vote = 1`；`AuthorizationSubject` 固定为 `Institution = 0`、`PersonalMultisig = 1`。
- 跨端 SCALE 金标唯一文件为 `memory/06-quality/fixtures/institution_role_permission_v1.json`；Node 使用本 crate 共享类型逐字节解码，OnChina、CitizenApp、CitizenWallet 对同一金标严格解码并拒绝尾随字段。
- 定义 `InstitutionCapabilityPolicy` 与 `InstitutionRoleAuthorizationQuery`，供业务模块校验“CID 顶层能力 + 岗位权限 + 有效任职”；本 crate 只定义 trait，不保存权限 storage、不选择投票引擎。
- `business_action.rs` 是稳定 `module_tag/action_code` 与受保护创世岗位固定权限唯一目录；协议升级与决议发行采用同一联合权限矩阵：NRC/PRC 委员岗位拥有 `Propose + Vote`，PRB 正式 `DIRECTOR / 董事` 只有 `Vote`。该目录不选择投票引擎，也不表示尚未迁移的业务已经按岗位执法。
- 机构内 `role_code` 与 `role_name` 分别唯一；同名多人属于一个岗位的多个任职席位。一个管理员可以担任多个不同岗位，但同一岗位内不得重复占席。
- 每个机构唯一的 `LR / 法定代表人` 岗位永久存在，任职只能为 0 或 1 人；法定代表人三字段与 LR 任职必须原子一致。岗位不保存阈值，阈值属于机构或业务绑定的投票计划。
- 复用 `primitives::multisig` 的账户校验、保留地址、保护地址 trait。

## 禁止事项

- 不允许在本 crate 增加 storage。
- 不允许把公权、私权、个人多签生命周期状态写到本 crate。
- 不允许恢复单独的 entity-registry pallet。
- 不允许把 `admins`、裸账户、裸 CID 或裸岗位码定义为机构业务授权主体。
- 不允许在本 crate 定义具体业务模块使用哪个投票引擎；该决定归各业务模块。
