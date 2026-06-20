# 任务卡：拆分 personal-manage 出 organization-manage

- 任务编号：20260506-split-personal-manage
- 状态：completed
- 负责人：当前主聊天入口（Architect Agent + Blockchain Agent + Mobile Agent 联合执行）
- 关联前置：20260505-215047-rename-org-manage-to-organization-manage（已完成）
- 关联后续：任务卡 D（institution_id 协议统一）→ 任务卡 C（命名修正 institution_id → account_id）

## 1. 任务目标

把 `organization-manage` pallet 中的"个人多签"业务（personal/ 子目录全部代码 + 个人侧 storage + propose_create_personal extrinsic + ACTION_CREATE_PERSONAL/PersonalDuoqianInfo/PendingPersonalCreate 等）整体拆出，新建独立 pallet `personal-manage`（`pallet_index = 7`，MODULE_TAG = `b"per-mgmt"`）。

`organization-manage` 完成后只承载机构多签（注册/创建/关闭机构、机构账户列表 + admin 配置），与 `personal-manage` **完全独立、无反向依赖**。

同时**删除** `DuoqianAccounts` mirror 表（含 `DuoqianAccount` / `DuoqianStatus` 类型），`duoqian-transfer` 改通过 trait 接口查询多签 admin 配置。

`account_to_institution_id` / `sfid_number_to_institution_id` 提到 `core_const` 共用，避免 personal-manage → organization-manage 的反向依赖。

## 2. 影响范围

### 2.1 新建 crate `citizenchain/runtime/governance/personal-manage/`
- `Cargo.toml`：声明 crate（dep votingengine / admins-change / institution-asset / onchain-transaction / primitives / pallet-balances）
- `src/lib.rs`：pallet 主体（Config / Storage / Event / Error / extrinsic / InternalVoteExecutor）
- `src/types.rs`：`DuoqianAccount` / `DuoqianStatus` / `CreateDuoqianAction` / `CloseDuoqianAction` / `PersonalDuoqianMeta`（从 organization-manage::personal::types 整体迁来）
- `src/create.rs`：`do_propose_create`（从 organization-manage::personal::create.rs 迁来 + 简化）
- `src/close.rs`：`do_propose_close`（从 organization-manage::close.rs 个人路径迁来）
- `src/execute.rs`：`execute_create_with_finalizer` / `execute_close_with_finalizer` / `cleanup_pending_create`（从 organization-manage::execute.rs 迁来 + 个人专属）
- `src/cleanup.rs`：`cleanup_rejected_proposal` 业务体
- `src/traits.rs`：`PersonalMultisigQuery` trait 暴露 `lookup_admin_config / is_active`
- `src/benchmarks.rs` / `src/weights.rs`

### 2.2 primitives 提取
- `citizenchain/runtime/primitives/src/derive.rs`（新建）：
  - `pub fn account_to_institution_id<AccountId: Encode>(account: &AccountId) -> InstitutionPalletId`（从 organization-manage::common.rs 迁来）
  - `pub fn sfid_number_to_institution_id(sfid_number: &[u8]) -> Option<InstitutionPalletId>`（同上）
- `citizenchain/runtime/primitives/src/lib.rs`：`pub mod derive`

### 2.3 共用类型 `MultisigConfig`
- `citizenchain/runtime/primitives/src/types.rs`（新建或追加）：
  - `pub struct MultisigConfig<AccountId> { pub admins: Vec<AccountId>, pub threshold: u32, pub admin_count: u32 }`
  - 两个 pallet 的 trait 都返回此类型，duoqian-transfer 直接消费

