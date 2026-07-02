# citizenapp 管理员更换模块技术文档

最新更新：2026-07-01。

## 模块定位

`citizenapp` 是 Flutter 客户端，不区分传统前端 / 后端，不新建 `backend/`。管理员更换作为一级业务模块放在：

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

1. `proposal_entry_page.dart` 的“换管理员”入口进入 `AdminSetChangePage`。
2. 入口页或调用方先构造 `AdminAccountIdentity`，再交给 `AdminAccountService` 查询目标 `AccountId`：
   - 内置治理机构：`0x01 Builtin + cidNumber`。
   - 个人多签：`PersonalAccount AccountId + AccountId`。
   - 机构账户：`InstitutionAccount AccountId + AccountId`。
3. 按机构码读取 `PersonalAdmins / PublicAdmins / PrivateAdmins` 的 `AdminAccounts` 并解码完整 `AdminAccount`；固定治理机构也读 `PublicAdmins`。
4. 用户选择管理员钱包、编辑完整管理员集合。
5. `AdminSetValidation` 做端上前置校验，同时校验目标阈值。
6. `AdminSetChangeCallCodec` 按机构码构造对应管理员 pallet 的 `propose_admin_set_change` call data。
7. `AdminSetChangeService` 通过 `SignedExtrinsicBuilder` 走热钱包或公民钱包签名并提交。

## 主体身份与查询门面

`/Users/rhett/GMB/citizenapp/lib/citizen/proposal/admins-change/models/admin_account.dart` 定义 `AdminAccountIdentity`，调用方必须显式传入三类主体之一：

- `governanceInstitution` / `fixedGovernanceInstitution`：固定治理公权主体，固定治理档机构码（NRC/PRC/PRB/FRG/NJD），`kind=0`。
- `institutionAccount`：公权机构账户主体，`kind=0`；私权机构账户主体，`kind=1`；非法人机构按所属法人归属选择 `kind=0` 或 `kind=1`。
- `personalAccount`：个人多签主体，个人多签码（PMUL），`kind=2`。

`/Users/rhett/GMB/citizenapp/lib/citizen/proposal/admins-change/services/institution_admin_service.dart` 是查询门面，但不接收模糊字符串身份；所有 `fetchAdmins / fetchThreshold / isAdmin / clearCache` 调用都必须传 `AdminAccountIdentity`。按单一字符串混用个人、机构、治理主体的入口不存在。

非法人机构码（`SFGT/SFGP/UNIN`）不是私权同义词。调用方必须从 CID 注册归属或链上 `AdminAccount.kind` 显式传入 `kind=0`（公权）或 `kind=1`（私权）；不得只凭机构码自动归入 `PrivateAdmins`。`OrganizationManage.propose_create_institution` 当前只直接创建公权法人或私权法人机构账户，裸非法人创建会被端上和链端拒绝。

## 管理员更换载荷与阈值

当前管理员更换载荷固定为：

```text
[pallet][call][institution_code:[u8;4]][account_id:32][admins:Compact<Vec<AccountId32>>][new_threshold:u32_le]
```

规则：

- PMUL 个人多签走 `PersonalAdmins(7).propose_admin_set_change(3)`。
- NRC/PRC/PRB/NJD 固定治理机构走 `PublicAdmins(29).propose_admin_set_change(0)`；FRG 省级组走 `PublicAdmins(29).propose_federal_registry_province_admin_set_change(2)`。
- 普通公权机构走 `PublicAdmins(29).propose_admin_set_change(0)`。
- 私权机构走 `PrivateAdmins(30).propose_admin_set_change(0)`。
- 非法人机构按所属法人归属走 `PublicAdmins(29).propose_admin_set_change(0)` 或 `PrivateAdmins(30).propose_admin_set_change(0)`。
- `new_threshold` 是载荷必填字段，端上和链端按同一字节结构构造、解析和签名。
- 固定治理机构不显示阈值输入框，`new_threshold` 固定为制度阈值：NRC=13，PRC=6，PRB=6，NJD=8；固定人数为 NRC=19，PRC/PRB=9，NJD=15；FRG 省级组固定为 3/5。
- 个人多签和机构账户显示动态阈值输入框，端上只做前置校验：`threshold * 2 > admins_len && threshold <= admins_len`。
- 阈值真源不在各管理员 `AdminAccounts`；治理固定阈值来自制度常量，动态阈值由 `InternalVote.ActiveDynamicThresholds` 保存。
- QR_V1 只携带 `b.a + b.d`；扫码端从 `b.d` 解码出的展示字段必须与冷钱包 decoder 逐项一致：`institution_code / subject / admins / new_threshold`。

## 管理员激活

管理员激活服务位于 `/Users/rhett/GMB/citizenapp/lib/citizen/proposal/admins-change/services/admin_activation_service.dart`。机构管理员列表和提案上下文只引用该服务，不再从 `lib/institution/` 承载激活逻辑。

激活记录使用 `activated_admins_v3`，只保存 `identityKey / accountIdHex / institutionCode / kind / pubkeyHex / activatedAtMs`，查询和清理都按 `accountIdHex + pubkeyHex` 精确匹配。

激活 QR 与 node 桌面端统一使用 QR_V1 `a=5 activate_admin_account`；payload 前缀为 `GMB || 0x18`，扫码端解码展示字段为 `institution_code / subject / pubkey`。

## 管理员资料展示

- `AdminAccountService` 返回的 `AdminAccountState.profiles` 是管理员展示真源；`admins` 只从 profiles 抽取账户，供签名、权限和投票校验使用。
- 机构管理员列表、公开机构管理员列表、管理员账户详情、管理员集合编辑器和变更差异卡统一使用 `/Users/rhett/GMB/citizenapp/lib/citizen/shared/admin_profile_card.dart`。
- UI 固定为顶部“序号/激活状态”、第 1 行“姓名:/职务:”、第 2 行“任期:/来源:”、第 3 行“身份CID:”、第 4 行“账户:”、第 5 行“余额:”；字段值为空时值区域留空，不隐藏标签、不用本地姓名兜底。余额通过 `ChainRpc.fetchFinalizedBalances` 批量读取 finalized `System.Account.free`，0 余额正常显示，查询失败才留空。个人多签只有账户时按 account-only 资料展示。

## 2026-05-10 修复记录

- citizenapp 所有 admins-change 查询、激活、页面跳转入口已改为 `AdminAccountIdentity`。
- 个人多签、机构账户、治理机构三类主体在 App 侧明确区分；`注册机构归属关系(0x02)` 不进入管理员更换。
- 机构账户发现、提案上下文和本地多签实体都会携带 `adminSubjectInstitutionCode`（CID 机构码）；转账、管理员更换和投票匹配按机构账户码（`is_institution_code`）进入，不再把机构账户当作个人多签码（`is_personal_code`）。
- 管理员更换成功后按 `accountIdHex` 清理缓存；投票执行返回和详情刷新仍可清理对应 identity。
- 通用 `OrgType.multisig` 文案改为“多签账户”，具体“个人多签 / 机构账户”由 admins-change identity 展示。
- 2026-06-27：管理员更换代码目录已迁到 `lib/citizen/proposal/admins-change/`；测试目录暂保留 `test/governance/admins-change/`，用于覆盖 QR call data、AdminAccounts storage key、非法人显式 kind 路由和激活缓存。
