任务需求：
2026-05-06 任务卡 C「命名修正 institution_id → account_id」把 admins-change
storage 名 `Institutions` → `Subjects`,但**没写 migration**。链上数据停留在
老前缀 `AdminsChange::Institutions` 下,wuminapp + citizenchain 节点 UI 都按
新前缀 `Subjects` 查 → 读到空 → 治理机构详情页"管理员 0 人" / "提案不显示"。

本卡补 v1 → v2 storage rename migration,顺手处理两条客户端文案与水印问题。

所属模块：Blockchain + Mobile（wuminapp）

输入文档：
- memory/08-tasks/done/20260506-...wuminapp-公民三tab扁平化重构.md
- memory/project_account_id_naming_2026_05_06.md（C 阶段命名修正记录）
- memory/feedback_no_chain_restart.md（链不可重启,migration 走 setCode）
- memory/feedback_chainspec_frozen.md
- runtime/votingengine/internal-vote/src/migrations/v1.rs（move_prefix 模板）

必须遵守：
- 不重启链(feedback_no_chain_restart),靠 setCode + StorageVersion 升级
- 不动 chainspec.json(feedback_chainspec_frozen)
- pallet StorageVersion 自治门控,不询问 spec_version 是否 bump

## 范围

### 链端（admins-change v1 → v2）
- `runtime/governance/admins-change/src/migrations/v1.rs` 新建 `MigrateV1ToV2`
  - `move_prefix(twox128("AdminsChange") ++ twox128("Institutions"),
                 twox128("AdminsChange") ++ twox128("Subjects"))`
  - 门控 `on_chain_storage_version() >= 2` noop,二次 set_code 安全
  - try-runtime pre/post 校验:旧 prefix 全清空 + 新 prefix 数量 ≥ 旧
- `runtime/governance/admins-change/src/migrations/mod.rs` 新建,导出 v1
- `runtime/governance/admins-change/src/lib.rs` `STORAGE_VERSION` 1 → 2
  + `pub mod migrations;`
- `runtime/src/lib.rs` `Migrations` tuple 加
  `admins_change::migrations::v1::MigrateV1ToV2<Runtime>`
- `runtime/src/lib.rs` `spec_version: 0 → 1`(整链首次 setCode upgrade)
- `runtime/src/lib.rs:452` 测试断言 `spec_version: 0 → 1`

### 客户端（wuminapp）
- `lib/vote/vote_view.dart` 引言水印 `Opacity 0.06 → 0.20`(若隐若现可见)
- `lib/institution/institution_detail_page.dart`
  - 注释 `// ──── 投票事件列表 ────` → `// ──── 提案列表 ────`
  - 函数名 `_buildVotingEvents()` → `_buildProposalList()`(含调用点)
  - 标题 `'投票事件'` → `'提案列表'`
  - 空态 `'暂无投票事件'` → `'暂无提案'`
  - 副文案 `'本机构提案和全局联合投票事件将在此显示'` →
            `'本机构提案与全局联合投票将在此显示'`

### 不在本卡范围
- 公权机构数据扩容(lf/sf/jc/zf/jy 灌入)
- 公权页省/市垂直导航栏
- votingengine v0→v1 历史提案反向索引 backfill(独立观察,需要时再补)

## 验证

- ✅ `cargo check -p admins-change` 0 error
- ✅ `WASM_FILE=... cargo check --manifest-path runtime/Cargo.toml` 0 error
- ✅ `flutter analyze` 0 issues
- ✅ `flutter test` 141/141 passed
- ⏳ **用户最后一步**:
  1. CI 重新 build 出 spec_version=1 的 wasm
  2. 链上发起 `propose_runtime_upgrade` 联合投票(JointVote)
  3. 投票通过后 setCode 自动跑 `Migrations` tuple,move_prefix 完成
  4. 再次打开 wuminapp 治理 tab,应能看到管理员列表 + 提案

## 输出物
- 链端代码 + 中文注释 + try-runtime 校验
- 客户端代码 + 中文注释
- 任务卡(本文件)
- memory 固化:storage rename 必须配 migration

## 验收标准
- 链上 setCode 后 `AdminsChange::AdminAccounts` 取代 `Institutions`,管理员数据可读
- 治理 tab 详情页"管理员 N 人"显示正常
- 国储会等机构详情页"提案列表"出现历史提案
- 链端 0 编译错误,wuminapp 0 analyze issue + 141 测试全过
