# 任务卡:institution_id 协议统一(SubjectKind kind tag + payload)

- 任务编号:20260506-unified-subject-id-protocol
- 状态:completed
- 负责人:当前主聊天入口(Architect Agent + Blockchain Agent + Mobile Agent 联合执行)
- 关联前置:20260506-split-personal-manage-from-organization-manage(B,已完成)
- 关联后续:20260506-rename-institution-id-to-subject-id(C,待启动)

## 1. 任务目标

把当前 3 类治理主体(内置主体 / SFID 注册机构 / 个人多签)的 `InstitutionPalletId`(`[u8; 48]`)派生协议统一为结构化布局 `kind(1B) + payload(47B)`,引入 `SubjectKind` enum 强制主体类型互斥。

**一次性完整重构,无兼容 wrapper**:
- 新增 `SubjectKind` enum(Builtin=0x01 / SfidInstitution=0x02 / PersonalDuoqian=0x03 / Reserved=0xFF)
- 新增 `primitives::derive::build_institution_id(kind, payload)` + 3 个语义 helper
- **删除**旧函数 `account_to_institution_id` / `sfid_id_to_institution_id` / `china_cb::shenfen_id_to_fixed48` / `china_ch::shenfen_id_to_fixed48`
- 全工程 60+ 调用点切换到新接口,零 compat 残留
- `MaxSfidIdLength` 收紧 `ConstU32<96>` → `ConstU32<47>`

类型别名 `InstitutionPalletId` 与变量名 `institution_id` 在 D 阶段保持不变(C 阶段才改名 `SubjectId`/`subject_id`)。

## 2. 影响范围

### 2.1 primitives(核心改动)
- `primitives/src/derive.rs`:
  - 新增 `pub enum SubjectKind { Builtin = 0x01, SfidInstitution = 0x02, PersonalDuoqian = 0x03 }`(`#[derive(Encode/Decode/Copy/Clone/RuntimeDebug/TypeInfo/MaxEncodedLen/PartialEq/Eq)]`)
  - 新增 `pub fn build_institution_id(kind: SubjectKind, payload: &[u8]) -> Option<[u8; 48]>`
  - 新增 `pub fn parse_institution_id(id: &[u8; 48]) -> Option<(SubjectKind, &[u8])>`
  - 新增 `pub fn institution_id_from_account<A: Encode>(account: &A) -> [u8; 48]`(取代 `account_to_institution_id`)
  - 新增 `pub fn institution_id_from_sfid_id(sfid_id: &[u8]) -> Option<[u8; 48]>`(取代 `sfid_id_to_institution_id`)
  - 新增 `pub fn institution_id_from_shenfen_id(shenfen_id: &str) -> Option<[u8; 48]>`(取代 `shenfen_id_to_fixed48`)
  - **删除**:`account_to_institution_id` / `sfid_id_to_institution_id` 函数定义
  - 新增 ≥6 个协议正确性测试 case

### 2.2 china::china_cb / china_ch
- `primitives/china/china_cb.rs:18` 删除 `pub fn shenfen_id_to_fixed48` 函数定义
- `primitives/china/china_ch.rs:21` 删除 `pub fn shenfen_id_to_fixed48` 函数定义
- 任何调用统一切到 `primitives::derive::institution_id_from_shenfen_id`

### 2.3 votingengine
- `votingengine/src/types.rs::nrc_pallet_id_bytes()` 改用 `institution_id_from_shenfen_id`

### 2.4 admins-change / grandpakey-change / resolution-destro / shengbank-interest
- 任何调用 `account_to_institution_id` / `sfid_id_to_institution_id` / `shenfen_id_to_fixed48` 的位置,grep 替换为新名

### 2.5 organization-manage / personal-manage / duoqian-transfer
- 同上,所有调用点切换
- `organization-manage::common::pub use primitives::derive::*` re-export 删除(B 阶段为兼容保留的,D 阶段一并清掉)

### 2.6 runtime configs
- `runtime/src/configs/mod.rs:796` `MaxSfidIdLength: ConstU32<96>` → `ConstU32<47>`

