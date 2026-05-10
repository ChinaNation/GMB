# wuminapp 管理员更换模块技术文档

最新更新：2026-05-10。

## 模块定位

`wuminapp` 是 Flutter 客户端，不区分传统前端 / 后端，不新建 `backend/`。管理员更换作为一级业务模块放在：

```text
/Users/rhett/GMB/wuminapp/lib/governance/admins-change/
```

边界：

- 不新建 `wuminapp/backend/`。
- 需要从治理提案聚合页跳转时，只在入口处引用 `lib/governance/admins-change/pages/admin_set_change_page.dart`。

## 目录结构

```text
wuminapp/lib/governance/admins-change/
├── admin_set_change_controller.dart
├── admin_set_change_qr_adapter.dart
├── models/
│   ├── admin_subject.dart
│   ├── admin_set_change.dart
│   └── admin_set_change_result.dart
├── codec/
│   ├── subject_id_codec.dart
│   ├── admin_subject_codec.dart
│   └── admin_set_change_call_codec.dart
├── services/
│   ├── admin_activation_service.dart
│   ├── admin_subject_service.dart
│   ├── institution_admin_service.dart
│   ├── admin_set_validation.dart
│   └── admin_set_change_service.dart
├── pages/
│   ├── admin_subject_detail_page.dart
│   ├── admin_set_change_page.dart
│   └── admin_set_change_confirm_page.dart
└── widgets/
    ├── admin_subject_card.dart
    ├── admin_set_editor.dart
    ├── admin_set_diff_card.dart
    └── admin_set_change_action_bar.dart
```

测试目录：

```text
wuminapp/test/governance/admins-change/
├── admins_change_codec_test.dart
└── institution_admin_service_test.dart
```

## 业务流程

1. `proposal_types_page.dart` 的“换管理员”入口进入 `AdminSetChangePage`。
2. 入口页或调用方先构造 `AdminSubjectIdentity`，再交给 `AdminSubjectService` 查询目标 `SubjectId`：
   - 内置治理机构：`0x01 Builtin + sfidNumber`。
   - 个人多签：`0x03 PersonalDuoqian + AccountId`。
   - 机构账户：`0x05 InstitutionAccount + AccountId`。
3. 读取 `AdminsChange::Subjects` 并解码完整 `AdminSubject`。
4. 用户选择管理员钱包、编辑完整管理员集合。
5. `AdminSetValidation` 做端上前置校验。
6. `AdminSetChangeCallCodec` 构造 `AdminsChange::propose_admin_set_change` call data。
7. `AdminSetChangeService` 通过 `SignedExtrinsicBuilder` 走热钱包或冷钱包签名并提交。

## 主体身份与查询门面

`/Users/rhett/GMB/wuminapp/lib/governance/admins-change/models/admin_subject.dart` 定义 `AdminSubjectIdentity`，调用方必须显式传入三类主体之一：

- `governanceInstitution`：治理机构主体，`org=0/1/2`，`kind=0`。
- `personalDuoqian`：个人多签主体，`org=3`，`kind=2`。
- `institutionAccount`：机构账户主体，`org=4/5`，`kind=3`。

`/Users/rhett/GMB/wuminapp/lib/governance/admins-change/services/institution_admin_service.dart` 是查询门面，但不再接收模糊字符串身份；所有 `fetchAdmins / fetchThreshold / isAdmin / clearCache` 调用都必须传 `AdminSubjectIdentity`。历史上按单一字符串混用个人、机构、治理主体的入口已删除。

## 管理员激活

管理员激活服务位于 `/Users/rhett/GMB/wuminapp/lib/governance/admins-change/services/admin_activation_service.dart`。机构管理员列表和提案上下文只引用该服务，不再从 `lib/institution/` 承载激活逻辑。

激活记录使用 `activated_admins_v3`，只保存 `identityKey / subjectIdHex / org / kind / pubkeyHex / activatedAtMs`。旧 `activated_admins_v1/v2` 不读取、不迁移，避免把旧 `sfidNumber` 语义带回管理员更换模块。

激活 QR 与 node 桌面端统一使用 `GMB_ACTIVATE_SUBJECT_V1 / activate_admin_subject`，字段为 `org / subject / pubkey`；旧 `GMB_ACTIVATE + sfidNumber` 只作为历史协议，不再作为当前 admins-change 激活入口。

## 2026-05-10 修复记录

- wuminapp 所有 admins-change 查询、激活、页面跳转入口已改为 `AdminSubjectIdentity`。
- 个人多签、机构账户、治理机构三类主体在 App 侧明确区分；`SfidInstitution(0x02)` 不进入管理员更换。
- 机构账户发现、提案上下文和本地多签实体都会携带 `adminSubjectOrg`；转账、管理员更换和投票匹配按 `ORG_PUP / ORG_OTH` 进入，不再把机构账户当作 `ORG_REN`。
- 管理员更换成功后按 `subjectIdHex` 清理缓存；投票执行返回和详情刷新仍可清理对应 identity。
- 通用 `OrgType.duoqian` 文案改为“多签账户”，具体“个人多签 / 机构账户”由 admins-change identity 展示。
- 本机 `flutter test test/governance/admins-change` 被 Flutter SDK 缓存写权限阻断：`/Users/rhett/flutter/bin/cache/engine.stamp: Operation not permitted`；`dart test` 因 Flutter 项目未引入 `package:test` 不能替代。当前已通过 `dart analyze` 与残留扫描完成验证。
