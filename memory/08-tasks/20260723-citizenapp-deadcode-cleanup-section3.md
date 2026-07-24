# CitizenApp 死代码清理（全仓扫描第 3 节）

任务需求：清理 citizenapp 扫描出的孤儿文件与陈旧注释；对「有开放任务卡的在飞功能线」与「未接线的成品」不做删除。
所属模块：citizenapp（Mobile）— 纯前端；仅 1 处注释涉及 citizenchain primitives 只读对照，不改链端。

## 复核结论（推翻扫描的两个前提）

1. **`lib/asset/` 不是残桩**：是开放任务卡 [20260507-onchain-issuance-plain-ft.md](open/20260507-onchain-issuance-plain-ft.md) 第 91 行的交付物，子任务 C 未实装。四端中链端（pallet 1332 行，index 23）、qr-protocol（action 0x1700–0x1704）、citizenwallet（decoder 全实装）均已完成，只差 citizenapp UI。
2. **`legislation_vote_page.dart` 不是死代码**：454 行成品未接线，且上游 `ProposalKind.legislation` 为 `enabled: true`、卡片文案承诺「本端查看 + 投票」→ 正在生效的假入口。**移交独立任务卡处理，本卡不动。**

## 定稿（用户 2026-07-23 逐条确认）

1. `lib/asset/` 分层：shared + entity（369 行，跨端契约常量与 SCALE 解码）**保留**；pages + widgets（10 文件 221 行纯占位壳）**删除**。
2. `legislation_vote_page` 走**甲案**（接线，不删）→ 见 [20260723-citizenapp-legislation-vote-wiring.md](20260723-citizenapp-legislation-vote-wiring.md)。
3. `seed_vault_harness.dart` 删除条件已满足（`HardwareBoundSeedVault` 已在 `wallet_manager.dart:111` 生产接线）→ **删除**。

## 检测方法学修正（写入本卡防复发）

- **禁止按「文件首个 class」判定孤儿**：`admin_set_validation.dart` 首类引用 0 但主类 `AdminSetValidation` 被引 5 次；`institution_admin_service.dart` 首类 0 但 `InstitutionAdminService` 被引 18 次。必须逐符号计数。
- **备用入口对引用计数天生免疫**：`lib/dev/seed_vault_harness.dart` 是独立 `void main()`，靠 `flutter build --target` 调用，永远 0 引用。
- **生成物不计入死代码**：`chat_envelope.pbjson.dart` 是 protoc 标准产物（源 `chat/proto/chat_envelope.proto`），删除后重跑生成必然再生。

## 分步计划

- **Step 1**：注释修正（`account_derivation.dart:167-169`）。零风险先落地。
- **Step 2**：删除 7 个真孤儿文件，一批做完跑一次验证。
- **Step 3**：`lib/asset/` 分层处理 + 契约层锚点 + 订正开放任务卡陈旧数据。

## Step 1 落点

`lib/citizen/shared/account_derivation.dart:167-169` 两处错：

| 错处 | 改为 | 真源 |
|---|---|---|
| `OP_INSTITUTION`（链端零命中） | `OP_NAME` | `citizenchain/runtime/primitives/src/account_derive.rs:16`（永久冻结 0x00）；本文件上方 3 行代码用的正是 `kOpName` |
| `CID accounts/derive.rs` | `onchina accounts/derive.rs` | `citizenchain/onchina/src/institution/accounts/derive.rs` |

## Step 2 落点（7 文件删除）

| 文件 | 判据 |
|---|---|
| `lib/dev/seed_vault_harness.dart` | 自带删除条件已满足；删后 `lib/dev/` 空目录一并移除 |
| `lib/wallet/wallet.dart` | 5 行死 barrel，全 app 走 `wallet_gate.dart` |
| `lib/wallet/capabilities/wallet_type_service.dart` | `WalletLabelService` 仅经死 barrel 导出，无现役替代 |
| `lib/chat/storage/chat_memory_store.dart` | 与现役 `ChatStore` 并存，自陈供测试用但无测试引用 |
| `lib/citizen/proposal/admins-change/models/admin_set_change.dart` | `AdminsChangeDraft` 全仓 0 引用 |
| `lib/citizen/proposal/admins-change/admin_set_change_controller.dart` | `AdminsChangeController` 全仓 0 引用 |
| `lib/citizen/proposal/admins-change/pages/admin_account_detail_page.dart` | `AdminAccountDetailPage` 全仓 0 引用 |