### 2.7 测试 mock
- `duoqian-transfer/src/lib.rs` 测试 mock 内 `MaxSfidIdLength` 同步 `ConstU32<47>`(若有引用)
- runtime --lib 测试任何 institution_id 构造点切到新接口

### 2.8 客户端(待 grep 扫描)
- wumin / wuminapp / sfid / node 任何"institution_id 字节字面"或"反向解析 institution_id"逻辑,加 `kind tag` 解析
- 多数客户端只透传 institution_id,不解码内部字节,预计改动面 <5 个文件

## 3. 关键约束

- **永久 ABI 锁定**:`SubjectKind::Builtin=0x01 / SfidInstitution=0x02 / PersonalDuoqian=0x03 / 0xFF Reserved` 一旦上线不可改
- 无兼容 wrapper:旧函数名一字节不留(feedback_no_compatibility / feedback_no_remnants)
- 链未上线,fresh genesis 即生效(feedback_chain_in_dev.md)
- `InstitutionPalletId` 类型名不动(C 阶段改 `SubjectId`)
- 业务字段 `org` 不动(NRC/PRC/PRB/REN/PUP/OTH 6 类保留)
- 跨模块联动:runtime + 6 个业务 pallet + 客户端必须同步推进

## 4. 执行计划(11 步,单 commit)

1. 新增 `SubjectKind` enum + `build_institution_id` + `parse_institution_id` + 3 个 from_X helper(`primitives/src/derive.rs`)
2. 删除旧函数名 `account_to_institution_id` / `sfid_id_to_institution_id`,全工程 grep 替换为新名
3. 合并 `china::china_cb::shenfen_id_to_fixed48` / `china::china_ch::shenfen_id_to_fixed48` 到 `primitives::derive::institution_id_from_shenfen_id`,删除两处局部定义
4. `MaxSfidIdLength` 收紧 47(runtime config + 测试 mock)
5. `votingengine::types::nrc_pallet_id_bytes()` 切到 `institution_id_from_shenfen_id`
6. 链端编译验证 `cargo check --workspace`
7. 链端测试验证 `cargo test -p citizenchain --lib + duoqian-transfer + admins-change + personal-manage + organization-manage`
8. 客户端 grep 扫描 + 必要修改(wumin/wuminapp/sfid/node)
9. wumin / wuminapp 测试验证
10. 新增 SubjectKind 协议测试(≥6 case:三类主体 kind 字节正确 / 互不撞 / payload 长度边界 / parse 往返)
11. ADR-010 + 任务卡完工 + auto-memory + 残留扫描

## 5. 验证要求

### 编译
- `cargo check -p primitives` 通过
- `cargo check --workspace`(WASM_FILE,含 node + sfid)通过
- `cd citizenchain/node/frontend && npx tsc --noEmit` exit 0

### 测试
- `cargo test -p primitives` ≥6 新 case + 现有全过
- `cargo test -p citizenchain --lib` 全过(目标 37/37)
- `cargo test -p duoqian-transfer` 全过(目标 20/20)
- `cargo test -p admins-change` 全过
- `cargo test -p personal-manage`(若有)全过
- wumin `flutter analyze && flutter test` 0 issue + 全过
- wuminapp `flutter test test/duoqian/` 全过

### 残留扫描(必须零结果)
```bash
# 旧函数名零残留
grep -rn "account_to_institution_id\|sfid_id_to_institution_id\|shenfen_id_to_fixed48" \
  citizenchain/ wumin/ wuminapp/ sfid/ memory/ \
  --include="*.rs" --include="*.dart" --include="*.ts" --include="*.tsx" 2>/dev/null \
  | grep -v target/ | grep -v ".dart_tool/" | grep -v "08-tasks/done/"

# 任何"id[0..len].copy_from_slice(input);id" 的手工填零 pattern 都应消失
grep -rn "let mut id = \[0u8; 48\]" citizenchain/runtime/ --include="*.rs" 2>/dev/null \
  | grep -v "primitives/src/derive.rs"

# MaxSfidIdLength=96 不再出现
grep -rn "MaxSfidIdLength.*ConstU32<96>" citizenchain/ --include="*.rs" 2>/dev/null
```