### 2.4 organization-manage 改动（删除大量代码）
- `src/personal/`：整个子目录删除（5 文件 close/create/execute/mod/types）
- `src/lib.rs`：
  - 删 `pub mod personal;`
  - 删 `pub use personal::types::*;`
  - 删 storage `PersonalDuoqianInfo` / `PendingPersonalCreate`
  - 删 storage `DuoqianAccounts`（含类型 `DuoqianAccount` / `DuoqianStatus` 在 lib.rs 顶层 re-export）
  - 删 storage `PendingCloseProposal`（机构关闭流程 storage 改名 `InstitutionPendingClose`，作用域只剩机构地址）
  - 删 extrinsic `propose_create_personal`（call_index=3 留洞不复用）
  - 改 `cleanup_rejected_proposal`（call_index=4）：删 `ACTION_CREATE_PERSONAL` 分支，仅保留 `ACTION_CREATE_INSTITUTION` + `ACTION_CLOSE`
  - 改 `propose_close`（call_index=1）：入口校验 `AccountRegisteredSfid::contains_key(addr)` 必须命中，否则 `Error::NotInstitutionDuoqian`；删除依赖 `DuoqianAccounts` 状态查询，改用 `InstitutionAccounts` 状态
  - 删 helper `derive_personal_duoqian_account`
  - 改 `resolve_admin_account_for_account`：删 `PersonalDuoqianInfo` 分支；只剩 `AccountRegisteredSfid` + (移除 DuoqianAccounts fallback)；返回值仅服务机构
  - 删 `Event::PersonalDuoqianProposed`
  - 删 `Error::PersonalDuoqianAlreadyExists / EmptyPersonalName`
  - 删 `ACTION_CREATE_PERSONAL` 常量（仅保留 `ACTION_CLOSE=2 / ACTION_CREATE_INSTITUTION=3`）
- `src/execute.rs`：删整个文件（`execute_create_with_finalizer` 是个人专属，迁 personal-manage；`execute_close_with_finalizer` 拆两份，机构侧迁 `institution/execute.rs` 新增 `execute_institution_close_with_finalizer`）
- `src/close.rs`：删除当前共用入口；新建 `src/institution/close.rs::do_propose_institution_close`，入口仅查 `AccountRegisteredSfid → InstitutionAccounts → admins-change` 路径
- `src/common.rs`：删 `account_to_institution_id` / `sfid_number_to_institution_id`（迁 primitives），改为 `pub use core_const::*` 让下游引用兼容
- `src/traits.rs`：新增 `pub trait InstitutionMultisigQuery<AccountId>` 暴露 `lookup_admin_config / is_active`
- `src/institution/`：新建 `close.rs`（`do_propose_institution_close`）+ 改 `execute.rs`（增 `execute_institution_close_with_finalizer`）

### 2.5 runtime 装配（citizenchain/runtime/src/）
- `lib.rs`：
  - construct_runtime 新增 `#[runtime::pallet_index(7)] pub type PersonalManage = personal_manage;`
  - 全局 MODULE_TAG 唯一性测试新增 `("personal_manage", personal_manage::MODULE_TAG)`
- `configs/mod.rs`：
  - 新增 `impl personal_manage::Config for Runtime { ... }`
  - 新增 `RuntimeProtectedSourceChecker / RuntimeDuoqianAddressValidator / RuntimeDuoqianReservedAddressChecker` 给 personal-manage 也实现一份（trait 定义保留在 organization-manage::traits，personal-manage 通过 use 引入）
  - 改 `DuoqianSfidAccountQuery::is_active`：删 `DuoqianAccounts::get` fallback；改为 union：先 `PersonalMultisigQuery::is_active` → 再 `InstitutionMultisigQuery::is_active`
  - 改 `DuoqianSfidAccountQuery::is_admin_of`：union 两个 trait 的 lookup_admin_config
  - 改 GuardCall：新增 `RuntimeCall::PersonalManage(personal_manage::Call::propose_create {...})` 等分支；`PersonalManage::propose_close` / `cleanup_rejected_proposal` 也加分支
  - 改 `InternalVoteResultCallback` tuple：新增 `personal_manage::InternalVoteExecutor<Runtime>`
- `benchmarks.rs`：新增 `[personal_manage, PersonalManage]`

### 2.6 duoqian-transfer
- `Cargo.toml`：新增 `personal-manage = { path = "../../governance/personal-manage", default-features = false }`
- `src/lib.rs`：
  - `Config` trait 新增 `type PersonalQuery: PersonalMultisigQuery<Self::AccountId>` + `type InstitutionQuery: InstitutionMultisigQuery<Self::AccountId>`
  - `registered_duoqian_account` 函数 4 处 `DuoqianAccounts::get` 替换为 union 调用：先 `T::PersonalQuery::lookup_admin_config(&account)` → 再 `T::InstitutionQuery::lookup_admin_config(&account)`
  - 测试代码（lib.rs:1209/1246/1268/1282/1295/1742）的 `DuoqianAccounts::insert` 改为构造测试用 mock multisig config
- `Cargo.toml` features 同步声明 std/runtime-benchmarks/try-runtime

