任务需求：
SFID 改造 Step 1:
1. 全 5 工程统一改名 `shenfen_id` / `sfid_id` → `sfid_number`,`shenfen_name` → `sfid_name`
   (覆盖所有 case 形态:snake/camel/Pascal/SCREAMING/字符串/URL/函数名)
2. 277 条内置机构 sfid_number 字面量按新规则重生成(强制 city=001 无例外,
   d=2026,n9 重算,c1 重算)

旧数据全部删除,无 migration。

所属模块：跨模块 SFID 命名统一(citizenchain + sfid + wuminapp + wumin + node frontend)

输入文档：
- memory/feedback_no_compatibility.md(死规则:绝不搞兼容)
- memory/feedback_user_naming_literal.md(用户命名字面照抄)
- memory/06-quality/20260507-china-sfid-remap-v2.md / .csv(277 条新映射表)

必须遵守：
- 不留旧名兼容,所有调用点统一切换
- subject_id_from_sfid_number 因参数类型不同(&str / &[u8])拆分两个函数:
  - Builtin(&str): subject_id_from_sfid_number(&str)
  - SfidInstitution(&[u8]): subject_id_from_registered_sfid_number(&[u8])
- 链上 AdminSubjectKind 枚举名(BuiltinInstitution/SfidInstitution/PersonalDuoqian)保留不动
- 不在 Step 1 重算派生地址,Step 2 跑 tools/duoqian.py --apply 同步

## 改动清单

### 工具新增/修改
- `tools/regenerate_china_sfids.py`:`new_city_code = "001"` 无条件
- `tools/big_bang_sfid_rename.py`(新建):一锅端跨工程 plain-text 替换
- `tools/resolve_stash_conflicts.py`(新建):git stash pop 冲突清理(保留 Stashed)

### 改名规则(12 条)
- snake_case:`shenfen_id`/`sfid_id` → `sfid_number`,`shenfen_name` → `sfid_name`
- SCREAMING:`SHENFEN_ID`/`SFID_ID` → `SFID_NUMBER`,`SHENFEN_NAME` → `SFID_NAME`
- PascalCase:`ShenfenId`/`SfidId` → `SfidNumber`,`ShenfenName` → `SfidName`
- camelCase:`shenfenId`/`sfidId` → `sfidNumber`,`shenfenName` → `sfidName`

### SFID 字面量重生成(277 条)
- 7 个 china_*.rs(cb 44 / ch 43 / lf 44 / sf 44 / jc 47 / jy 1 / zf 54)
- 强制 city=001(包括原 LN002 也改 LN001)
- d=2026 (D8 → D4)
- n9 = blake2b-256(sfid_name | a3 | province | city | t2 | "2026")[:4] mod 10^9
- c1 = checksum 重算

### 派生函数拆分(避免 &str / &[u8] 同名冲突)
- 新增 `subject_id_from_registered_sfid_number(&[u8]) -> Option<[u8;48]>` (kind=0x02)
- `subject_id_from_sfid_number(&str)` 保留(kind=0x01,Builtin)
- 调用方:organization-manage 的 byte 调用全切到 `subject_id_from_registered_sfid_number`

## 影响面统计

- 改名脚本扫描 1042 文件
- 实际改动 163 文件,3436 次替换
- 按后缀:.rs 87 / .dart 42 / .tsx 26 / .ts 8

## 验证(全部 ok)

| 工程 | 命令 | 结果 |
|---|---|---|
| sfid backend | `cargo test --bin sfid-backend` | **69/69** ✅ |
| organization-manage | `cargo test -p organization-manage` | **24/24** ✅ |
| admins-change | `cargo test -p admins-change` | **31/31** ✅ |
| duoqian-transfer | `cargo test -p duoqian-transfer` | **20/20** ✅ |
| votingengine | `cargo test -p votingengine` | **79/79** ✅ |
| internal-vote | `cargo test -p internal-vote` | **5/5** ✅ |
| 全 runtime | `cargo check --manifest-path runtime/Cargo.toml` | **0 error** ✅ |
| wuminapp | `flutter analyze && flutter test` | **0 issues + 154/154** ✅ |
| wumin | `flutter analyze && flutter test` | **0 issues + 94/94** ✅ |
| node/frontend | `npm run build` | **0 error** ✅ |
| 残留扫描 | `grep shenfen_id\|sfid_id 全工程` | **0 命中** ✅ |

## 不在本步范围(Step 2 处理)

- 277 条 main_address(blake2b-256 with sfid_number)重算
- 87 条 fee_address(cb + ch)重算
- 1 条 NRC_ANQUAN_ADDRESS(NRC sfid_number)重算
- 408 条 china_zb.rs CHINA_RESERVED_MAIN_ADDRESSES 重算
- tools/duoqian.py NRC_SFID_NUMBER 常量改名 + 改值
- 跑 `python3 tools/duoqian.py --apply` 自动同步

## 输出物
- 7 个 china_*.rs 改名 + 277 条 sfid_number 字面量替换
- 53+77+8=138 个改名命中点(去重 100+ 文件)
- 3 个工具脚本(regenerate / big_bang / resolve_stash)
- 任务卡(本文件)
- memory 固化
