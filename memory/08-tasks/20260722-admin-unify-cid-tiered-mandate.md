# 任务卡：管理员统一结构 + 分层强制 + 授权切公民 CID（第2步）

★ 方案已确认（2026-07-22，用户逐条确认）。**Phase 1 全部 5 步已落地并全绿验证**：Step1 `Admin` 加 `cid_number`(可空)、Step2 删 `PublicAdmin` 全并入 `Admin`、Step3 `required_admin_elements`、Step4 `AdminPolicyApi` 只读节点守卫锁「个人多签禁强制 CID」、Step5 CitizenApp Dart 五端镜像(46 项 admin 测试全绿)。**Phase 2「字段强制=ChainPhase 期段门控」已落地并全绿验证**(A 方案,2026-07-23)：`required_admin_elements` 翻真值表 + 新增 `ChainPhaseCheck` 相位 trait,强制一律 `if is_operation()`,创世期放行/运行期强制,一次 `SwitchToProduction` 升级即全域启用,无专属迁移。**Phase 3「授权切公民 CID」的 Step 3a(提案/操作/费用门)已落地并全绿验证**(A 方案期段门控,2026-07-23)：新增 `InstitutionAdminQuery::resolve_admin_account`(caller 钱包→名册规范账户;运行期有 CID 按 `matches_citizen_account` 绑定解析、否则 account_id),`is_active_assignment`/`is_authorized`/费用路由改走 resolve;`is_institution_admin` **保持 account_id 语义**(枚举/投票快照用,不可混用)。**Step 3b「投票门」已落地并全绿验证**(A 方案全量,2026-07-23)：`InternalAdminProvider` 加默认方法 `resolve_institution_voter`(免改 11 个 mock)+ 运行时 override 复用 3a resolver;核心 `resolve_subject_voter`/`is_subject_voter_in_snapshot` 解析投票人(资格层 4 子 pallet 一次全覆盖);internal/joint/legislation/election 四处投票票据 `voter_account_id` 改规范账户(闭合换绑双投窗口)。**「私权 LR 名册 cid」缺口亦已闭合**(方案乙读侧回落,2026-07-23,见 §9)。**本任务卡全部完成、全绿验证。** 前置：另一线程「钱包账户命名向 Substrate 官方统一（`account_id: AccountId`）+ 机构账户重构」runtime 部分已完成、已静默 → 本卡基于当前状态执行，账户字段一律用 `account_id`。

## 背景
第2步身份审计结论：公民=CID ✅、机构=CID ✅、岗位挂机构 CID ✅，唯一缺口 = **管理员授权身份仍是钱包账户，不是管理员公民 CID**。本卡关闭该缺口。

## 目标不变量
管理员 = **公民 CID（身份/授权）+ 钱包 `account_id`（签名）**。extrinsic 由钱包签名 → 链上解析 `account_id → 公民 CID`（citizen-identity 1:1 绑定闭环）→ 按管理员公民 CID 判名册成员 + 岗位任职授权。个人多签（PMUL，无 CID）永久按 `account_id` 豁免。

## 决策（用户 2026-07-22 确认）
1. 一 CID **同时只绑一个钱包**（1:1，换绑=先解绑再绑）；授权时钱包→CID 解析确定性。
2. 合并 `Admin`/`PublicAdmin`/个人多签到 **admin-primitives 单一结构**，按(类型, 岗位)分哪些必填。
3. **LR（法定代表人）= 全机构统一强制岗**，就是 LR 岗。
4. 分层强制（后期强制，现在预留）：
   - **公权机构**：所有管理员四要素（`account_id + cid_number + family_name + given_name`）完整。
   - **私权机构**：仅任职 **LR 岗** 的管理员四要素完整；其他岗**仅 `account_id`**（cid/姓/名可空）。
   - **个人多签**：仅 `account_id`；**禁止**强制 cid/姓/名（能填、不可强制），且此豁免由**节点守卫**锁死，防 runtime 升级篡改。