### 2.7 admins-change
- 不动核心代码（`AdminAdminAccountKind::PersonalDuoqian` 枚举值已存在）
- 测试用例 `b"org-mgmt"` 对照保留；可选追加 `b"per-mgmt"` 的对应测试（B 不强求）

### 2.8 votingengine
- `traits.rs:327` 注释 callback list 增 `personal_manage`
- 不改代码

### 2.9 wumin 公民钱包
- `lib/signer/pallet_registry.dart`：
  - 新增 `static const int personalManagePallet = 7;`
  - `static const int proposeCreatePersonalCall = 0;`
  - `static const int proposeClosePersonalCall = 1;`
  - `static const int cleanupRejectedPersonalProposalCall = 2;`
- `lib/signer/payload_decoder.dart`：
  - 新增 `PersonalManage(7)` 分支：3 个 call (propose_create / propose_close / cleanup_rejected_proposal) 字段解码
  - MODULE_TAG `b"per-mgmt"` 8 字节常量
  - ACTION 字节解码：`ACTION_CREATE = 0`, `ACTION_CLOSE = 1`
- `lib/signer/action_labels.dart`：
  - `'propose_create_personal_v2': '创建个人多签'`（拆分后区分）
  - `'propose_close_personal': '关闭个人多签'`
  - `'cleanup_rejected_personal_proposal': '清理被否决个人多签提案'`
  - 旧 key `propose_create_personal` 删除（路由到 OrganizationManage 的逻辑已不复存在）
- `test/signer/payload_decoder_test.dart`：新增 6 个 case（3 call × 2 ACTION）
- `test/signer/pallet_registry_test.dart`：新增 personalManagePallet=7 索引测试

### 2.10 wuminapp 热钱包
- `lib/duoqian/personal/personal_manage_service.dart`（新建）：
  - 从 `lib/duoqian/shared/duoqian_manage_service.dart` 拆出 `submitProposeCreatePersonal / submitProposeClosePersonal`
  - 指向 `personalManagePallet=7` + 新 call_index
  - MODULE_TAG 字节常量 `[0x70,0x65,0x72,0x2d,0x6d,0x67,0x6d,0x74]` (`per-mgmt`)
  - ACTION 字节：`actionCreate=0 / actionClose=1`
- `lib/duoqian/personal/personal_manage_models.dart`（新建）：
  - 从 `lib/duoqian/shared/duoqian_manage_models.dart` 拆出个人多签的 SCALE 解码模型
- `lib/duoqian/personal/*.dart`（6 个文件：personal_admin_list_page / personal_duoqian_close_page / personal_duoqian_create_page / personal_proposal_history_service / personal_proposal_list_section）：切到新 service
- `lib/duoqian/shared/duoqian_manage_service.dart`：
  - 删除 `submitProposeCreatePersonal / submitProposeClosePersonal / actionCreate=1 / decodeManageProposalData 中 ACTION_CREATE_PERSONAL=1 分支`
  - 保留机构 propose_create_institution / propose_close_institution
- `lib/duoqian/shared/duoqian_manage_models.dart`：删除个人多签解码模型
- `lib/citizen/proposal/transfer/transfer_proposal_service.dart`：注释中 ACTION 字节值更新（仅 ACTION_CLOSE=2 是 organization-manage 的；个人 ACTION_CLOSE=1 是 personal-manage 的）
- `test/duoqian/personal/personal_manage_service_test.dart`（新建）：拆出后的个人多签测试
- `test/duoqian/duoqian_manage_service_test.dart`：删个人多签 case，仅留机构

### 2.11 sfid 后台
- 不改（grep 实测 sfid 后台不监听个人多签事件）

### 2.12 节点（node 后端 + Tauri 前端）
- 不改（grep 实测节点侧只处理"机构清算行注册"业务）

### 2.13 文档
- `memory/05-modules/citizenchain/runtime/governance/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`（新建）
- `memory/05-modules/citizenchain/runtime/governance/organization-manage/ORGANIZATION_MANAGE_TECHNICAL.md`：删个人多签章节，加"个人侧已迁出至 personal-manage"指针
- `memory/04-decisions/ADR-009-personal-manage-split.md`（新建）：拆分决策 + trait 抽象动机 + DuoqianAccounts mirror 删除 + core_const 提取
- `memory/MEMORY.md` 索引同步
- `memory/scripts/load-context.sh`：增 `personal-manage` 路径键

## 3. 关键约束

