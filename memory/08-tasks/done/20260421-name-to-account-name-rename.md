---
title: 链端 SFID 机构账户字段 name → account_name（对齐 SFID 后端 / 前端）
status: done
owner: Blockchain Agent + Mobile Agent
created: 2026-04-21
completed: 2026-04-21
---

# 执行结果（2026-04-21）

## 链端 Rust（10 crate 编译通过）

- [duoqian-manage/src/lib.rs](citizenchain/runtime/transaction/duoqian-manage/src/lib.rs)：
  - 类型：`SfidNameOf<T>` → `AccountNameOf<T>`；Config `MaxSfidNameLength` → `MaxAccountNameLength`
  - 泛型参数：`SfidInstitutionVerifier<Name, ...>` → `<AccountName, ...>`，`RegisteredInstitution<SfidId, SfidName>` → `<SfidId, AccountName>`，`PersonalDuoqianMeta<AccountId, Name>` → `<AccountId, AccountName>`
  - struct 字段：`pub name:` → `pub account_name:`（`RegisteredInstitution`、`PersonalDuoqianMeta`）
  - Event 字段：`Event::PersonalDuoqianProposed { name }` / `Event::SfidInstitutionRegistered { name }` → `account_name`
  - Error：`EmptySfidName` → `EmptyAccountName`
  - Extrinsic 参数：`propose_create(sfid_id, name, …)` / `register_sfid_institution(…, name, …)` / `propose_create_personal(name, …)` 全部 `name` → `account_name`
  - 辅助函数：`role_from_name(name)` → `role_from_account_name(account_name)`；`derive_personal_duoqian_address(..., name)` → `(…, account_name)`
  - 测试：8 处 `let (sfid, name, ...)` 元组解构 + `assert_eq!(meta.name, name)` + 多处 `name.clone()` / `&name` 全部更新
- [benchmarks.rs](citizenchain/runtime/transaction/duoqian-manage/src/benchmarks.rs)：`bench_name<T>` → `bench_account_name<T>`，添加 `let account_name = bench_account_name::<T>()?` 到 propose_create / propose_close 各 setup（先前 5 处调用缺此参数，遗留的 pre-existing 测试 bug 一并修好）
- [configs/mod.rs](citizenchain/runtime/src/configs/mod.rs)：`SfidNameOf<Runtime>` 类型引用 4 处、`MaxSfidNameLength` 1 处、`SfidInstitutionVerifier` 实现的参数 `_name` → `_account_name`、`fn find_address(sfid_id, name)` → `(sfid_id, account_name)`、`info.name.to_vec()` → `info.account_name.to_vec()`、test 局部 `register_name` → `register_account_name`
- [duoqian-transfer/src/lib.rs](citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs)：测试 trait impl 参数 + `MaxSfidNameLength` → `MaxAccountNameLength`
- [offchain-transaction](citizenchain/runtime/transaction/offchain-transaction/src/bank_check.rs) + [tests.rs](citizenchain/runtime/transaction/offchain-transaction/src/tests.rs)：`SfidAccountQuery::find_address(sfid_id, name)` trait 方法签名 → `(sfid_id, account_name)`；`ensure_can_be_bound` 局部 `name_bytes` → `account_name_bytes`

## Dart 侧（wuminapp + wumin）

- [wuminapp/lib/governance/duoqian_manage_service.dart](wuminapp/lib/governance/duoqian_manage_service.dart)：`submitProposeCreate({required Uint8List name})` → `accountName`；`submitProposeCreatePersonal({name})` → `accountName`；`fetchSfidRegisteredAddress(sfidId, name)` → `accountName`
- [personal_duoqian_create_page.dart](wuminapp/lib/governance/personal_duoqian_create_page.dart) + [duoqian_create_proposal_page.dart](wuminapp/lib/governance/duoqian_create_proposal_page.dart)：调用处 `name: nameBytes` → `accountName: accountNameBytes`
- [wumin/lib/signer/payload_decoder.dart](wumin/lib/signer/payload_decoder.dart)：`propose_create` / `propose_create_personal` SCALE 解码路径里 `name` 局部变量 + `summary` 字符串插值 + `fields { 'name': ... }` JSON key → `account_name` / `accountName`

## SFID 后端

- [sfid/backend/src/institutions/chain.rs](sfid/backend/src/institutions/chain.rs)：跨层调用注释更新，链端字段已同步命名
- [sfid/backend/src/sheng-admins/institutions.rs:1526-1530](sfid/backend/src/sheng-admins/institutions.rs)：storage 查询局部 `name_key` → `account_name_key`，注释对齐