5. 分期：现在字段预留（可空）+ 节点守卫；后期 runtime 升级翻开强制 + 授权切 CID。

## 所属模块
- `runtime/admins/admin-primitives`：统一 `Admin` + `required_admin_elements`。
- `runtime/admins/{public,private,personal}-admins`：共用统一 `Admin`（删 `PublicAdmin`）。
- `runtime/entity/entity-primitives/institution_role.rs`：`InstitutionAdminAssignment` 授权键、`InstitutionRoleQuery` 入参（Phase 2）。
- `runtime/entity/{public,private}-manage/institution/role.rs`：`is_authorized`/`is_active_assignment` 切 CID（Phase 2）。
- `node/src/core/node_guard/runtime_policy.rs`：个人多签「禁强制」探针（Phase 1）。
- `runtime/genesis/src/institution/seeder.rs`：创世 4 字段。
- 五端 SCALE 镜像：onchina + CitizenWallet + CitizenApp。

## 设计
### 1. 统一 Admin（admin-primitives，删 PublicAdmin）
`Admin<AccountId> = { account_id, cid_number: AdminCidNumber, family_name, given_name }`，四字段 SCALE 层全可空；三 pallet 共用；`AdminAccountKind{PublicInstitution, PrivateInstitution, PersonalMultisig}` 保留作分层 dispatch 键。

### 2. 分层强制单源
```
required_admin_elements(kind, is_lr_role) -> { cid, family, given }  // account_id 恒必填
  PublicInstitution           => 全 true
  PrivateInstitution & LR岗    => 全 true
  PrivateInstitution & 非LR岗  => 全 false
  PersonalMultisig            => 全 false  【死规则,永不可翻真】
```
Phase 1 恒返回全 false（零行为变更）；Phase 2 翻真值。LR role_code 单源识别（对齐 private-manage `legal_representative` 机制）。

### 3. 节点守卫锁死「个人多签禁强制 CID」（防 runtime 升级篡改）—— 已落地(修订)
**原「活跃 propose_create 探针」方案已否决**：propose_create 被 BaseCallFilter 放行(活跃),且成功路径要真建投票引擎提案 + `reserve`,在候选 WASM overlay 里播种极难,漏一处即把**任何**候选判 `KnownBad` → 硬阻断**所有**链升级,风险不可接受(比 H1 危险,H1 休眠)。
**改为只读 runtime API 守卫**(已实现):新增 `primitives::admin_policy::AdminPolicyApi::personal_multisig_cid_mandated() -> bool`(实现 = `admin_primitives::required_admin_elements(PersonalMultisig,false).cid`,恒 false);`check_candidate_runtime` 调 `"AdminPolicyApi_personal_multisig_cid_mandated"`,候选返 `true`(强制)或**缺该 API**(`call_candidate?` 传播 Err)→ 判 `KnownBad`(fail-closed)。纯读、不执行 propose_create、零播种、零阻断升级风险。**范围收敛**:节点守卫只锁「CID 不强制」;姓名保持 `normalize_names` 自动填(空姓→「管理」、空名→「员」),不进节点守卫。链端锁:`required_admin_elements(PersonalMultisig)=全false` 死规则测试 + `propose_create_does_not_mandate_personal_multisig_cid`(空 cid/姓名 propose_create 成功)。

### 4. 授权切公民 CID（Phase 2，仅机构管理员）
`is_authorized(caller_account_id, RoleSubject{机构CID, role}, …)` → `cid_of_wallet(caller)`（citizen-identity 1:1 闭环，校验 `WalletAccountByCid[cid]==caller` + CID Active）→ 按**管理员公民 CID** 判名册成员 + 任职。`InstitutionAdminAssignment` 授权键改 `admin_cid_number`，`account_id` 降为签名快照；`InstitutionRoleQuery` 三方法入参改 CID。个人多签 `AuthorizationSubject::PersonalMultisig(account_id)` 不变。

