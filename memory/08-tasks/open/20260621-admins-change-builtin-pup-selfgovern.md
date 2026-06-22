# 任务卡：链端 — PUP 自治 + 机构注销凭证 close + 根账户硬保护（先沟通）

- 任务编号：20260621-admins-change-builtin-pup-selfgovern
- 状态：open（先沟通条件:涉及 runtime 规则改动;设计已与用户收敛,实施前再确认一次）
- 所属模块：citizenchain/runtime/governance/{admins-change, organization-manage} + citizencode/backend（注销态+凭证）
- 当前负责人：Blockchain Agent（链端）+ CID Agent（CID 注销态+凭证签发）
- 创建时间：2026-06-21（2026-06-21 并入 close 设计）

---

## 甲、PUP 自治阻塞修复

### 问题（已核实）
创世内置机构一律 `kind=BuiltinInstitution`（`admins-change/src/lib.rs:275`）。但 `ensure_account_kind_matches_org`（`lib.rs:567-584`）只准 `BuiltinInstitution` 配 NRC/PRC/PRB，`validate_admins_len_for_account`（`lib.rs:506-533`）对 `BuiltinInstitution` 走 `expected_admins_len(org)`，而该函数对 PUP 返回 None。后果:**联邦注册局(PUP,215 admin)及 SF/JC/JY/LF 发不了 `propose_admin_set_change`**。

### 方案（用户取向 = A，保留 724 兜底语义）
PUP 内置**保持 `BuiltinInstitution`**,放宽两处校验接受 PUP：
- `ensure_account_kind_matches_org`:`BuiltinInstitution` 分支允许 `ORG_NRC|PRC|PRB|PUP`。
- `validate_admins_len_for_account`:`BuiltinInstitution` 分支对 PUP 走可变上限 `>=2 && <= MaxAdminsPerInstitution`，NRC/PRC/PRB 仍走精确 `expected_admins_len`。

---

## 乙、机构注销凭证 close + 根账户硬保护（2026-06-21 用户拍板）

### 真源方向（不循环）
链不查 CID 库。注册局在 CID 设【注册局域注销态】(区别于链投影 `RevokedOnChain`,见 `citizencode/backend/subjects/model.rs:55`) + 签发**注销凭证** → 机构管理员带凭证发 `organization-manage::propose_close` → 链验签(对称于创建凭证 `verify_institution_registration`) → 关闭 → indexer 写投影 `RevokedOnChain`。

### 显式硬保护（用户拍板"封存全部 china/ 创世机构 + 多层",纵深三层)
1. **创世封存全部初始机构**:`admins-change::build()` 在 CB/CH/ZF/SF/JC/JY/LF 各插入循环里,把每个机构主账户(已是 AdminAccounts 键)**同时写进不可变 storage** `ProtectedGenesisAccounts: StorageMap<AccountId, ()>`;创世后无 extrinsic 可改。覆盖:总统府、联邦注册局、安全/情报/特勤/人事局、国储会/省储会/省储行、顶层司法/监察/教育/立法 = CID 系统根基。
2. **close 入口 `ensure_closeable`**（`organization-manage::do_propose_institution_close` 最前,专门错误码,**无条件、注销凭证不可绕过**）：
   - `ProtectedGenesisAccounts` 命中 → `CannotCloseGenesisInstitution`（创世根基,封存表精确匹配,最硬）。
   - `org ∈ {ORG_NRC, ORG_PRC, ORG_PRB}` → `CannotCloseGovernance`（治理机构按 org 多叠一层,不依赖 kind/封存表）。
3. **保留 `do_close_admin_account` 的 `!BuiltinInstitution`（lib.rs:724）** 作最后兜底。
   - 结论:全部创世初始机构(china/)**永不可注销关闭**;行政区生成 + organization-manage 创建出来的机构(市注册局/公安局/公司,InstitutionAccount,不在 china/)**可注销关闭**。个人走 `personal-manage::propose_close`,不受本闸影响。

### 主账户级联（用户拍板:一起关）
关 `Role::Main` = 注销整个机构 → execute 遍历 `InstitutionAccounts(cid_number, *)` 逐个 `close_admin_account`,主+费用+自定义全关,不留孤儿;注销凭证 scope=整机构。关非主账户只关该账户,凭证 scope=该账户。链端校验"凭证 scope ↔ 被关账户 Role"匹配。

### CID 侧（CID Agent）
- 新增"注册局域注销态"(机构级 + 账户级),由注册局管理员在 CID 发起;区别于 chain_status 投影。
- 新增 `AdminActionType::InstitutionDeregister`(整机构)/`InstitutionAccountDeregister`(单账户),`auth_type = AdminOperationAuth::PasskeyChallenge`(最严:passkey + 公民钱包确认 + 一次性 SecurityGrant),与现有 `DeleteFederalRegistry/InstitutionDeleteAccount/CpmsDeleteKeys` 同档(`operation_auth.rs:115-127`)。
- 注销凭证签发器(对称 create 凭证 `verify_institution_registration`),只在 de-register 动作通过后签发;绝不为 `ProtectedGenesisAccounts`/治理机构签发。

---

## 部署
预创世/重新创世阶段:改常量/规则 + 写 `RegistryRootAccount` 后**重生 chainspec**一次性生效,零迁移;若链已固定则 setCode + migration。

## 已拍板（2026-06-21）
- 甲 = 方案 A（PUP 保持 BuiltinInstitution + 放宽两处校验）。
- 乙硬保护 = 封存全部 china/ 创世机构 `ProtectedGenesisAccounts` + org 多层 + 724 兜底。
- 关主账户级联关该机构全部账户。
- 注销由注册局管理员在 CID 发起,走 PasskeyChallenge 最严档(passkey + 冷钱包确认 + 一次性授权)。

## 待确认问题（仅剩 1 项,定了即可开工）
- 链上 close 提案谁提交:(a) 注册局直接提交可强制处置违规机构 / (b) 机构 admin 带凭证提交需机构配合 / (c) 两者皆可。建议 (a) 或 (c)。