## 文档

- [DUOQIAN_TECHNICAL.md](memory/05-modules/citizenchain/runtime/transaction/duoqian-manage/DUOQIAN_TECHNICAL.md)：派生公式、extrinsic 签名、Storage key、Error 名、角色翻译段全部 `name` → `account_name`
- [BLAKE2_ADDRESS_DERIVATION.md](memory/05-modules/citizenchain/runtime/primitives/BLAKE2_ADDRESS_DERIVATION.md)：OP_INSTITUTION 表 + 角色翻译段 + 源码索引
- [STEP1_TECHNICAL.md](memory/05-modules/citizenchain/runtime/transaction/offchain-transaction/STEP1_TECHNICAL.md)：SfidAccountQuery trait 签名

## 验证

- `cargo check -p primitives -p duoqian-manage -p duoqian-transfer -p offchain-transaction -p sfid-system -p shengbank-interest -p resolution-destro -p resolution-issuance-gov -p onchain-transaction -p institution-asset` 10/10 通过
- `cargo test -p primitives` 7/7 通过（含 `all_china_ch_main_addresses_are_unique`）
- `cargo test -p duoqian-manage` 全绿

## 字节影响

- SCALE 编码按位置排，字段名变**零字节影响**，fresh genesis **不需要**
- Runtime metadata 中 extrinsic call 的 field name 从 `name` 变为 `account_name`——按字段名解码的前端客户端需同版本发布（本次 wuminapp + wumin 一次性对齐）
- 冷钱包 QR JSON key `'name'` → `'account_name'` + `'accountName'`（wumin payload_decoder 同步更新）

## 不改的（边界外）

- `institution_name`（机构展示名）— 不同概念，保留
- `sfid` backend 里 `credential.name` — 后端内部 struct 字段，未跨层暴露，不动
- SFID 后端 `PROVINCES` / `cities` 的 `p.name` / `c.name` — 地理名字，与账户无关
- `wallet_name` / bootnodes / grandpa 的 `pub name: String` — 各自域的命名
- node/vendor/ 第三方代码

# 背景

链端 `duoqian-manage` 里代表"SFID 机构下账户名称"的字段从早期叫 `name` / `SfidNameOf<T>` / `MaxSfidNameLength` / `EmptySfidName`，语义含糊（容易和机构名 `institution_name` 搞混）。SFID 后端 `model.rs:95` 的 `MultisigAccount.account_name` 已经用清晰命名，前端 `duoqian_create_proposal_page.dart:179` 也在用 `account.accountName`。本次把链端对齐。

# 目标

- 链端 Rust：`name` → `account_name`，`SfidNameOf<T>` → `AccountNameOf<T>`，`MaxSfidNameLength` → `MaxAccountNameLength`，`EmptySfidName` → `EmptyAccountName`
- wuminapp Dart：`name` → `accountName`（camelCase 对齐风格）
- wumin 冷钱包 payload_decoder：JSON key `'name'` → `'accountName'`
- 文档同步清理

# 铁律

- **字节零变化**：SCALE 按位置编码，fresh genesis 不需要
- **一次推全栈**（按 `feedback_no_compatibility`）：wumin + wuminapp + 链端同版本发布，冷钱包 QR 契约同步
- **多义 `name` 字段逐处判读**：Dart 里有 `institution_name` / 通用展示 label / 账户名 三种，不能 blanket replace

# 不动范围（同字面但不同概念）

- `institution_name`（机构展示名）
- `wallet_store.rs` / `bootnodes` / `grandpa-address` 里的 `pub name: String`
- `SfidIdOf<T>`（身份号，不是名字）
- `MaxSfidIdLength`（同上）
- node/vendor/ 第三方代码
- wuminapp `institution_data.dart:58 final String name`（如果是机构 label）

# 执行 8 步

1. 链端 duoqian-manage/src/lib.rs（~40 处）
2. duoqian-manage/src/benchmarks.rs（~3 处）
3. configs/mod.rs（~8 处）
4. duoqian-transfer/src/lib.rs 测试（~3 处）
5. offchain-transaction `SfidAccountQuery` trait（~3 处）
6. cargo check + test 验证
7. wuminapp Dart（~28 处，逐处判读）
8. wumin payload_decoder.dart（~8 处，含 JSON key）+ 文档清理