### 5. Dart 五端镜像（as-built，Phase 1 已落地）
单源 per-admin 解码 = `institution_role_storage_codec.dart:decodeAdminVector`（布局恒 `account_id ‖ Compact(cid) ‖ Compact(family) ‖ Compact(given)`，`includeCitizenCid=true`、`allowEmptyNames=true`；删 `isPublic` 分支——公权/私权/个人多签同一解码）。上层 `admin_account_storage_codec.dart` / `admins-change/codec/admin_account_codec.dart` 去 `isPublic` 参数转调。编码侧对齐：`personal-manage/personal_manage_service.dart`（propose_create）与 `admins-change/codec/admin_set_change_call_codec.dart`（个人多签换人载荷）每个 admin 恒写 `Compact(cid)`（个人多签空 CID=`Compact(0)`=单字节 `0`）。测试夹具全部按恒带 cid 更新（金标 hex 已含 `00`）。**范围收敛**：仅统一「恒带 cid + 允许空姓名」，授权仍按 `account_id`（Phase 2 才切 CID）。

### 6. Phase 2 字段强制=ChainPhase 期段门控（as-built，2026-07-23 已落地）
**设计哲学（用户 A 方案）**：创世期/运行期就是把「该强制的字段」用一次 runtime 升级从可空切强制的开关。真值表恒真、门控在调用侧;`Phase=Genesis`(当前)恒放行=零行为变更;`SwitchToProduction` 翻 `Operation` 即全域强制生效,无专属迁移。
- **`admin-primitives`**：`required_admin_elements(kind,is_lr)` 翻真值表（公权全 true / 私权 LR 岗全 true / 私权非 LR·个人多签全 false，个人多签 cid 永 false 死规则）；新增 trait `ChainPhaseCheck{is_operation()->bool}`（定义于此而非 genesis-pallet——后者反向依赖各 admin/entity pallet，定义于 genesis-pallet 会成环）；`Admin::satisfies(req)` 校验原始字段（须在 normalize 前）；`impl ChainPhaseCheck for ()` 恒 Genesis 供无需切换的测试 mock。
- **`genesis-pallet`**：`impl admin_primitives::ChainPhaseCheck for Pallet`（读 `Phase==Operation`），仿 `DeveloperUpgradeCheck` 范式。
- **`public-admins`**：Config 加 `type ChainPhase`；`validate_admin_set` 加 `if is_operation(){四要素完整}`（本 pallet 不 normalize，直验原始值）；新 error `IncompleteAdminFields`。
- **`private-manage`**：Config 加 `type ChainPhase`；governance.rs LR `Set` 既有 `EmptyLegalRepresentative*` 强制包 `if is_operation()` 并单源自 `required_admin_elements(PrivateInstitution,true)`。
- **`runtime/src/configs.rs`**：public_admins/private_manage 两处 `type ChainPhase = GenesisPallet`。
- **未改动**：private-admins/personal-admins（真值表全 false，normalize 照旧填默认）；节点守卫 `AdminPolicyApi`（真值表下个人多签 cid 仍 false，死规则不变）。
- **验证**：admin-primitives 6 / public-admins 11 / private-manage 21 / public-manage 11 / genesis-pallet 18 全绿，`cargo check --workspace --tests` 0 错。**范围收敛**：仅字段强制门控，授权切 CID 未做。

