# wuminapp 管理员更换模块技术文档

最新更新：2026-05-09。

## 模块定位

`wuminapp` 是 Flutter 客户端，不区分传统前端 / 后端，不新建 `backend/`。管理员更换作为一级业务模块放在：

```text
/Users/rhett/GMB/wuminapp/lib/admins_change/
```

边界：

- 不放在 `lib/proposal/` 下面；`proposal` 只作为入口页面和其它提案域的聚合位置。
- 不保留 `lib/proposal/admin_change/` 空占位。
- 不新建 `wuminapp/backend/`。
- 需要从 proposal 类型页跳转时，只在入口处引用 `admins_change/pages/admin_set_change_page.dart`。

## 目录结构

```text
wuminapp/lib/admins_change/
├── admins_change.dart
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
├── qr/
│   └── admin_set_change_qr_adapter.dart
├── controllers/
│   └── admin_set_change_controller.dart
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
wuminapp/test/admins_change/
├── admins_change_codec_test.dart
└── institution_admin_service_test.dart
```

## 业务流程

1. `proposal_types_page.dart` 的“换管理员”入口进入 `AdminSetChangePage`。
2. `AdminSubjectService` 按机构身份解析目标 `SubjectId`：
   - 内置治理机构：`0x01 Builtin + sfidNumber`。
   - 个人多签：`0x03 PersonalDuoqian + AccountId`。
   - 机构账户：`0x05 InstitutionAccount + AccountId`。
3. 读取 `AdminsChange::Subjects` 并解码完整 `AdminSubject`。
4. 用户选择管理员钱包、编辑完整管理员集合。
5. `AdminSetValidation` 做端上前置校验。
6. `AdminSetChangeCallCodec` 构造 `AdminsChange::propose_admin_set_change` call data。
7. `AdminSetChangeService` 通过 `SignedExtrinsicBuilder` 走热钱包或冷钱包签名并提交。

## 兼容入口

`/Users/rhett/GMB/wuminapp/lib/admins_change/services/institution_admin_service.dart` 保留原 public API，但实现委托到 `AdminSubjectService`。这是为了让机构详情、投票、提案详情等既有调用方继续使用同一查询门面；旧的 `lib/institution/institution_admin_service.dart` 不再保留。

管理员激活服务位于 `/Users/rhett/GMB/wuminapp/lib/admins_change/services/admin_activation_service.dart`。机构管理员列表和提案上下文只引用该服务，不再从 `lib/institution/` 承载激活逻辑。
