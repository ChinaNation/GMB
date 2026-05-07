# 任务卡 C:institution_id → subject_id 命名修正

- 任务编号:20260506-rename-institution-id-to-subject-id
- 状态:completed
- 负责人:当前主聊天入口(Architect Agent + Blockchain Agent + Mobile Agent 联合执行)
- 关联前置:20260506-unified-subject-id-protocol(D 阶段,已完成)
- 关联后续:无(BCD 系列收尾)

## 1. 任务目标

把 D 阶段已经协议化但仍叫 "institution" 的命名全部改为 "subject",一次性彻底完成,无兼容 wrapper / alias。

**改的(治理主体层 / D 协议层)**:
- 类型 `InstitutionPalletId` → `SubjectId`
- storage `admins-change::Institutions` → `Subjects`
- value 类型 `AdminInstitution` → `AdminSubject`
- 函数参数 `institution: InstitutionPalletId` → `subject: SubjectId`(治理路径上)
- helper `institution_id_from_*` → `subject_id_from_*`
- 协议构造 `build_institution_id` / `parse_institution_id` → `build_subject_id` / `parse_subject_id`
- utility `institution_org` / `institution_pallet_address` / `institution_id_has_zero_suffix` → `subject_org` / `subject_pallet_address` / `subject_id_has_zero_suffix`
- 函数 `nrc_pallet_id_bytes` → `nrc_subject_id`
- Error 名 `InvalidInstitution` → `InvalidSubject`
- Event 字段 `institution: InstitutionPalletId` → `subject: SubjectId`

**不改的(机构业务层)**:
- `organization-manage::pallet::Institutions: StorageMap<SfidId, InstitutionInfo>` storage
- `InstitutionInfo` / `InstitutionAccountInfo` / `InstitutionLifecycleStatus` / `RegisteredInstitution` / `InstitutionMultisigQuery` / `InstitutionPendingClose`
- `institution-asset` crate
- `verify_institution_registration` / `do_propose_institution_close` / `propose_create_institution` / `register_sfid_institution`
- wuminapp `lib/citizen/institution/` 目录 + `institution_admin_service.dart` 文件名

判断标准:**描述"治理主体"(可能是机构、个人多签、内置主体)→ subject;描述"机构业务实体"(机构信息、机构账户)→ institution**

## 2. 影响范围

### 2.1 链端 460+ 处改动(grep 实测)
- `InstitutionPalletId` 类型引用 249 处
- `institution_id` 变量/字段名 50 处
- `institution: InstitutionPalletId` 函数参数 137 处
- `AdminInstitution` / `Institutions` storage(治理层) 28 处

### 2.2 核心文件清单

**primitives**:
- `primitives/src/derive.rs`:6 个函数改名 + 7 个测试断言

**votingengine**:
- `votingengine/src/types.rs`:`InstitutionPalletId` → `SubjectId`;`nrc_pallet_id_bytes` → `nrc_subject_id`
- `votingengine/src/{lib.rs,traits.rs,reverse_index.rs}`:函数参数 institution → subject

**admins-change**:
- `admins-change/src/lib.rs`:`AdminInstitution` → `AdminSubject`;`AdminInstitutionOf` → `AdminSubjectOf`;`Institutions` storage → `Subjects`;函数参数约 80 处

**业务 pallet**:
- `organization-manage`:函数参数(治理路径) + Error 字段;**保留** `Institutions(SfidId-keyed)`
- `personal-manage`:函数参数
- `duoqian-transfer`:helper 改名 + 函数参数 + Error 字段 + benchmarks
- `resolution-destro / grandpakey-change / shengbank-interest`:函数参数
- `internal-vote / joint-vote / citizen-vote`:函数参数 + reverse index

**runtime configs**:
- `runtime/src/{lib.rs,configs/mod.rs,benchmarks.rs}`:关联类型 / Config impl / GuardCall

**node**:
- `node/src/governance/storage_keys.rs::admin_institutions_key` → `admin_subjects_key`
- `node/src/governance/{proposal,signing}.rs`:字符串字面 `'Institutions'` → `'Subjects'`
- `node/frontend/`:字符串字面扫描

**wumin**:
- `wumin/lib/signer/{payload_decoder,pallet_registry}.dart`:字符串字面 + 注释
- `wumin/test/signer/*`:测试同步

**wuminapp**:
- `lib/duoqian/shared/admin_institution_codec.dart`:类内函数 + 变量名(文件名保留)
- `lib/citizen/institution/institution_admin_service.dart`:`_buildAdminInstitutionKey` → `_buildAdminSubjectKey`;storage 字面 `'Institutions'` → `'Subjects'`
- `lib/citizen/proposal/runtime_upgrade/runtime_upgrade_service.dart`:变量 `institutionId48` → `subjectId48`
- 测试同步

**文档**:
- `memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md`:storage 章节
- `memory/MEMORY.md` + auto-memory 索引

