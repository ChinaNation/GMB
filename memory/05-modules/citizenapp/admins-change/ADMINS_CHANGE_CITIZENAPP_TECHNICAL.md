# citizenapp 管理员更换模块技术文档

最新更新：2026-07-13。

## 模块定位

`citizenapp` 是 Flutter 客户端，不区分传统前端 / 后端，不新建 `backend/`。个人多签管理员更换作为一级业务模块放在：

```text
/Users/rhett/GMB/citizenapp/lib/citizen/proposal/admins-change/
```

边界：

- 不新建 `citizenapp/backend/`。
- 需要从统一提案入口跳转时，只在入口处引用 `lib/citizen/proposal/admins-change/pages/admin_set_change_page.dart`。

## 目录结构

```text
citizenapp/lib/citizen/proposal/admins-change/
├── admin_set_change_controller.dart
├── admin_set_change_qr_adapter.dart
├── models/
│   ├── admin_account.dart
│   ├── admin_set_change.dart
│   └── admin_set_change_result.dart
├── codec/
│   ├── account_id_codec.dart
│   ├── admin_account_codec.dart
│   └── admin_set_change_call_codec.dart
├── services/
│   ├── admin_activation_service.dart
│   ├── admin_account_service.dart
│   ├── institution_admin_service.dart
│   ├── admin_set_validation.dart
│   └── admin_set_change_service.dart
├── pages/
│   ├── admin_account_detail_page.dart
│   ├── admin_set_change_page.dart
│   └── admin_set_change_confirm_page.dart
└── widgets/
    ├── admin_account_card.dart
    ├── admin_set_editor.dart
    ├── admin_set_diff_card.dart
    └── admin_set_change_action_bar.dart
```

测试目录：

```text
citizenapp/test/governance/admins-change/
├── admins_change_codec_test.dart
└── institution_admin_service_test.dart
```

## 业务流程

1. 只有 `PMUL` 个人多签显示“换管理员”入口并进入 `AdminsChangePage`。
2. 调用方构造 `personalAccount` 类型的 `AdminAccountIdentity`，由 `AdminAccountService` 读取 `PersonalAdmins::AdminAccounts`。
3. 机构账户页面只读取 `PublicAdmins / PrivateAdmins::AdminAccounts` 钱包集合，并通过 `InstitutionAdminService` 联合读取 entity 岗位任职，不进入管理员集合编辑流程。
4. 用户选择管理员钱包、编辑完整管理员集合。
5. `AdminSetValidation` 做端上前置校验，同时校验目标阈值。
6. `PersonalAdminsChangeCallCodec` 固定构造 `PersonalAdmins.propose_admin_set_change` call data。
7. `AdminSetChangeService` 通过 `SignedExtrinsicBuilder` 走热钱包或公民钱包签名并提交。

## 主体身份与查询门面

`/Users/rhett/GMB/citizenapp/lib/citizen/proposal/admins-change/models/admin_account.dart` 定义 `AdminAccountIdentity`，调用方必须显式传入三类主体之一：

- `governanceInstitution` / `fixedGovernanceInstitution`：固定治理公权主体，固定治理档机构码（NRC/PRC/PRB/FRG/NJD），`kind=0`。
- `institutionAccount`：公权机构账户主体，`kind=0`；私权机构账户主体，`kind=1`；非法人机构按所属法人归属选择 `kind=0` 或 `kind=1`。
- `personalAccount`：个人多签主体，个人多签码（PMUL），`kind=2`。

`/Users/rhett/GMB/citizenapp/lib/citizen/proposal/admins-change/services/institution_admin_service.dart` 是查询门面，但不接收模糊字符串身份；所有 `fetchAdmins / fetchThreshold / isAdmin / clearCache` 调用都必须传 `AdminAccountIdentity`。按单一字符串混用个人、机构、治理主体的入口不存在。

非法人机构码（`SFGT/SFGP/UNIN`）不是私权同义词。调用方必须从 CID 注册归属或链上 `AdminAccount.kind` 显式传入 `kind=0`（公权）或 `kind=1`（私权）；不得只凭机构码自动归入 `PrivateAdmins`。`OrganizationManage.propose_create_institution` 当前只直接创建公权法人或私权法人机构账户，裸非法人创建会被端上和链端拒绝。