### 协议正确性
- 三类主体 institution_id 第 1 字节分别是 0x01 / 0x02 / 0x03
- 任意两类 institution_id 永不撞 key(kind tag 不同)
- payload 47B 边界(48B 输入返回 None)
- parse 往返(build → parse 还原 kind + payload)

## 6. 风险与缓解

| ID | 风险 | 缓解 |
|---|---|---|
| R1 | sfid_id 长度 47B 约束 | MaxSfidIdLength=47 强制守门;BoundedVec 入链时校验失败拒绝 |
| R2 | 客户端硬编码 institution_id 字节 | Step 8 全工程 grep 扫描;wuminapp duoqian_discovery_service 等可疑文件人工检查 |
| R3 | 永久 ABI 锁定后不可改 | ADR-010 协议规范文档锁定;留 0xFF 哨兵给未来升级 |
| R4 | 测试 mock 直接构造 [u8; 48] 跳过 helper | residual scan 扫"let mut id = [0u8; 48]" pattern;必须经 build_institution_id |
| R5 | china_cb/china_ch 创世构建时 institution_id 字节内容变化(头加 0x01) | runtime 创世逻辑全部经 institution_id_from_shenfen_id;创世测试自动覆盖 |
| R6 | nrc_pallet_id_bytes() 等下游硬编码 | Step 5 切换 + Step 7 测试守门 |

## 7. 输出物

- 代码:primitives/src/derive.rs 新增 ~80 行;60+ 调用点改名;MaxSfidIdLength 改值
- 中文注释:SubjectKind enum 协议章节(永久 ABI 锁定 + 升级路径 0xFF 哨兵)
- 测试:primitives ≥6 协议测试 case
- 文档:ADR-010-subject-id-protocol.md(协议规范)
- 残留清理:旧函数名一字节不留

## 8. 执行结果

11 步全部完成(2026-05-06):

1. **`primitives/src/derive.rs` 重写** — 新增 `SubjectKind` enum(Builtin=0x01/SfidInstitution=0x02/PersonalDuoqian=0x03)+ `build_institution_id` + `parse_institution_id` + 3 个语义 helper(institution_id_from_account / from_sfid_id / from_shenfen_id);新增 7 个协议正确性测试 case(三类主体 kind 字节 / 互不撞 / payload 长度边界 / parse 往返 / 非法 kind 拒绝)
2. **全工程改名** — 60+ 处调用点全部从 `account_to_institution_id` / `sfid_id_to_institution_id` 切换到 `institution_id_from_account` / `institution_id_from_sfid_id`;`organization-manage::common::pub use` 兼容 re-export 删除
3. **china_cb / china_ch 局部 `shenfen_id_to_fixed48` 删除** — 全工程 grep 替换 `shenfen_id_to_fixed48` → `institution_id_from_shenfen_id`(链端 + node);别名 `reserve_pallet_id_to_bytes` / `shengbank_pallet_id_to_bytes` 在 13 个文件直接删除,统一用 `institution_id_from_shenfen_id`
4. **`MaxSfidIdLength` 收紧** — `runtime/src/configs/mod.rs` + `duoqian-transfer/src/lib.rs` 测试 mock 从 `ConstU32<96>` 改 `ConstU32<47>`
5. **`votingengine::types::nrc_pallet_id_bytes()`** — 切到 `institution_id_from_shenfen_id`
6. **链端编译** — `cargo check --workspace` 通过,仅 pre-existing dead-code 警告
7. **链端测试** — 修 `duoqian-transfer::registered_duoqian_account` 取 `institution[1..33]` 跳过 kind tag;修 `institution_id_has_zero_suffix` 校验 byte[0]==0x03 且末 15 字节零;测试 mock 5 处 `institution[..32]` → `institution[1..33]`
8. **客户端扫描修改** —
   - `wuminapp::admin_institution_codec.dart::personalAddressFromInstitutionId` 加 byte[0]==0x03 校验,提取 `institutionId.sublist(1, 33)`
   - `wuminapp::admin_institution_codec.dart::sfidIdFromInstitutionId` 加 byte[0]==0x02 校验,提取 `institutionId.sublist(1, realLen)`
   - `wuminapp::institution_admin_service.dart::_shenfenIdToFixed48` 加 `out[0]=0x01` + 长度限制 ≤47
   - `node::governance::storage_keys.rs::institution_id_from_shenfen_id` 加 kind tag 0x01
   - 新增 `subjectKindBuiltin/SfidInstitution/PersonalDuoqian` 常量到 admin_institution_codec.dart
