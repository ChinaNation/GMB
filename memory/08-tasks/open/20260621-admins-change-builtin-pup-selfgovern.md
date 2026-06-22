# 任务卡：链端 — PUP 自治 + 机构注销凭证 close + 根账户硬保护（先沟通）

- 任务编号：20260621-admins-change-builtin-pup-selfgovern
- 状态：in_progress（链端 + CID 后端 + 钱包 decoder + CID 前端入口全落地验证;仅剩 node propose_close 调用面适配 + 运行期端到端冒烟 + runtime 级集成测试 3 项 follow-up）
- 所属模块：citizenchain/runtime/governance/{admins-change, organization-manage} + citizencode/backend（注销态+凭证）
- 当前负责人：Blockchain Agent（链端）+ CID Agent（CID 注销态+凭证签发）
- 创建时间：2026-06-21（2026-06-21 并入 close 设计）

---

## 实施进度（2026-06-21）

**已完成并验证 — admins-change（甲 + 乙的封存/检查器底座）**：
- 甲 PUP 自治(方案A):`ensure_account_kind_matches_org`(lib.rs:567)BuiltinInstitution 分支加 `| ORG_PUP`;`validate_admins_len_for_account`(lib.rs:512)BuiltinInstitution 改 `match expected_admins_len(org)`(NRC/PRC/PRB 精确数,PUP 走可变上限 `>=2 && <=Max`)。
- 乙 创世封存:新增 `ProtectedGenesisAccounts` StorageMap(lib.rs:237);`build()` 三处插入循环(CB/CH/insert_pup_builtin 宏)同步写入(lib.rs:326/337/356);新增 `pub fn is_genesis_protected`(lib.rs:521)供 organization-manage 调。
- 724 `!BuiltinInstitution` 兜底保持不变。
- 测试:新增 `genesis_protected_seals_every_builtin_institution`(封存条数==admin账户条数,治理+PUP都封存,非创世不封)+ `pup_builtin_clears_admins_change_validation_for_set_change`(PUP 创世过 admins-change 校验;完整投票流转因单测桩 InternalAdminProvider 仅到投票引擎边界,运行时 provider=admins-change 本身可全通)。**cargo test 43/43、0 warning、fmt 过**。
- 发现:admins-change 单测夹具用桩 `TestInternalAdminProvider`(mod.rs:101),无法端到端跑创世 PUP 投票;**需补一个 runtime 级集成测试**(真实 provider)验证创世 PUP 自治全流程——记为本卡 follow-up。

**已完成并验证 — organization-manage（机构注销 close 统一模型,2026-06-21）**:
- **管理员属于机构**:`resolve_admin_account_for_account`(lib.rs:788)改为任意账户→`Institutions[cid].main_account`,机构管理员统一授权所有账户(顺带修好 `lookup_admin_config` 对非主账户返回机构管理员)。
- 凭证 verifier:`traits.rs` `CidInstitutionVerifier` 加 `verify_institution_deregistration`(+`()` impl);`runtime/src/configs/mod.rs` 加具体实现,payload=`DUOQIAN‖OP_SIGN_DEREGISTER(0x14,新增于 primitives core_const)‖genesis_hash‖scope‖cid_number‖account_name‖target_account‖nonce‖issuer…`(target+scope 入签名防重放)。
- `propose_close`(lib.rs:566)加 `register_nonce/signature/issuer_cid_number/issuer_main_account/signer_pubkey` 参数。
- `do_propose_institution_close`(close.rs):org 解析 → **ensure_closeable 三层硬闸**(`is_genesis_protected` / `org∈{NRC,PRC,PRB}` / admins-change 724 兜底)→ **role 推 scope**(Main=整机构/非主=单账户)→ is_active_account_admin(机构管理员)→ **验注销凭证 + `UsedDeregisterNonce` 防重放**。scope 由 role 推导经签名绑定,故不需要独立 scope 参数/错误(原计划的 `DeregisterScopeMismatch/InvalidDeregisterScope` 实为死码,未落地)。
- 级联:`CloseInstitutionAction` 加 `scope`;`execute_institution_close_with_finalizer` 按 scope 分流——整机构 `InstitutionAccounts::iter_prefix(cid)` 收集全部账户逐个(扣费后,dust 子账户整额免费转)→ 同一 beneficiary,末尾 `close_admin_account` 一次;单账户只关该账户不动 AdminAccount。整体 AllowDeath。
- 存储/错误/常量:`UsedDeregisterNonce`、`SCOPE_INSTITUTION=0/SCOPE_ACCOUNT=1`、`OP_SIGN_DEREGISTER=0x14`、4 个错误(GenesisInstitution/Governance/InvalidDeregisterCredential/DeregisterNonceAlreadyUsed)。
- **测试**:新增 非主账户只删该账户(机构/AdminAccount 保留)、凭证拒签、nonce 重放拒;更新 2 个级联断言(关主=级联两账户 beneficiary 收 1980)。**organization-manage cargo test 29/29、0 warning、fmt 过;全 citizenchain runtime cargo check 通过**。
- 注:propose_close 签名变更属 runtime,预创世重生 chainspec 生效;benchmarks 无 propose_close 实际调用,无需改。