### 7. Phase 3 授权切 CID · Step 3a 提案/操作/费用门（as-built，2026-07-23 已落地）
**设计**：不改任何存储结构，只在鉴权入口把 caller 钱包解析为名册规范账户（canonical `account_id`=快照键），下游 account_id 匹配逻辑原样不动。期段门控同 Phase 2 一个开关。
- **新增 `admin_primitives::InstitutionAdminQuery::resolve_admin_account(code,cid,caller)->Option<AccountId>`**：运行期(Operation)+ 该 admin 有 CID → 只按 `matches_citizen_account(cid,caller)` 解析（**无 account_id 回退**，旧钱包换绑即掉权、新钱包解析到同一 canonical=换绑不掉权）；创世期 / 无 CID admin → 按 `account_id`。`() 实现` 返回 None。四处实现：`()`、public-admins、private-admins、`RuntimeInstitutionAdminQuery`（按 code 分派，非法人 public.or_else(private)）。
- **`is_institution_admin` 保持 account_id 语义**（枚举/投票快照 `active_accounts_for_role`/`RuntimeInternalAdminProvider` 用；曾误改为 resolve 版会把换绑陈旧账户从快照剔除，已回退）。
- **调用者门切 resolve**：`is_active_assignment`（public+private role.rs：resolve→canonical 后匹配 `assignment.account_id`）、`is_authorized`（删冗余 is_institution_admin，靠 is_active_assignment）、**费用路由 `is_authorized_institution_actor`**（configs.rs，否则换绑管理员 extrinsic 卡费用路由）。`is_nrc_admin` **不动**（仅门控创世期专属的开发者直升，改动惰性）。
- **依赖注入**：private-admins Config 补 `ChainPhase`+`CitizenIdentityBinding`（public-admins Phase 2 已有）；runtime 用 `GenesisPallet`+`RuntimePublicAdminCitizenIdentityBinding`（身份层通用绑定，命名历史遗留）。
- **验证**：public-manage 19（含换绑用例）/ private-manage 22（含换绑用例）/ citizenchain 集成 48 全绿，`cargo check --workspace --tests` 0 错。换绑金标：运行期新钱包授权成功、旧钱包掉权、无 CID admin 仍按 account_id。
### 8. Phase 3 · Step 3b 投票门（as-built，2026-07-23 已落地）
**两层都按账户、都必须改**（只改资格不改票据 → 换绑后新钱包=第二张票=**双投**）：
- **资格层（一改覆盖 4 子 pallet）**：`InternalAdminProvider` 加**默认方法** `resolve_institution_voter(cid, caller) -> Option<AccountId>`（默认 `Some(caller)`=账户语义 → **11 个测试 mock 全部免改**）；运行时 `RuntimeInternalAdminProvider` override → `cid_institution_code` → 复用 3a 的 `RuntimeInstitutionAdminQuery::resolve_admin_account`（同一 `ChainPhase` 门控）。核心 `votingengine/src/snapshot.rs` 新增 pub `resolve_subject_voter`（Institution→provider 解析；PersonalMultisig→恒按账户），`is_subject_voter_in_snapshot` 内部解析后匹配快照，`None`→拒（运行期换绑旧钱包即 None=掉权）。
- **票据层（防双投，4 处）**：`internal-vote/vote.rs`、`joint-vote/jointinternal.rs`、`legislation-vote/representative/mod.rs`、`election-vote/lib.rs` 的 `InstitutionVoteTicket.voter_account_id` 由 `who` 改为 `resolve_subject_voter(&subject,&who).unwrap_or(who)` → 换绑前后归并同一张票。个人多签票据/`AdminSnapshot` **不动**（无 CID 身份，账户即身份）。
- **死规则**：资格与票据必须解析**同一个原始 `who`**，**绝不可二次解析规范账户**（运行期 `resolve(旧规范账户)=None` 会误杀合法投票人）。
- **行为变化（用户已确认接受）**：资格改为「按当前解析身份」后，建案后被移出名册者其冻结票不再能投（原为快照冻结仍可投）。
- **验证**：internal-vote 96（含换绑金标：新钱包投票成功且票据记规范账户、旧钱包 `NoPermission`、新钱包再投 `AlreadyVoted`）/ joint-vote 13 / legislation-vote 35 / election-vote 17 / votingengine 17 / citizenchain 集成 48 全绿，`cargo check --workspace --tests` 0 错。