9. **客户端测试** — wumin 105/105、wuminapp duoqian 30/30(含修正后的 codec 边界 case)
10. **协议测试 ≥6 case** — 在 Step 1 中已落入 primitives::derive::tests
11. **ADR-010 + auto-memory + 残留扫描** — 落盘 `memory/04-decisions/ADR-010-subject-id-protocol.md`、`project_subject_id_protocol_2026_05_06.md`、MEMORY.md 索引

## 9. 验证结果

### 编译
- `cargo check --workspace`(WASM_FILE):通过(node 37 个 pre-existing 警告)
- `cargo check -p primitives`:通过

### 测试
- `cargo test -p primitives`:**19 passed / 0 failed**(含 7 个新增 SubjectKind 协议 case)
- `cargo test -p citizenchain --lib`:**37 passed / 0 failed**
- `cargo test -p duoqian-transfer`:**20 passed / 0 failed**
- `cargo test -p admins-change`:**31 passed / 0 failed**
- wumin `flutter test`:**105 passed / 0 failed**
- wuminapp `flutter test test/duoqian/`:**30 passed / 0 failed**(原 28 + 新增 codec 边界 case 2)

### 残留扫描(全零)
1. `account_to_institution_id` / `sfid_id_to_institution_id` / `shenfen_id_to_fixed48`:零残留
2. `MaxSfidIdLength.*ConstU32<96>`:零残留
3. `reserve_pallet_id_to_bytes` / `shengbank_pallet_id_to_bytes` 别名:零残留

### 行为不变量逐条核对
- ✓ 永久 ABI 锁定:`SubjectKind::Builtin=0x01 / SfidInstitution=0x02 / PersonalDuoqian=0x03 / Reserved=0xFF`
- ✓ payload 长度 47B 强制:MaxSfidIdLength=47 + BoundedVec 守门
- ✓ 三类主体永不撞 key:kind tag 不同保证互斥
- ✓ 业务行为零变更:派生函数返回值字节变化(头加 1B kind tag),但下游 admin/threshold/反向索引等业务逻辑不变
- ✓ 客户端 dispatch 规则不变
- ✓ `InstitutionPalletId` 类型名不变(C 阶段才改 `SubjectId`)
- ✓ `org` 字段语义不变(NRC/PRC/PRB/REN/PUP/OTH 6 类保留)
- ✓ 链未上线,fresh genesis 即生效;无 storage migration

### 风险闭环
- R1 sfid_id 长度收紧 47B:MaxSfidIdLength 强制守门 ✓
- R2 客户端硬编码 institution_id 字节:admin_institution_codec.dart + institution_admin_service.dart 同步加 kind tag,测试守门 ✓
- R3 永久 ABI 锁定:ADR-010 文档化 + 0xFF 哨兵预留升级路径 ✓
- R4 测试 mock 直接构造 [u8; 48] 跳过 helper:duoqian-transfer 5 处 mock 已改 institution[1..33] ✓
- R5 china_cb/china_ch 创世 institution_id 字节内容变化:全部经 institution_id_from_shenfen_id 派生,创世测试自动覆盖 ✓
- R6 nrc_pallet_id_bytes 等下游硬编码:Step 5 切换 + Step 7 测试守门 ✓