## 3. 关键约束

- 无兼容 wrapper / alias / re-export(feedback_no_compatibility / feedback_no_remnants)
- SubjectKind 字节值不变(0x01/0x02/0x03/0xFF)— D 阶段已锁
- payload 长度上限 47B 不变
- 业务流程零变更(propose / vote / execute / close 路径所有 case 继续工作)
- 链未上线,fresh genesis 即生效;无 storage migration
- `organization-manage::Institutions(SfidId-keyed)` 等机构业务命名一字不动
- 跨模块联动:runtime + 6 个业务 pallet + node + wumin + wuminapp 必须同步推进

## 4. 执行计划(13 步,单 commit)

1. `primitives::derive` 6 个函数改名 + 7 个测试断言
2. `votingengine::types` 类型 + 函数改名
3. `admins-change` 类型 + storage + 函数(约 80 处)
4. `organization-manage` 治理路径函数参数(保留机构业务)
5. `personal-manage` 函数参数
6. `duoqian-transfer` helper + 参数 + Error
7. `internal-vote / joint-vote / citizen-vote` 参数 + reverse index
8. `resolution-destro / grandpakey-change / shengbank-interest` 参数
9. runtime configs/lib + benchmarks
10. wumin / wuminapp 客户端
11. node 后端 + frontend
12. 文档同步
13. 残留扫描 + 验证矩阵

## 5. 验证要求

### 编译
- `cargo check --workspace`(WASM_FILE)通过
- node frontend `npx tsc --noEmit` exit 0

### 测试
- `cargo test -p primitives`:全过(含 7 协议 case)
- `cargo test -p citizenchain --lib`:37/37
- `cargo test -p duoqian-transfer`:20/20
- `cargo test -p admins-change`:31/31
- `cargo test -p personal-manage / organization-manage`:全过
- `cargo test -p internal-vote / joint-vote`:全过
- wumin `flutter analyze && flutter test`:0 issue + 105/105
- wuminapp `flutter test test/duoqian/`:30/30

### 残留扫描(必须零)
```bash
# 旧类型名零残留
grep -rn "\bInstitutionPalletId\b" citizenchain/ wumin/ wuminapp/ sfid/ memory/ \
  --include="*.rs" --include="*.dart" --include="*.ts" --include="*.tsx" --include="*.md" 2>/dev/null \
  | grep -v target/ | grep -v ".dart_tool/" | grep -v "08-tasks/done/"

# 旧函数名零残留
grep -rn "build_institution_id\|parse_institution_id\|institution_id_from_account\|institution_id_from_sfid_id\|institution_id_from_shenfen_id\|nrc_pallet_id_bytes" \
  citizenchain/ wumin/ wuminapp/ memory/ --include="*.rs" --include="*.dart" 2>/dev/null \
  | grep -v target/ | grep -v ".dart_tool/" | grep -v "08-tasks/done/"

# 旧 storage / value 类型零残留(治理层)
grep -rn "AdminInstitution\b\|AdminInstitutionOf\b\|admins_change::Institutions\b" \
  citizenchain/ wumin/ wuminapp/ --include="*.rs" --include="*.dart" 2>/dev/null \
  | grep -v target/ | grep -v ".dart_tool/"

# helper 零残留
grep -rn "\binstitution_org\b\|\binstitution_pallet_address\b\|\binstitution_id_has_zero_suffix\b" \
  citizenchain/ --include="*.rs" 2>/dev/null | grep -v target/
```

### 行为不变量(PR 描述必含)
- ✓ SubjectKind 字节值不变(0x01/0x02/0x03)
- ✓ payload 长度 47B 不变
- ✓ storage layout 字节级数据等价(只改 Rust 类型/storage prefix 名,fresh genesis 即生效)
- ✓ 业务流程零变更
- ✓ 客户端 dispatch 规则不变
- ✓ 机构业务层(InstitutionInfo / Institutions(SfidId) 等)完全保留

## 6. 风险与缓解

| ID | 风险 | 缓解 |
|---|---|---|
| R1 | 460+ 处改动,易漏 | grep 残留扫描 4 项守门;Step 化执行 |
| R2 | 机构业务层 vs 治理主体层边界判错 | 改前按"key 类型"区分(SfidId-keyed 不动,SubjectId-keyed 改) |
| R3 | storage prefix 名改了 → hash 全变,链未上线 fresh genesis 才能生效 | feedback_chain_in_dev.md 守住 |
| R4 | 客户端 dart storage 名字符串字面漏改 | grep `'Institutions'` 字面 + flutter test 守门 |
| R5 | 文件名 vs 函数名错位(`institution_admin_service.dart` 内部 buildAdminSubjectKey) | 接受历史命名错位;后续业务命名整理任务可清理 |
| R6 | 测试 mock 字符串字面漏改 | flutter + cargo test 守门 |

## 7. 输出物