删 barrel 前须确认其余 4 个导出目标（`wallet_manager` / `wallet_secure_keys` / `wallet_page` / `attestation_service`）另有直接 import，不随 barrel 变孤儿。

**admins-change 定性**：该目录 19 文件中 16 个是活的（`AdminAccountIdCodec` 被 13 个文件引用），memory `admin-change-source-model`（推迟到链端）**不适用**；这 3 个是活模块里的重构残渣。

## Step 3 落点

- 删除 `lib/asset/pages/`（7 文件）与 `lib/asset/widgets/`（3 文件），全部为 `return Text('XX占位')` + `// TODO`。
- `lib/asset/shared/onchain_asset_constants.dart` 顶部补锚点注释：指向开放任务卡、注明「子任务 C 未实装，本目录为跨端契约常量层，非死代码」。
- 订正 [open/20260507-onchain-issuance-plain-ft.md](open/20260507-onchain-issuance-plain-ft.md)：
  - 第 206 行 `onchainIssuancePalletIndex=25` → `23`（链端 `construct_runtime` 与 app/wallet 三方实际均为 23，文档错、代码对）。
  - 删除对 pages/widgets 骨架文件的交付记录，避免子任务 C 照着找不存在的文件。

## 边界

- 不动 citizenchain / qr-protocol / citizenwallet 任何一行。
- 不动 `lib/asset/shared/` 与 `lib/asset/entity/` 的实现内容（仅加锚点注释）。
- 不动 `legislation_vote_page.dart` 及其同目录 service（归属接线任务卡）。
- 不动 `chat_envelope.pbjson.dart`（生成物）。
- 不动 `admin_set_validation.dart` / `institution_admin_service.dart`（活文件，扫描误判）。

## 验收

- `flutter analyze lib` 零问题（基线：零问题）。
- `flutter test --concurrency=1` 793 过 / 5 skip / 0 失败（基线不变）。
- `dart format` 无待格式化改动。
- 删除项全仓 grep 零残留引用。

---

## 2026-07-23 逐行复核补记（独立入口二次核验）

对已落地的 17 个删除项做了逐行 diff 复核，**16 项判据成立**，其中 admins-change 三件套的判据可以写得更强：不止「0 引用」，而是**有现役替代**——`admin_set_change_page.dart:104` 已承载同样的 `AdminAccountState` + `AdminAccountCard`，且 `AdminAccountCard` 未成孤儿。

### 发现并已修复：`wallet_type_service.dart` 删得不干净

原判据「仅经死 barrel 导出，无现役替代」不准确。该文件**不是空壳**，是 144 行完整实现（扫 `PublicAdmins`/`PrivateAdmins`/`PersonalAdmins` 三 pallet → 按机构码去重派生标签 → Isar 缓存 + 300s TTL + 链不可用兜底）。删除它之后，其专属持久化层成为**孤儿**：

- `lib/isar/app_isar.dart` 的 `@collection class AdminGroupCacheEntity` 声明
- 同文件 schema 列表里的 `AdminGroupCacheEntitySchema` 注册
- `lib/isar/app_isar.g.dart` 中 979 行生成代码

**已补删**（用户 2026-07-23 确认「补删干净」而非恢复接线，理由：`resolveWalletLabel` 从未接线，钱包徽章区 `wallet_page.dart:807-821` 现役只有「身份钱包」「默认用户」两个 badge，无机构标签展示位，故不存在功能回归；开发期零用户无需 migration）：删声明 + 删 schema 注册 + 重跑 `build_runner` 重生 `app_isar.g.dart`。全仓 `AdminGroupCache` / `adminGroupName` grep 零残留。

### 检测方法学再补一条（写入本卡防复发）

- **删 service 必须连带检查其专属持久化 schema**：service 与它独占的 Isar collection / 缓存表是一体的。只删 service 会留下无写入方也无读取方的孤儿 collection 和成百上千行生成代码，违反死规则 `no-remnants`。判定「某 service 可删」时，必须同时 grep 它读写的 Entity 是否还有别的消费方。
