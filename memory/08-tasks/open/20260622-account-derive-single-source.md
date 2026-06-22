# 账户派生统一为唯一真源（OP_XX + 保留名 + 路由 + 字段 schema 收敛一处）

## 状态

**设计锁定，执行未开始（2026-06-22）。** 取代原 `20260622-derive-domain-rename-gmb-op-name.md`（改名并入本卡 Tier 3）。Tier 1/2 行为中性（地址不变，本不需创世）；Tier 3 域字节变更随 `20260622-cid-classification-unify-t3t4` 末尾**一起创世**。

## 任务需求

账户派生**必须统一为唯一真源**：把派生的 `OP_XX` 常量、保留名、路由（name→op_tag→payload）、每种 payload 字段 schema 全部收敛到**一处**，各端一律调它，消除当前 4 处重复 + 已存在的漂移。CID 号不在范围内（已有唯一真源 `citizencode/backend/number/`）。

### 用户决策记录（2026-06-22）
- 「统一字段」= **统一 OP_XX 且只有一处真源**（纯收敛重构，**不改** payload schema 本身）。
- 三种 payload schema 保持原状（机构按 cid_number / 自定义追加 name / 个人多签按 creator+name）——异构是账户种类本质，不抹平。
- Dart 跨语言防漂移 = **金标向量 fixture**（不走 codegen）。
- 改名 `DUOQIAN→GMB`、`OP_INSTITUTION→OP_NAME` 并入本卡 Tier 3；字节值变更随 T3/T4 末尾创世。
- 创世留到最后统一做，本卡其余部分先行。

## 现状（真实核对）

底层哈希算法真源 = 链端 `core_const::derive_account(op_tag, ss58, payload)` ✓（所有 Rust 调用方都走它；Dart 跨语言字节对齐重写）。问题全在哈希**之上**那层散落多处：

| 要素 | 散落处（份数） |
|---|---|
| 域 `DUOQIAN` | `core_const.rs:35` + citizenapp `account_derivation.dart:45`（2） |
| op_tag 常量 | `core_const.rs:40-46` + citizenapp dart `kOp*:25-43`（2） |
| 5 保留名 | `core_const.rs:62-75`(`&[u8]`) + 后端 `accounts/derive.rs:45-64`(`&str`) + citizenapp `reserved_account_names.dart` + citizenwallet `lib/chain/reserved_account_names.dart`（4） |
| `isForbidden` 判定 | `core_const.rs:80` + citizenapp + citizenwallet（3，**行为不一致**↓） |
| 路由 name→op_tag→payload | 链 `organization-manage`(lib.rs:653-677 derive + 689- role，3 tag) + 后端 `accounts/derive.rs:78-102`(6 tag) + citizenapp dart `deriveInstitutionAccountIdByName:120-145`(6 tag) + 内联 `china/mod.rs:23-70`、`personal-manage/lib.rs:328-341`（≥4 张表） |

### payload schema（三种，保持不变）
- `OP_MAIN/FEE/STAKE/AN/HE` → `cid_number`
- `OP_INSTITUTION`（自定义）→ `cid_number ‖ account_name`
- `OP_PERSONAL`（个人多签）→ `creator_pubkey(32B) ‖ account_name`
- 公共前缀 `域 ‖ op_tag ‖ ss58_le(2B)` 所有 tag 一致。

### 🔴 已挖到的真实漂移 bug（单源根治）
`isForbidden` 两种行为：
- 链端 `core_const::is_forbidden_account_name` + 冷钱包 citizenwallet：判 **3 名**（质押/安全/两和），**不 trim**。
- 热钱包 citizenapp `isForbiddenAccountName`：判 **5 名**（含主/费）且 **`name.trim()`**。
- 后果：`"  主账户  "` 链端当合法自定义名（字节不 trim，后端测试 `derive_account("cid","   ").is_some()` 已证），citizenapp 却 trim 后判禁止 → 两端结论相反 + 混淆「强制(主/费)」与「禁止(质押/安全/两和)」语义。

## 方案

### Tier 1 — Rust 单源（行为中性，地址不变）
新建 `citizenchain/runtime/primitives/src/account_derive.rs`（或 core_const 子模块），集中：op_tag 常量（从 core_const 迁入或 re-export）+ 5 保留名 + `is_forbidden_account_name` + 路由 + 每种 payload 字段拼装 + 高层 helper：
- `derive_main(cid)` / `derive_fee(cid)` / `derive_by_name(cid, name)`（全 6 tag 路由）/ `derive_personal(creator, name)`。
`core_const` 只留底层 `derive_account` 原语 + SS58。改造调用方全部改调新模块：
- `organization-manage`：`derive_institution_account`/`role_from_account_name` 改成 helper 的薄适配（保留 Role 枚举 + DispatchError 包装，删内部 payload 重拼）。
- 后端 `accounts/derive.rs`：**删** `&str` 重复保留名 + 路由，改调新模块（&str↔&[u8] 转换）。
- `china/mod.rs`、`personal-manage/lib.rs`：内联 `derive_account(OP_X,..)` 改调 helper。
→ Rust 路由/常量收敛成**唯一一处**。