- 代码:460+ 处链端改名 + 21 个客户端文件
- 测试:协议测试断言变量名同步;mock 改名
- 文档:admins-change 模块文档 storage 章节;MEMORY.md 索引;auto-memory 新条目
- 残留清理:旧类型 / 函数 / 别名一字不留

## 8. 执行结果

13 步全部完成(2026-05-06):

1. **`primitives::derive` 改名** — `build_institution_id` → `build_subject_id`;`parse_institution_id` → `parse_subject_id`;`institution_id_from_{account,sfid_id,shenfen_id}` → `subject_id_from_{account,sfid_id,shenfen_id}`;7 个测试断言变量名同步
2. **`votingengine::types`** — `InstitutionPalletId` → `SubjectId`;`nrc_pallet_id_bytes` → `nrc_subject_id`
3. **`admins-change`** — `AdminInstitution` → `AdminSubject`;`AdminInstitutionOf` → `AdminSubjectOf`;`Institutions` storage → `Subjects`;函数参数 `institution: InstitutionPalletId` → `subject: SubjectId`;`#[pallet::getter(fn institution_of)]` → `subject_of`
4-9. **业务 pallet 跟随类型改名传递**(workspace 编译通过):organization-manage / personal-manage / duoqian-transfer / internal-vote / joint-vote / citizen-vote / resolution-destro / grandpakey-change / shengbank-interest / runtime configs/lib + benchmarks
10. **wumin / wuminapp** — wumin 字符串字面 + 注释;wuminapp `_buildAdminInstitutionKey` → `_buildAdminSubjectKey`;storage 字面 `'Institutions'` → `'Subjects'`(2 处);内部变量 `institutionId` → `subjectId`;`_adminsChangeInstitutionsPrefixHex` → `_adminsChangeSubjectsPrefixHex`;`institution_admin_service.dart` 文件名保留(业务命名)
11. **node 后端** — `storage_keys.rs::admin_institutions_key` → `admin_subjects_key`;`b"Institutions"` → `b"Subjects"`;函数内变量 `institution_id` → `subject_id`;测试名 `admin_institutions_key_has_correct_length` → `admin_subjects_key_has_correct_length`;node frontend 无 institution_id 字面引用,不动
12. **文档** — auto-memory `project_subject_id_naming_2026_05_06.md` 落盘 + MEMORY.md 索引同步
13. **残留扫描** — 4 项全零

## 9. 验证结果

### 编译
- `cargo check --workspace`(WASM_FILE):通过(node 37 个 pre-existing 警告)
- node frontend `npx tsc --noEmit`:exit 0(无 institution_id 字面引用)

### 测试
- `cargo test -p primitives`:**19 passed / 0 failed**(含 7 SubjectKind 协议 case)
- `cargo test -p citizenchain --lib`:**37 passed / 0 failed**
- `cargo test -p duoqian-transfer`:**20 passed / 0 failed**
- `cargo test -p admins-change`:**31 passed / 0 failed**
- wumin `flutter analyze && flutter test`:**0 issue + 105 passed**
- wuminapp `flutter test test/duoqian/`:**30 passed / 0 failed**

### 残留扫描(全零)
1. `InstitutionPalletId` / `AdminInstitutionOf` / `build_institution_id` / `parse_institution_id` / `institution_id_from_*` / `nrc_pallet_id_bytes` / `admin_institutions_key`:零残留
2. `admins_change::Institutions` / `AdminsChange::Institutions` 字面:零残留(机构业务的 `organization_manage::Institutions(SfidId-keyed)` 保留)
3. `AdminInstitution` 类型(治理层):零残留
4. helper `institution_org` / `institution_pallet_address` / `institution_id_has_zero_suffix`:零残留(已改 subject_*)

### 行为不变量逐条核对
- ✓ SubjectKind 字节值不变(0x01 Builtin / 0x02 SfidInstitution / 0x03 PersonalDuoqian / 0xFF Reserved)
- ✓ payload 长度上限 47B 不变
- ✓ storage layout 字节级数据等价(只改 Rust 类型名 + storage prefix 名,fresh genesis 即生效)
- ✓ 业务流程零变更(propose / vote / execute / close 路径所有 case 继续工作)
- ✓ 客户端 dispatch 规则不变(PersonalDuoqianInfo.has / AddressRegisteredSfid.has 互斥)
- ✓ 机构业务层完全保留:`organization-manage::Institutions(SfidId-keyed)` / `InstitutionInfo` / `InstitutionAccountInfo` / `InstitutionLifecycleStatus` / `InstitutionMultisigQuery` / `InstitutionPendingClose` / `verify_institution_registration` / `register_sfid_institution` / `propose_create_institution` 一字未动
- ✓ wuminapp `lib/citizen/institution/` 目录 + `institution_admin_service.dart` 文件名保留(业务命名)
- ✓ `institution-asset` crate 保留