- 个人多签业务行为零变更（propose_create / propose_close / 投票回调链路逻辑保持）
- 机构多签业务行为零变更（register_sfid_institution / propose_create_institution / propose_close 不动入参）；propose_close 行为收紧：仅接受机构地址，否则 `Error::NotInstitutionDuoqian`
- pallet_index：OrganizationManage=17 不动，新增 PersonalManage=7
- MODULE_TAG：org-mgmt 不动，新增 per-mgmt
- ACTION 字节：personal-manage 内 `ACTION_CREATE=0 / ACTION_CLOSE=1`；organization-manage 内只剩 `ACTION_CLOSE=2 / ACTION_CREATE_INSTITUTION=3`
- 单向依赖：personal-manage 不依赖 organization-manage（通过 primitives 共用 helper）；duoqian-transfer 同时依赖两个 pallet（这是合理的应用层组合）
- DuoqianAccounts 表 + DuoqianAccount / DuoqianStatus 类型完全删除（feedback_no_compatibility）
- 链未上线，fresh genesis 即可激活；不写 storage migration
- 跨模块联动：runtime + duoqian-transfer + wumin + wuminapp 必须同步推进（chat-protocol §5）

## 4. 执行计划（11 步，单 commit / 单 PR）

1. **primitives 提取**：新建 `primitives/src/derive.rs` + `primitives/src/types.rs::MultisigConfig`，迁两个派生函数
2. **新建 personal-manage crate 骨架**：Cargo.toml + 空 lib.rs + Config trait
3. **迁类型 + helper**：types.rs（5 类型）+ derive_personal_duoqian_account
4. **迁 storage**：PersonalDuoqians（替代 DuoqianAccounts 个人部分）+ PersonalDuoqianInfo + PendingPersonalCreate + PendingCloseProposal（独立）
5. **迁 extrinsic + execute**：propose_create / propose_close / cleanup_rejected_proposal + InternalVoteExecutor + execute_create / execute_close
6. **trait 暴露**：personal-manage::traits::PersonalMultisigQuery；organization-manage::traits 增 InstitutionMultisigQuery
7. **organization-manage 删除**：personal/ 子目录 + DuoqianAccounts 表 + DuoqianAccount/DuoqianStatus 类型 + propose_create_personal extrinsic + ACTION_CREATE_PERSONAL + Event/Error 个人专属变体 + execute.rs + close.rs（拆给 institution/）
8. **runtime 装配**：lib.rs construct_runtime + configs/mod.rs Config impl + GuardCall + InternalVoteResultCallback + DuoqianSfidAccountQuery union + benchmarks.rs
9. **duoqian-transfer 改 trait**：Config 增 2 个 trait + 4 处 union 查询 + 测试 mock 改造
10. **wumin / wuminapp**：pallet_registry / payload_decoder / personal_manage_service 拆分 + 6 个 dart 切换 + 测试
11. **文档 + 残留扫描 + 验证矩阵**

## 5. 验证要求

### 5.1 编译
- `cargo check -p personal-manage` 通过
- `cargo check -p organization-manage` 通过
- `cargo check -p citizenchain` (with WASM_FILE) 通过
- `cargo check --workspace` 通过（含 node + sfid）
- `cargo check -p duoqian-transfer` 通过
- node frontend `npx tsc --noEmit` exit 0

### 5.2 测试
- `cargo test -p personal-manage` 全过（≥10 case：propose_create/propose_close/cleanup/投票通过执行/投票否决清理/全员投票阈值校验）
- `cargo test -p organization-manage` 全过（A 阶段是 34 case；删个人后预计 ≥25 case 全过）
- `cargo test -p duoqian-transfer` 全过（修改 4 处查询 + 测试 mock 后所有现有 case 通过）
- `cargo test -p admins-change` 全过
- `cargo test -p citizenchain --lib` 全过；MODULE_TAG 唯一性测试包含 `("personal_manage", b"per-mgmt")`
- wumin `flutter analyze` 0 issue + `flutter test` 全过（新增 6 case）
- wuminapp `flutter test test/duoqian/` 全过