## 管理员更换载荷与阈值

个人多签管理员更换载荷固定为：

```text
[pallet][call][institution_code:[u8;4]][account_id:32][admins:Compact<Vec<AccountId32>>][new_threshold:u32_le]
```

规则：

- 只允许 PMUL 个人多签走 `PersonalAdmins(29).propose_admin_set_change(0)`；codec 和 service 对其它 `AdminAccountKind` 关闭失败。
- 公权、私权、非法人及固定治理机构的管理员人员集合独立于岗位任职；CitizenApp 当前不构造对应管理员集合变更调用，第2步再接入专用维护协议。
- `new_threshold` 是载荷必填字段，端上和链端按同一字节结构构造、解析和签名。
- 个人多签显示动态阈值输入框，端上前置校验：`threshold * 2 > admins_len && threshold <= admins_len`。
- 个人多签动态阈值由 `InternalVote.ActivePersonalThresholds[personal_account]` 保存；机构阈值按 CID 使用 `ActiveInstitutionThresholds[cid_number]`，不属于本页面。
- QR_V1 只携带 `b.a + b.d`；扫码端从 `b.d` 解码出的展示字段必须与冷钱包 decoder 逐项一致：`institution_code / subject / admins / new_threshold`。

## 管理员激活

管理员激活服务位于 `/Users/rhett/GMB/citizenapp/lib/citizen/proposal/admins-change/services/admin_activation_service.dart`。机构管理员列表和提案上下文只引用该服务，不再从 `lib/institution/` 承载激活逻辑。

激活记录使用 `activated_institution_admins_v1`，只保存 `cid_number / institution_code / kind / pubkey_hex / activated_at_ms`；查询、去重和清理统一按 `cid_number + pubkey_hex` 精确匹配，不读取旧账户主键记录。

激活 QR 与 node 桌面端统一使用 QR_V1 `a=5 activate_admin_account`；payload 前缀为 `GMB || 0x18`，机构主体字段固定为 `cid_number`，扫码端解码展示字段为 `institution_code / cid_number / pubkey`。

## 管理员与岗位展示

- `AdminAccountService` 对机构账户只解码 `admins` 钱包集合；`InstitutionAdminService` 再从对应 entity pallet 联合读取岗位定义和有效任职，并校验每个管理员钱包都有有效岗位。
- 机构管理员列表和公开机构管理员列表统一使用 `/Users/rhett/GMB/citizenapp/lib/citizen/institution/institution_assignment_card.dart`；展示岗位、任期、来源、账户和余额，不保存管理员姓名或公民 CID 副本。
- 个人多签仍由独立 `AdminAccount` 布局和个人管理员集合页面处理；`PersonalAdmins.propose_admin_set_change` 是 CitizenApp 唯一保留的管理员集合变更调用，公权/私权机构不得从客户端直接改写管理员集合。

## 2026-05-10 修复记录

- citizenapp 所有 admins-change 查询、激活、页面跳转入口已改为 `AdminAccountIdentity`。
- 个人多签与机构 CID 两类主体在 App 侧明确区分；机构具体账户不进入管理员集合变更或激活主键。
- 机构账户发现、提案上下文和本地多签实体都会携带 `adminSubjectInstitutionCode`（CID 机构码）；转账、管理员更换和投票匹配按机构账户码（`is_institution_code`）进入，不再把机构账户当作个人多签码（`is_personal_code`）。
- 管理员更换成功后按个人多签 `personal_account` 清理缓存；机构管理员查询和激活缓存只按 `cid_number` 管理。
- 通用 `OrgType.multisig` 文案改为“多签账户”，具体“个人多签 / 机构账户”由 admins-change identity 展示。
- 2026-06-27：管理员更换代码目录已迁到 `lib/citizen/proposal/admins-change/`；测试目录暂保留 `test/governance/admins-change/`，用于覆盖 QR call data、AdminAccounts storage key、非法人显式 kind 路由和激活缓存。