**CID 后端(D)进行中(2026-06-21)**:
- **已完成并验证**:
  - 注销凭证签发器 `chain_runtime::build_institution_deregistration_credential`(citizencode/backend/core/chain_runtime.rs),payload=`DUOQIAN‖OP_SIGN_DEREGISTER(0x14)‖genesis_hash‖scope‖cid_number‖account_name‖target_account‖nonce‖issuer×3`,与链端 `verify_institution_deregistration` **逐字节一致**;抽纯函数 `deregistration_payload_digest` + **golden 测试锁死字节**(任何类型/顺序漂移即红)。
  - 最严档动作 `AdminActionType::InstitutionDeregister/InstitutionAccountDeregister`(operation_auth.rs:enum/as_str/label/parse/`auth_type=PasskeyChallenge`)。
  - **注销态表**(D1):`core/db.rs init_current_schema` 新增 `institution_deregistrations`(cid_number/account_name/scope/target_account/deregister_nonce UNIQUE/signature/status[ISSUED|ONCHAIN_CLOSED]/issued_by/issued_at/closed_at)+ 活跃唯一索引(同账户同时仅一张 ISSUED)。
  - **注销态表**(D1):`core/db.rs init_current_schema` 新增 `institution_deregistrations`(+ issuer×3 列,签发时回填供下发直读)+ `deregister_nonce` UNIQUE + 活跃唯一索引。
  - **handler 接线 + 路由(D 剩余,2026-06-21 完成)**:
    - conn 级机构查询:`main.rs` 抽 `Db::get_institution_with_accounts_conn`(原方法薄包装,零行为变化),让 preview/apply 派发留在 conn 级。
    - `actions.rs`:`validate_institution_deregister_conn`(查存→`get_visible_scope` 管辖→`created_by='SYSTEM'` 拒根基→account 活跃→`derive_account` 派生 target,与链端同源)+ preview/apply 两 match 分支 + `is_governance` 纳入两动作 + **commit 后处理(state 层)建凭证 + 回填 signature/issuer,失败删 ISSUED 行**。
    - repo:`insert_deregistration_issued_conn`/`set_deregistration_credential_conn`/`delete_deregistration_by_nonce_conn`/`get_active_deregistration_by_cid_conn` + `DeregistrationCredentialRow`。
    - 路由:`GET /api/v1/app/institutions/:cid/deregistration-info`(镜像 registration-info,只读 ISSUED 凭证下发)。
  - **验证**:citizencode backend cargo test **64/64 + golden 字节锁**、**0 warning**、fmt 过;`institution_deregistrations` 表 + 索引在开发库**实测落库**(14 列 + 唯一约束齐)。
  - 设计微调:`SCOPE_INSTITUTION/SCOPE_ACCOUNT` 在 CID 侧定义为 0/1(与链端同值);issuer 字段 commit 回填(env 源,与签名同源)。