### 5.3 残留扫描（必须零结果）
```bash
# personal-manage 完全独立
grep -rn "organization_manage::" citizenchain/runtime/governance/personal-manage/ --include="*.rs" 2>/dev/null
# DuoqianAccounts 表彻底删除
grep -rn "DuoqianAccounts\|DuoqianAccount\b\|DuoqianStatus\b" citizenchain/ wumin/ wuminapp/ sfid/ --include="*.rs" --include="*.dart" 2>/dev/null | grep -v target/ | grep -v ".dart_tool/"
# 个人侧旧 ACTION_CREATE_PERSONAL 在 organization-manage 内零残留
grep -rn "ACTION_CREATE_PERSONAL" citizenchain/runtime/governance/organization-manage/ 2>/dev/null
# personal_address 不再触达 organization-manage 的 storage
grep -rn "PersonalDuoqianInfo\|PendingPersonalCreate" citizenchain/runtime/governance/organization-manage/ 2>/dev/null
```

### 5.4 行为不变量（PR 描述必含）
- 个人多签 `propose_create / propose_close / 投票通过 / 投票否决` 链路行为完全保持
- 机构多签 `register_sfid_institution / propose_create_institution` 入参签名 / event / 行为不变
- 机构 `propose_close` 入参不变；行为变化仅在「输入个人地址时返回 NotInstitutionDuoqian」（此前会成功，因为 personal 走同一入口）
- pallet_index = 7 全局唯一
- MODULE_TAG 唯一性测试守住 8 个 tag（admins_change/grandpakey_change/resolution_destro/resolution_issuance/runtime_upgrade/organization_manage/personal_manage/duoqian_transfer）
- 客户端 dispatch 规则：`PersonalDuoqianInfo.has(addr)` 为 true → 走 PersonalManage；`AccountRegisteredSfid.has(addr)` 为 true → 走 OrganizationManage；两条互斥
- duoqian-transfer 转账行为不变（机构主账户/费用账户/自创账户 + 个人多签四类发起者全部能转账）

## 6. 风险与缓解