### Tier 2 — 跨语言金标对齐
- citizenapp 保留**唯一** Dart 派生镜像（`account_derivation.dart`）；citizenwallet 只共享保留名。
- 新增 **Rust 导出金标向量**：canonical `(kind, cid/creator, name) → address` fixture（JSON）；Dart 测试加载断言逐字节一致，CI 防漂移。
- 修 citizenapp `isForbidden` 漂移 → 改回 3 名 + 不 trim（对齐链端）。

### Tier 3 — 改名并入（随 T3/T4 末尾创世）
- `DUOQIAN→GMB`（`b"GMB"`，`&[u8;7]`→`&[u8;3]`）：地址派生 + 全部 sr25519 签名 payload 域（`configs/mod.rs` OP_SIGN_* + 后端 `chain_runtime.rs` + 热钱包 `_domain`）锁步改；china_*.rs 创世地址重算。
- `OP_INSTITUTION 0x06`→`OP_NAME 0x06`（值不变）。
- 详见下「改名爆炸半径」。

## 实施顺序
1. **Tier 1 Rust 单源**（不依赖创世）：建 `account_derive` 模块 + 迁移 + 全调用方改调 + 删后端重复；TDD，`cargo check`/`cargo test`（organization-manage / personal-manage / primitives / 后端 derive）零行为变化。
2. **Tier 2**：Rust 导出金标 fixture + citizenapp 加载断言 + 修 isForbidden 漂移；`flutter analyze` + dart test。
3. **Tier 3 改名**（与 T3/T4 同批合入）：见爆炸半径；冷钱包 citizenwallet 签名链路审计。
4. **末尾统一创世**（并入 T3/T4 Phase 3）：china_*.rs 派生地址随新域 + 新机构码重烤；重跑公权机构数据包生成器（见 `feedback_registry_regen_after_genesis`）。
5. 验证：链端全测过；后端 `chain_runtime` golden 重算对齐；端到端扫码签名 + 机构账户地址↔链上回执一致；全仓旧域/旧 op 名/重复保留名残留=0。

## 关键文件:行

**算法真源 + op_tag/保留名定义**
- `citizenchain/runtime/primitives/src/core_const.rs:35`(域)/`:40-46`(op_tag)/`:62-82`(保留名+is_forbidden)/`:89-97`(derive_account)

**Rust 路由/拼装（待收敛）**
- `citizenchain/runtime/governance/organization-manage/src/lib.rs:653-677`(derive_institution_account)/`:689-`(role_from_account_name)；`address.rs`(Role 枚举)
- `citizenchain/runtime/governance/personal-manage/src/lib.rs:328-341`(derive_personal_account)
- `citizenchain/runtime/primitives/china/mod.rs:15-70`(创世 main/fee 内联)
- `citizencode/backend/accounts/derive.rs:45-102`(&str 保留名 + 路由，删)

**Rust 调用方（改调新源）**
- 链:organization-manage `institution/register.rs:93-94`、`institution/accounts.rs:88-93`、`close.rs:105`、`benchmarks.rs:28`、tests/cases.rs
- 后端:`admins/actions.rs:909`、`accounts/handler.rs:86`、`subjects/service.rs:335,344`、`citizenapp/public_institution.rs:400`

**Dart 镜像 + 金标**
- citizenapp `lib/governance/shared/account_derivation.dart`（唯一镜像）+ `reserved_account_names.dart`（修 isForbidden）+ 调用方 `citizen/public/public_institution_detail_page.dart`、`citizen/public/data/public_institution_accounts.dart`、`governance/personal-manage/personal_account_create_page.dart`
- citizenwallet `lib/chain/reserved_account_names.dart`（只共享保留名）

**改名爆炸半径（Tier 3，字节值变 = 创世）**
- 定义 `core_const.rs:35`；签名调用 `configs/mod.rs:798,855,952,1020,1099` + tests `mod.rs:95,139,173`/`cases.rs:889`；后端签名 `core/chain_runtime.rs:7,147,191,249,296,692`；热钱包 `account_derivation.dart:45` `_domain`；创世地址 `china/{cb,ch,zb}.rs`；OP_INSTITUTION→OP_NAME：`core_const.rs:46-47`、`organization-manage/{lib.rs:647,668,address.rs:7,17}`、后端 `derive.rs`。

## 行为中性 vs 创世
- Tier 1/2 = 纯收敛 + 修漂移，同算法同路由 → **地址不变，本不需创世**。
- 仅 Tier 3 域字节变 → 地址变 → 随 T3/T4 末尾创世。

## 阻塞与协调
- **强绑定 `20260622-cid-classification-unify-t3t4.md`**：Tier 3 + 创世并入其 Phase 3。
- 并行线程 dirty 的 `china_cb/ch/zb.rs` + `gov/service.rs`：创世重烤前等其提交。
- `feedback_no_compatibility`：改即全切不留旧源；`feedback_scale_domain_must_be_array`：域常量保持 `&[u8;N]`。