- **D 完成。** 运行期端到端(真实机构+管理员+passkey+冷签)需带 dev DB 的集成冒烟,留作验收。

**已完成并验证 — CitizenWallet decoder(E)+ CID 前端注销入口(F,2026-06-21)**:
- **E CitizenWallet decoder**:`lib/signer/payload_decoder.dart` 机构 `propose_close`(pallet OrganizationManage=17,call=1)分支改走**专用** `_decodeProposeCloseInstitution`(不再复用个人 66 字节 `_decodeProposeClose`):解 `[scope:2][account:32][beneficiary:32]` 后顺序 skip 3 个 `Vec<u8>`(register_nonce/signature/issuer_cid_number,各 `_decodeCompactU32` 读长跳过)再 issuer_main_account:32 + signer_pubkey:32,末尾 `_hasValidSigningTail`;返回 action `propose_close_institution`/摘要"提案注销机构多签…"。个人多签 `propose_close`(per-mgmt 7.1)仍走原 66 字节解码,不动。`action_labels.dart` 机构标签改"注销机构多签提案"。**dart analyze 0 issue**。
- **F CID 前端注销入口**:`gov/GovDetailPage.tsx` 机构信息卡 `extra` 用 `<Space>` 包 CPMS 按钮 + `<Popconfirm>` 包 `<Button danger>注销机构</Button>`,门控 `canWrite && status==='ACTIVE' && created_by!=='SYSTEM'`(创世/治理机构前端即隐藏,后端 `is_genesis_protected`/org 闸权威兜底);`onDeregisterInstitution` 复用 `runPasskeyChallengeGrant('INSTITUTION_DEREGISTER', {target,cid_number})` → 成功提示后 `load()`。`admins/admin_security_api.ts` `AdminActionType` 补 `INSTITUTION_DEREGISTER`/`INSTITUTION_ACCOUNT_DEREGISTER`。**npx tsc --noEmit 0 error**。

**已完成并验证 — CID 前端两处登录/列表修复(2026-06-21)**:
- **无 passkey 登录直达管理员列表 tab**(#2):`core/InstitutionDetailNavLayout.tsx` 加 `initialActiveKey?` prop,`useState` 初始化优先取(命中 availableItems 才用);`gov/GovDetailPage.tsx` 传 `initialActiveKey={auth.passkey_bound===false && adminListSection ? 'admins' : undefined}`。无 passkey 管理员登录后直接落在【管理员列表】tab,看得到待绑 passkey 红点。
- **管理员姓名列显示哈希修复**(#3):`backend/admins/seed.rs` 联邦注册局 215 admin 的 `admin_display_name` 原写空串(`String::new()`),导致前端无 catalog 回退时裸显 0xd641… 哈希;改 `format!("{province}联邦注册局管理员{seat}")`(seat=`idx % FEDERAL_ADMINS_PER_PROVINCE + 1`)。开发库实测 0xd641…→"中枢省联邦注册局管理员1"。

**仅剩 follow-up(node + 集成测试)**:
- node:凡构造 propose_close 的 Tauri/前端调用面随签名变更补齐新参数(动态构造不阻断编译)。
- runtime 级集成测试(真实 InternalAdminProvider)验创世 PUP 自治全流程。
- 运行期端到端冒烟:真实机构 + 管理员 + passkey + 冷签 + 上链 close,带 dev DB。

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

## 已拍板补充（2026-06-21,落地即定）
- 链上 close 提案谁提交 = **(b) 机构 admin 带凭证提交**:CID 注册局只签发注销凭证(不代签 extrinsic),机构管理员在 CitizenWallet 冷签 `propose_close` 上链;CID 端绝不持机构私钥,符合"CID 不代签"红线。强制处置(a)若需,另立卡走治理投票路径,不混入本注销凭证通道。