### 9. 私权 LR 名册 cid 缺口 —— 已闭合（as-built，方案乙，2026-07-23）
**缺口**：分层规则「私权仅 LR 岗强制钱包+姓+名+公民CID」的**强制落点是 `InstitutionInfo.legal_representative`（独立身份记录）**，而**名册 `Admin.cid_number` 从不被 private-admins `validate_admin_set` 校验、默认空**——同一个人两份记录；授权/投票解析读的是**名册**字段，故私权 LR 若名册无 cid 仍账户锚定、换绑照样掉权（公权无此问题，强制点就是名册字段）。
**闭合（读侧回落，零新强制、零新数据、零 SCALE 变更）**：
- `entity-primitives::InstitutionLegalRepresentativeQuery` 加**默认方法** `legal_representative_cid(cid) -> Option<Vec<u8>>`（默认 None → 现有实现全免改）；public-manage / private-manage / runtime 路由三处实现（读 `InstitutionInfo.legal_representative.cid_number`）。
- private-admins Config 加 `type LegalRepresentativeQuery`（已依赖 entity-primitives，零新依赖、零成环）；`resolve_admin_account` 改「**有效 CID**」语义：名册 cid 优先 → 名册为空且该 admin 是本机构 LR 则用 LR 身份记录的 cid → 仍无则按 account_id。运行期才查 LR（创世期恒 None）。
- **授权与投票一次同闭合**：投票解析最终也走 `resolve_admin_account`。
- **验证**：private-admins 6 / private-manage 23（含新金标 `operation_phase_lr_without_roster_cid_falls_back_to_identity_record`：LR 名册无 cid、仅身份记录有 cid → 运行期换绑后新钱包授权成功、LR 旧钱包掉权、非 LR 无 cid 管理员仍按 account_id）/ public-manage 19 / citizenchain 集成 48 全绿，`cargo check --workspace --tests` 0 错。
- 未选 (丙)（LR 设置路径反写名册 cid）：跨层写入 + 双源同步，风险与复杂度高于读侧回落。

## 分期
**Phase 1（已落地，零行为变更）**：合并统一 `Admin`（删 PublicAdmin）+ 节点守卫 `AdminPolicyApi`（只读 fail-closed）+ 五端 SCALE 镜像统一 Admin 解码/编码 + 逐字节 guard。
**Phase 2（已落地，2026-07-23，零行为变更）**：`required_admin_elements` 翻真值表 + `ChainPhaseCheck` 期段门控（创世放行/运行期强制）。当前 `Phase=Genesis` 恒放行；将来 `SwitchToProduction` 翻 `Operation` 即启用强制,无专属迁移。
**唯一剩项（未排期）**：授权切公民 CID——`is_authorized` 钱包→CID 解析、`InstitutionAdminAssignment` 授权键改 `admin_cid_number`。开发期重新创世无 migration；已上线走 setCode + StorageVersion migration。

## 前置依赖
- `cargo check --workspace` 基线绿（另一线程刚收尾，先验证）。
- 账户字段命名 `account_id`（那个线程标准），合并时保持。
- **LR role_code 单源**须确认（`legal_representative` 是字段还是 role 任职 → 影响 `is_lr_role`）。
- 个人多签创建路径 + `BaseCallFilter` 状态（节点守卫探针走真实 call）。

## 验收标准
- 统一 `Admin` 五端 SCALE 逐字节对齐；`PublicAdmin` 全删无残留。
- `required_admin_elements` 单源；Phase 1 恒 false（零行为变更）。
- 节点守卫：候选 runtime 若对个人多签强制姓/名/CID → 判 `KnownBad`（行为锁单测覆盖）。
- Phase 2：公权/私权LR岗四要素强制、私权其他岗/个人多签豁免；授权按公民 CID（换绑钱包仍授权）；个人多签仍按 `account_id`。
- `cargo check --workspace` / offchain·node·entity 全绿；`dart analyze` / 金标通过。
