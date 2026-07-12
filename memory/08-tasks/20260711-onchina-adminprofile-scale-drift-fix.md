# 任务卡：onchina 冷签 AdminProfile SCALE 线格漂移修复(四端统一)

## 任务需求

来源:命名审计任务([[project_naming_audit_2026_07_11]])阶段 G 跑 `cargo test --workspace` 时暴露的遗留生产 bug,独立于命名任务。

链端权威 `AdminProfile`(admin-primitives/src/lib.rs:96-115,2026-06-28 runtime breaking [[project_institution_admin_field_model_2026_06_28]])自那日起为 **9 字段**:
`admin_account[32] · admin_cid_number(BV) · admin_name(BV) · role_code(BV) · role_name(BV) · term_start(u32) · term_end(u32) · admin_source(enum1B) · admin_source_ref(BV)`
—— `admin_role` 拆成 `role_code + role_name`,尾部新增 `admin_source_ref`。

onchina 的手工 SCALE 编/解仍是 **7 字段旧布局**,冷签创建机构 call data 在链端解错位、执行失败/数据错乱。`cargo check` 抓不到(不引用链端字段名);唯一能抓的逐字节比对测试因引用已删 `admin_role: BoundedVec` 编译失败、从未跑到,故坏了 40 天没暴露。

用户拍板:**(1) 读路径镜像同源 bug 一并纳入;(2) 全仓库统一 `role_code`/`role_name`,展示层 `admin_role→role_name` 零例外**([[feedback_unify_means_zero_exceptions]]、[[feedback_no_remnants]])。

创世构造(genesis/institution.rs:313-325)为权威映射样例:`role_name = 职务文本`,`role_code = 空`,`admin_source_ref = 空`。onchina 沿用:表单单一职务 → `role_name`,另两字段留空,无需新增表单字段。

## 范围(SCALE AdminProfile 四个编/解点 + 展示统一)

四个真·线格点(必须逐字节对齐链端 9 字段):
1. onchina 写:`core/institution_call.rs` `AdminProfileArg` + `encode_admin_profile`(create-institution call data)。
2. onchina 读:`core/chain_runtime.rs` `OnChainAdminProfile` 解码镜像(读 AdminAccounts storage)。
3. CitizenWallet 解:`signer/payload_decoder.dart` `_decodeProposeCreateInstitution` skip 逻辑。
4. CitizenApp 读:`citizen/shared/admin_profile.dart` `decodeAdminsVec`(读 AdminAccounts storage)。

展示层 `admin_role→role_name` 统一(零例外):
- onchina:`chain_runtime.rs`(`OnChainAdminProfileView`.admin_role + `title` 残桩 + `admin_profile_views`)、`auth/model.rs·catalog.rs·city_registry_admins.rs·actions.rs`、`institution/subjects/model.rs`(`CreateInstitutionAdminInput`)·`registration.rs`·`registration_call.rs`、`domains/legislation/display/model.rs·service.rs`。
- CitizenApp:`admin_profile.dart`(字段 `adminRole→roleName`、缓存 JSON 键 + **bump 缓存版本/形状校验** [[feedback-dto-field-rename-bump-cache-version]])、`admin_profile_card.dart`。

金标/测试(逐字节 guard,即真源):
- onchina Rust 单测 `institution_call.rs`(`real_admin_profile`/`sample_admin`/full_args)+ `chain_runtime.rs:1676` 跨真类型对拍 → 用新 9 字段,让编译跑通。
- CitizenWallet 测试 `payload_decoder_test.dart` `buildProposeCreateInstitutionPayload`(第 991 行 in-test 金标)→ 9 字段。
- **注**:`memory/06-quality/fixtures/step2d_credential_payload.json` 无 create-institution 用例,无独立 JSON 需重生;create 的逐字节金标就在上述 in-test 构造器。

不受影响(已核实):admins-change(`propose_admin_set_change`)携带裸 `Vec<AccountId>` 非 AdminProfile;onchina/frontend `src` 零 `admin_role` 引用;runtime 权威定义已正确不改。

## 铁律

- 逐字节四端一致:node 构造 / node 读镜像 / CitizenWallet 解 / CitizenApp 读,九字段顺序类型完全对齐链端 `AdminProfile`([[project_unified_signing_protocol_adr026]])。
- 客户端按 call 名解码不撞名([[project_unified_voting_entry]] 同族踩坑)。
- 只在主检出 `/Users/rhett/GMB` 操作,绝不碰 worktree([[feedback_user_evaluates_in_main_checkout]]);改动留工作区不提交供 review。

## 验收

- citizenchain `cargo test -p onchina` GREEN(institution_call + chain_runtime 逐字节比对真正跑通)。
- CitizenWallet `flutter test`(propose_create 解码)+ `flutter analyze` 零新增。
- CitizenApp `flutter analyze` 零新增 + 相关 test GREEN。
- 全仓 `admin_role`/`adminRole` 展示残留零引用(SCALE 已切 role_code/role_name)。

## 进度(2026-07-11 已完成)

**执行中发现范围再扩(同源 2026-06-28 breaking):** 真 `AdminAccount` 除 AdminProfile 拆字段外,还新增**前导字段 `cid_number: AdminCidNumber`**(在 institution_code 之前,个人多签为空)。onchina 读镜像 + CitizenApp 三个 AdminAccount 解码器全部从 institution_code 起读、集体错位 → 一并修。

**四端 + 金标全部落地并逐字节验证 GREEN:**
- onchina 写:`institution_call.rs` `AdminProfileArg`(拆 role_code/role_name + admin_source_ref)+ `encode_admin_profile` 9 字段;上游 `registration_call.rs` 表单职务→role_name、另二字段空;表单 DTO `CreateInstitutionAdminInput.admin_role→role_name`。
- onchina 读:`chain_runtime.rs` `OnChainAdminProfile`(补 role_code/role_name/admin_source_ref)+ `OnChainAdminAccount`(补前导 cid_number)+ 视图 `OnChainAdminProfileView.admin_role→role_name`(删死残桩 `title`)。
- onchina 展示统一:`auth/model.rs`(三 DTO)·`catalog.rs`·`city_registry_admins.rs`·`actions.rs`·`legislation/display`(`SeatView.title→role_name`)全 `admin_role→role_name`;前端零引用无需改。
- CitizenWallet:`payload_decoder.dart` create skip 9 字段;金标 `buildProposeCreateInstitutionPayload` 重生。
- CitizenApp:`admin_profile.dart`(`decodeAdminsVec` 9 字段 + `adminRole→roleName` + 缓存键 `admin_role→role`)·`admin_account_codec.dart`/`multisig_storage_codec.dart`/`personal_manage_storage_codec.dart` 三解码器补前导 cid_number·`admin_profile_card.dart` 展示·`admin_account_service.dart` 缓存 `_schemaVersion=2`(旧布局缓存作废回退链读);四处金标测试(admins_change_codec/institution_admin_service/personal_manage_storage_codec/personal_manage_service)重生。

**验证:** onchina `cargo test` 131/131 GREEN(含 4 个逐字节比对:admin_profile_encoding / admin_profile_vec / full_args / onchain_admin_account_mirror);CitizenWallet `flutter analyze` 干净 + payload_decoder 71/71;CitizenApp `flutter analyze` 干净(2 既有 info 无关)+ 全部 AdminAccount/AdminProfile 金标测试 GREEN。`personal_proposal_history_service_test` 的 -4 已 git stash 定性为**干净树既有 Isar 隔离失败**,与本次无关。全仓 `admin_role/adminRole` 残留清零。

**改动留工作区未提交,供 review。**