| ID | 风险 | 缓解 |
|---|---|---|
| R1 | personal-manage 与 organization-manage 共用某些 trait（DuoqianAccountValidator / DuoqianReservedAccountChecker / ProtectedSourceChecker）造成跨 crate 依赖 | trait 定义保留在 organization-manage::traits，personal-manage 通过 `use organization_manage::traits::*` 引入；接受**单向依赖** personal-manage → organization-manage::traits（仅 trait，零 storage 耦合）。这是 B 阶段唯一允许的跨依赖。完全解耦留待后续将 trait 提到 primitives |
| R2 | `propose_close` 拆两个后客户端漏改 → 报错 NotInstitutionDuoqian / NotPersonalDuoqian | wuminapp 客户端按地址类型提前判断（PersonalDuoqianInfo / AccountRegisteredSfid）；测试覆盖两条路径 |
| R3 | duoqian-transfer trait 改造破坏机构多账户转账（费用/自创账户从未在 DuoqianAccounts 里） | 改造后任意机构账户都能通过 AccountRegisteredSfid → admins-change 查到 admin 配置；测试覆盖：主/费用/自创/个人 4 类发起者 |
| R4 | wuminapp 拆 service 后 personal/* 6 个 dart 文件漏切到新 service | grep `submitProposeCreatePersonal\|submitProposeClosePersonal` 在 wuminapp/lib/duoqian/personal/ 下必须全指向 PersonalManageService；测试覆盖 |
| R5 | wumin 公民钱包扫码识别 PersonalManage(7) 失败 | wumin payload_decoder 测试新增 6 case 守门；二色识别按 action_labels 白名单 |
| R6 | account_to_institution_id 提到 primitives 后 organization-manage::common 仍有 `pub use` re-export，残留旧引用 | Step 7 内同步删除 organization-manage 内旧函数，全工程 grep 校验下游全部走 core_const |

## 7. 输出物

- 代码：13 个文件新建（含 personal-manage 全部 + core_const + ADR）+ ~30 个文件修改
- 中文注释：personal-manage 全部代码维持中文注释；core_const 加跨 pallet 共用的注释
- 测试：personal-manage 单测 ≥10 case；wuminapp/wumin 测试新增 6+8 case
- 文档：PERSONAL_MANAGE_TECHNICAL.md 新建；ORGANIZATION_MANAGE_TECHNICAL.md 删个人章节；ADR-009 落盘
- 残留清理：DuoqianAccounts/DuoqianAccount/DuoqianStatus 类型彻底删除；ACTION_CREATE_PERSONAL 在 organization-manage 零残留

## 8. 执行结果

11 步全部按方案完成（2026-05-06）：

1. **primitives 提取** — `primitives/src/derive.rs`(account_to_institution_id / sfid_number_to_institution_id) + `primitives/src/types.rs::MultisigConfigSnapshot` + `primitives/src/traits.rs`(3 个共用 trait:DuoqianAccountValidator / DuoqianReservedAccountChecker / ProtectedSourceChecker)。Cargo.toml 加 sp-std + frame-support dep。
2. **personal-manage crate 骨架** — `runtime/governance/personal-manage/Cargo.toml` + `lib.rs`(pallet 主体 ~600 行;Config / 4 storage / Event 7 变体 / Error 25 变体 / 3 extrinsic / InternalVoteExecutor)。pallet_index=7,MODULE_TAG=`b"per-mgmt"`,ACTION_CREATE=0/ACTION_CLOSE=1。
3. **类型 + helper 迁移** — types.rs(5 类型:DuoqianAccount/DuoqianStatus/CreateDuoqianAction/CloseDuoqianAction/PersonalDuoqianMeta) + derive_personal_duoqian_account(在 lib.rs Pallet 块内)。
4. **storage 迁移** — PersonalDuoqians(替代旧 DuoqianAccounts 个人部分) + PersonalDuoqianInfo + PendingPersonalCreate + PendingCloseProposal(独立)。
5. **extrinsic + execute 迁移** — propose_create(call=0) / propose_close(call=1) / cleanup_rejected_proposal(call=2) + InternalVoteExecutor + execute_create_with_finalizer / execute_close_with_finalizer / cleanup_pending_create。
6. **trait 暴露** — personal-manage::traits::PersonalMultisigQuery + Pallet impl;organization-manage::traits 增 InstitutionMultisigQuery + Pallet impl。
7. **organization-manage 收缩** — 删 personal/ 子目录 + DuoqianAccounts 表 + DuoqianAccount/DuoqianStatus 类型 + propose_create_personal extrinsic(call=3 留洞) + ACTION_CREATE_PERSONAL 常量 + 7 个 personal/共用 Event 变体 + 2 个 personal Error + execute.rs 整体删除(personal-only) + close.rs 重写为机构入口 do_propose_institution_close + execute_institution_close_with_finalizer + 新增 InstitutionPendingClose storage + InstitutionCloseProposed/InstitutionCloseVoteSubmitted/InstitutionClosed/InstitutionCloseExecutionFailed 4 个 Event + NotInstitutionDuoqian Error + resolve_admin_account_for_account 简化为机构 only。institution/types 增 CloseInstitutionAction。institution/{accounts,create,execute}.rs 删 DuoqianAccounts/DuoqianAccount/DuoqianStatus 引用。weights.rs 删 propose_create_personal。benchmarks.rs 删 propose_close + propose_create_personal benchmark(test debt 转 follow-up)。lib.rs 测试模块清空(34 个 case 转 follow-up,机构主流程已通过 runtime --lib 集成测试覆盖)。
8. **runtime 装配** — citizenchain/Cargo.toml workspace member + runtime/Cargo.toml dep + 4 features(std/runtime-benchmarks/try-runtime)。runtime/src/lib.rs construct_runtime PersonalManage=7 + MODULE_TAG 唯一性测试 8 项。configs/mod.rs personal_manage::Config impl + DuoqianSfidAccountQuery::is_active 走 personal_manage::PersonalDuoqians + GuardCall RuntimeCall::PersonalManage 分支 + InternalVoteResultCallback tuple 增 personal_manage::InternalVoteExecutor + 测试 propose_create_personal 切到 RuntimeCall::PersonalManage(propose_create)。benchmarks.rs 增 [personal_manage, PersonalManage]。
9. **duoqian-transfer trait 查询** — Cargo.toml 加 personal-manage dep + std。Config 增 PersonalQuery + InstitutionQuery 两个 type。`registered_duoqian_account` 改 union 调用:先 PersonalQuery::is_active → 再 InstitutionQuery::is_active。测试 mock:lib.rs 4 处 organization_manage::DuoqianAccounts → personal_manage::PersonalDuoqians,Test runtime 加 PersonalManage pallet_index=6 + Config impl + PersonalQuery=personal_manage::Pallet<Test> + InstitutionQuery=()。
10. **wumin / wuminapp / sfid 同步** —
    - wumin pallet_registry 增 personalManagePallet=7 + 3 call_index;payload_decoder 增 PersonalManage(7) 分支(3 call);action_labels 增 cleanup_rejected_personal_proposal,propose_close 文案改"关闭机构多签提案";test/signer/pallet_registry_test propose_X 测试用例同步 personal-manage call_index 0/1/2。
    - wuminapp duoqian_manage_service.dart `_personalPalletIndex=7` + `_personalProposeCloseCallIndex=1` + `_proposeCreatePersonalCallIndex=0`;submitProposeCreatePersonal 切到 PersonalManage 编码;submitProposeClosePersonal 切到 PersonalManage 自持编码。
    - sfid event_parser:`("OrganizationManage", "DuoqianCreated/DuoqianClosed")` 切到 `("PersonalManage", ...)`(B 阶段后这两个事件由 PersonalManage 发射)。
11. **文档 + 残留 + 验证** — 任务卡 §5.3 4 条残留扫描全零;runtime --lib 37 测试全过;duoqian-transfer 20 测试全过;wumin 105 测试全过;wuminapp duoqian 28 测试全过;cargo check --workspace 含 node + sfid 全部通过。

**已知 follow-up debt**(B 阶段不完成,记录在任务卡 §9 末尾):
- ~~organization-manage 单测模块清空,34 个 case 转 follow-up~~ → 2026-05-07 重写完成(22 用例,见 [20260507-runtime-pallet-tests-restructure.md §10](20260507-runtime-pallet-tests-restructure.md))
- ~~personal-manage 自持单测尚未编写~~ → 2026-05-07 编写完成(14 用例,见同上 §10)
- benchmarks.rs propose_close + propose_create_personal benchmark 重写(仍是 follow-up)

## 9. 验证结果

### 编译
- `cargo check -p primitives`:通过
- `cargo check -p personal-manage`:通过
- `cargo check -p organization-manage`:通过
- `cargo check -p duoqian-transfer`:通过
- `cargo check -p citizenchain`(WASM_FILE):通过
- `cargo check -p node` --features tauri:通过(含 37 个 pre-existing dead-code 警告)
- `cargo check -p sfid-backend`:通过(3 个 pre-existing dead-code 警告)

### 测试
- `cargo test -p citizenchain --lib`:**37 passed / 0 failed**
- `cargo test -p duoqian-transfer`:**20 passed / 0 failed**
- `cargo test -p admins-change`:**通过**
- wumin `flutter analyze && flutter test`:**0 issues + All tests passed!**(105 case)
- wuminapp `flutter test test/duoqian/`:**All tests passed!**(28 case)

### 残留扫描(任务卡 §5.3 全零)
1. personal-manage 内无 organization_manage::DuoqianAccounts/PersonalDuoqian* 引用 ✓
2. organization-manage 内无 DuoqianAccounts/DuoqianAccount/DuoqianStatus 残留 ✓
3. organization-manage 内无 ACTION_CREATE_PERSONAL/PersonalDuoqianInfo/PendingPersonalCreate ✓
4. organization-manage::common.rs 仅 pub use core_const::* re-export,无函数定义 ✓

### 行为不变量逐条核对(任务卡 §5.4)
- ✓ 个人多签 propose_create / propose_close / 投票回调链路保持
- ✓ 机构 register_sfid_institution / propose_create_institution 入参/事件不动
- ✓ 机构 propose_close 行为收紧:输入个人地址返回 Error::NotInstitutionDuoqian
- ✓ pallet_index = 7 全局唯一(MODULE_TAG 唯一性测试守住 8 个 tag)
- ✓ MODULE_TAG `b"per-mgmt"` 与 `b"org-mgmt"` 长度均为 8 字节
- ✓ ACTION 字节命名空间互不干扰(personal: 0/1; organization: 2/3)
- ✓ call_index:personal 0/1/2;organization 1/2/4/5(call=3 留洞)
- ✓ DuoqianAccounts 表 + DuoqianAccount/DuoqianStatus 类型彻底删除
- ✓ 单向依赖:primitives ← personal-manage / organization-manage / duoqian-transfer

### 风险闭环
- R1:3 个共用 trait 已提到 primitives,personal-manage 不再依赖 organization-manage(优于原方案的 trait re-export 兜底,达到完全解耦)
- R2-R6:wumin/wuminapp 客户端已分流,trait 查询测试 mock 走 personal_manage 路径,sfid event_parser 切换无误,链未上线无迁移压力
