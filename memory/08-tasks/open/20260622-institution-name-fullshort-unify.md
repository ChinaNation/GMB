# 机构全称/简称统一(单一命名 + runtime 保护锚)

## 状态

**1.1 本地实现与验收完成(2026-06-22)。** 用户已允许直接修改 `citizenchain/runtime/`。本任务目标提升为全仓机构名称硬统一:任何机构中文全称/中文简称/英文全称/英文简称只允许使用 `cid_full_name / cid_short_name / cid_full_name_en / cid_short_name_en` 四字段;Dart/TS 内部使用 `cidFullName / cidShortName / cidFullNameEn / cidShortNameEn`;不得再用旧展示列、旧全称列、旧简称列、旧英文名列、旧登录简称字段或旧接口名承载机构名称。生成物已同步修改生成器,不得只改输出文件。

**1.2 本地实现与真实库验收完成(2026-06-23)。** 非常量机构只在命名规范文件登记英文规则,不把英文名扩入 CID 数据库/API/前端字段。非常量机构中文全称/简称模板已统一,规范文件已补每类机构的 `province_code / city_code / town_code` 行政区代码规则。白皮书和公民宪法仅完成扫描,按用户要求等待二次确认后再修改。

## 背景(reconcile 后开发库终态)

PUBLIC 共 245,016,其中 `cid_short_name == cid_full_name` = 50,795,结构 = **5 真 bug + 50,661 模板故意短==全 + 129 本就最短**。

- 旧口径认为常量机构走字符串匹配函数产出 `(full, short)` 已足够;用户最终确认此口径不够。该字符串匹配函数是第二实现源,必须删除;常量机构必须直接携带名称字段。
- **病根**:历史实现拿 china_*.rs 的中文全名当 match 键。china 改名 → arm 静默落默认臂 → 简称=全称。国家储委会、4 联邦局、129 省两院都是这一个病根。

## 已确认决策

1. **4 总统府联邦局简称 = 去前缀**(注释明确「勿擅造」,本值经用户确认):
   - 总统府联邦安全局 → 简称 `联邦安全局`
   - 总统府联邦情报局 → 简称 `联邦情报局`
   - 总统府联邦特勤局 → 简称 `联邦特勤局`
   - 总统府联邦人事局 → 简称 `联邦人事局`
   - (全称保持,与「总统府联邦注册局 → 联邦注册局」一致)
2. **排期**:并入 T3/T4 同次 reconcile(见上「状态」)。

## 锁定的目标名字对

### A. 常量机构(`china_*.rs` 直接携带 full/short)
| 机构(china 全名) | 全称 | 简称 | 现状 |
|---|---|---|---|
| 总统府联邦安全局 | 总统府联邦安全局 | 联邦安全局 | 已落 `cid_full_name/cid_short_name` |
| 总统府联邦情报局 | 总统府联邦情报局 | 联邦情报局 | 已落 `cid_full_name/cid_short_name` |
| 总统府联邦特勤局 | 总统府联邦特勤局 | 联邦特勤局 | 已落 `cid_full_name/cid_short_name` |
| 总统府联邦人事局 | 总统府联邦人事局 | 联邦人事局 | 已落 `cid_full_name/cid_short_name` |
| 国家公民储备委员会 | 国家公民储备委员会 | 国家储委会 | 已经真实库 reconcile/check 通过 |

### B. 模板机构(改 `cid_short_name_suffix`,`cid_full_name_suffix` 保持)
| org_code/模板 | cid_full_name_suffix(保持) | cid_short_name_suffix(改为) | 条数 |
|---|---|---|---|
| TOWN_GOV | 自治政府 | 政府 | 39,087 |
| CITY_GOV | 自治政府 | 政府 | 2,872 |
| CITY_EDU | 公民教育委员会 | 教委会 | 2,872 |
| PROVINCE_SENATE_COUNCIL | 联邦立法院参议会 | 参议会 | 43 |
| PROVINCE_REPRESENTATIVE_COUNCIL | 联邦立法院众议会 | 众议会 | 43 |
| CITY_COURT | 司法院 | 司法院(保持,已最短) | 2,872 |
| CITY_SUPERVISION | 监察院 | 监察院(保持,已最短) | 2,872 |

> 注:CITY_COURT/SUPERVISION 的 短==全 是合法终态(已最短),不是 bug,保留。

### C. 省两院 129 条(PROVINCE_LEGISLATURE/COURT/SUPERVISION,常量默认臂)
目标:全称带「联邦」(`X省联邦立法院/司法院/监察院`),简称去「联邦」(`X省立法院/…`)。
- ⚠️ **执行前先核对** china_lf/sf/jc.rs 省级常量实际存的全称:若已是「X省联邦立法院」则只需补 `cid_short_name` 让简称去联邦;若存的是「X省立法院」则全称需补联邦,别盲改。

## 架构原则(2026-06-22 用户定稿 —— 反转早先"移出"方向)

**⚠️ 本节取代早先"把名字移出 china_*.rs"和"subjects 可与 china 任意分叉"两种旧说法。最终语义:**

- `china_*.rs` 不是 CID 运营展示名的数据库真源;它是内置重要机构全称/简称的 **runtime 保护锚**。
- 内置机构全称/简称可以在 CID 系统中修改,但只在 CID 链下展示/运营数据中生效。
- 要让区块链侧保护锚同步改变,必须修改 `china_*.rs` 的 `cid_full_name / cid_short_name`,从而触发 runtime 升级。
- 全仓实现形态只能有一组字段:中文全称 `cid_full_name`,中文简称 `cid_short_name`,英文全称 `cid_full_name_en`,英文简称 `cid_short_name_en`。禁止再用旧展示列、旧全称列、旧简称列、旧英文名列或旧登录简称字段承载机构名称。

**三条轴(各管各的)**:
- **创世轴(改=重新创世)**:`cid_number` + 由它派生的全部账户(`derive_account(OP_x, ss58, cid_number)`):OP_MAIN/OP_FEE/OP_STAKE(永久质押)/OP_AN(安全基金)/OP_HE(两和基金)+ china_zb 落给它们的创世余额。**单根=cid_number**(改号→全部派生账户平移→余额错位),外加派生原语 `core_const.rs:40-46/89`。
- **runtime 升级轴(改=setCode)**:**内置重要机构全称/简称保护锚**(在 china_*.rs)。故意的保护摩擦,不是 CID 数据库真源。
- **链下轴(改=reconcile/编辑)**:CID 系统中的运营展示名,字段仍只能是 `cid_full_name / cid_short_name`。

**admins 非硬保护**:真源=链上 `admins-change::AdminAccounts`(service.rs:577-586),china admins 仅创世种子/止血兜底,改管理员走治理。⇒ 创世轴不含 admins。

**WASM 绑定现状(要扩展不是移除)**:CB(国家储委会/省储委会)/CH(省储行)被生产 pallet 读 `.main_account`,整数组含名字进 WASM⇒改名已吃 setCode;但 ZF/LF/SF/JC/JY 运行期无 pallet 触达,数组不进生产 WASM(`pub const` 未被编译进的代码引用→globaldce 裁掉)⇒改名不动哈希。**保护不一致,要补齐成全部内置统一吃 setCode。**

## 设计(用户定稿,随 T3/T4 一并做)

**① china_*.rs 结构体补完整名称字段**(紧贴 `cid_full_name` 之后):built-in 中文全称/中文简称/英文全称/英文简称都成常量数据。built-in 不再通过字符串匹配推导名称(直接读字段)⇒ **匹配臂全删,国家储委会/4联邦局/省两院 bug 类根除**,简称也一起受保护。

**② 内置名摘要锚点(✅ 定稿就此方案,不做返回目录变体)**:
- runtime 折叠所有 CHINA_*(中文全称+中文简称+英文全称+英文简称)字节 → `builtin_institution_name_digest() -> [u8;32]`,经 `runtime/src/apis.rs` 的 `BuiltinInstitutionNameApi` 暴露(单点引用即强制编译进 WASM)。
- 改任一内置名 → 摘要变 → WASM blake2 变 → **活链必须 setCode**;统一覆盖 ZF/LF/SF/JC/JY/CB/CH,不再看碰巧哪个 pallet 引用哪数组。
- 摘要是 const-eval 32 字节,名字字符串 eval 后被裁剪 → **名字不进链上状态、不触发重新创世(setCode 轴非创世轴)**,仍是链下数据。用户确认:此机制已足够保护重要机构名不被随意改。
- (已否决:runtime API 直接返回目录的"链上可读"变体——不要,名字保持链下。)

**③ 两层(✅ 定稿:只保护现有 china_*.rs 成员,不扩)**:
- **Tier 1(内置,改链上承诺名要 setCode)= 凡在 china_*.rs 的常量机构**(总统府/10部委/5联邦局/两院+监察/教育/国家储委会/CB 省储委会/CH 省储行/LF·SF·JC 省两院监察)。
- **Tier 2(其他,全部 CID 系统自由改)= 后端模板区划机构(省厅/市局/镇政府)+ 用户注册**。**省厅不提升、不搬进 china_*.rs。**
- **关键语义(用户最终确认)**:Tier 1 机构在 CID 系统**照样能改全称/简称,改了只在 CID 系统(链下)生效**;要让区块链侧也变,必须改 china_*.rs 常量 → runtime 升级。`china_*.rs` 是 runtime 保护锚,CID `subjects.cid_full_name/cid_short_name/cid_full_name_en/cid_short_name_en` 是链下运营展示数据。两者不是第二套命名,但全仓字段名和生成逻辑只能使用这四个字段,不得出现旧展示列作为机构名称别名。

**subjects 旧展示缓存列清理**:当前旧展示缓存列只是机构展示/搜索缓存,写入值等于 `cid_short_name`,已形成第三命名源。目标态删除机构代码对该列的读写;搜索改查 `cid_number / cid_full_name / cid_short_name`;列表排序按 `cid_short_name, cid_full_name, cid_number`;启动期对既有 PostgreSQL 表执行删除旧展示缓存列的清残留 SQL。

**reconcile/check-gov**:不得再比较或写入 `name`。Tier 1 名称保护锚由 `china_*.rs` 摘要约束;CID 运营展示名由 `subjects.cid_full_name/cid_short_name/cid_full_name_en/cid_short_name_en` 表达。若执行命令目标是“按保护锚重播内置目录”,则写入这四字段;不再通过字符串匹配另造名字。

## 落地结果
1. **链端(citizenchain)**:china_*.rs 各结构体补 `cid_short_name/cid_full_name_en/cid_short_name_en` 字段并填值;加 `builtin_institution_name_digest()` + `BuiltinInstitutionNameApi` runtime API;名称指纹覆盖七个具名常量文件共 294 个机构。
2. **后端(Tier 1)**:删除字符串匹配第二实现,常量机构直接投影 `china_*.rs` 的四个名称字段。
3. **后端(Tier 2)**:模板 `cid_short_name_suffix` 按目标表改造,`GOV_TEMPLATE_VERSION` 已升级,目录 hash 能触发 strict 检查。
4. **数据库结构**:启动期删除旧展示缓存列;搜索与排序只查 `cid_number / cid_full_name / cid_short_name`。
5. **前端/移动端/冷钱包**:机构名称统一为 `cidFullName/cidShortName/cidFullNameEn/cidShortNameEn`(JSON/API 仍为 `cid_full_name/cid_short_name/cid_full_name_en/cid_short_name_en`);生成器和生成物同步改造。
6. **查重接口名**:旧查重路径与旧前端函数已改为 `/api/v1/institution/check-cid-full-name` 与 `checkCidFullName`。

## 关键文件:行
- `citizencode/backend/gov/service.rs` 常量投影 / 模板 PROVINCE:144-211 CITY:213-310 TOWN:312-343 / mismatch 比对
- 链端常量:`citizenchain/runtime/primitives/china/china_{zf,lf,sf,jc,jy,cb,ch}.rs`(补齐名称四字段)/ `core_const.rs:40-46,89`(OP 派生)/ `runtime/src/apis.rs`(挂名称摘要 API)
- 省级常量全称待核对:`china_{lf,sf,jc}.rs`(省两院带不带「联邦」)

## 1.1 验收记录(2026-06-22)
- 常量库完整性脚本:通过,7 个具名常量文件共 296 个机构全部具备 `cid_full_name / cid_short_name / cid_full_name_en / cid_short_name_en` 四字段。
- 旧 runtime API/旧二字段命名扫描:通过,目标文件内未发现旧二字段 API 名称残留。
- 第二套英文字段名扫描:通过,目标文件内未发现 `english_name` 等替代字段承载机构英文名。
- `node scripts/generate_citizenapp_governance_registry.mjs`:通过,重新生成公民端和公民钱包 87 个治理机构。
- `cargo test --manifest-path citizenchain/runtime/primitives/Cargo.toml builtin_institution -- --nocapture`:通过。
- `cargo check --manifest-path citizenchain/runtime/Cargo.toml`:通过。
- `flutter analyze` in `citizenapp`:通过。
- `flutter test test/governance/admins-change/institution_admin_service_test.dart test/governance/governance_list_page_test.dart` in `citizenapp`:通过。
- `flutter analyze` in `citizenwallet`:通过。

## 1.2 验收记录(2026-06-23)

- 命名规范文件 `memory/07-ai/institution-naming.md`:已补常量补充机构 NSN/NRP/FDA/NGB/ARM/NAV/AIR/SPF/JOS/ARC/NVC/AFC/SFC/NGC,以及省/市/镇模板机构的中文全称、中文简称、英文全称规范、英文简称规范和行政区代码字段。
- 省代码表校验:通过,`memory/07-ai/institution-naming.md` 43 个省代码与 `citizencode/backend/china/china.sqlite` 的 `provinces.code` 全量一致。
- 后端模板:已统一 `PSN/PRP/CGOV/CEDU/TGOV` 的简称模板,`GOV_TEMPLATE_VERSION` 升级为 `gov-deterministic-v7`。
- 对账性能修正:省/市 scope 在目标生成阶段生效,`china.sqlite` 哈希改为进程内缓存,避免 `reconcile-gov --changed-only` 逐省重复全量生成和重复哈希。
- 残留修正(2026-06-27 修订):公安局 `CPOL` 已废弃历史专用种子,统一并入普通市级公权机构模板;清理旧 helper 作用域残留后用当前源码重新编译通过。
- `cargo check --manifest-path citizencode/backend/Cargo.toml`:通过。
- `cargo test --manifest-path citizencode/backend/Cargo.toml gov -- --nocapture`:通过,5 个 gov 相关测试通过。
- 真实库 `reconcile-gov --changed-only`:通过,`scopes=33 inserted=0 updated=174947 account_inserted=349927 removed=0`。
- 真实库 `check-gov --strict`:通过,`ok=true manifest_current=true target_count=245016 active_count=245016 missing=0 mismatched=0 missing_accounts=0 obsolete=0`。
- 真实库 SQL 抽样: `PSN/PRP/CGOV/CEDU/TGOV` 的 `cid_full_name = cid_short_name` 均为 0;旧简称尾缀均为 0。
- 本地 CID HTTP 运行态验证:临时启动 `127.0.0.1:8901`,`/api/v1/app/institutions/search` 返回 `瑶海市公民教育委员会 / 瑶海市教委会`、`明光路镇自治政府 / 明光路镇政府`;`/api/v1/app/public-institutions` 返回 `province_code=AH city_code=001` 与 `CEDU` 的新简称。验证后已关闭临时服务。
- 白皮书 `docs/《白皮书》.md` 与公民宪法 `citizenchain/runtime/primitives/src/旧宪法 HTML`:仅扫描待统一项,未修改;需用户二次确认后执行。

## 验收记录(2026-06-22)
- `cargo test --manifest-path citizencode/backend/Cargo.toml`:通过 71 单元 + 5 integration。
- `cargo check --manifest-path citizencode/backend/Cargo.toml`:通过。
- `npm run build` in `citizencode/frontend`:通过;仅 Vite chunk size warning。
- `flutter analyze` in `citizenapp`:通过。
- `flutter test` in `citizenapp`:通过(4 个 native 相关测试按既有条件 skipped)。
- `flutter analyze` in `citizenwallet`:通过。
- `cargo test --manifest-path citizenchain/runtime/primitives/Cargo.toml builtin_institution -- --nocapture`:通过。
- `cargo check --manifest-path citizenchain/runtime/Cargo.toml`:通过。
- `scripts/sync-derive-vectors.sh --write`:通过,同步 Rust/citizenapp golden。
- 真实库 `reconcile-gov --changed-only`:通过,`scopes=43 inserted=245016 updated=0 account_inserted=490077 removed=245016`。
- 真实库 `check-gov --strict`:通过,`ok=true manifest_current=true target_count=245016 active_count=245016 missing=0 mismatched=0 missing_accounts=0 obsolete=0`。
- 临时启动当前 CID 后端 `127.0.0.1:8898`:健康检查 200;`/api/v1/app/institutions/search` 与 `/api/v1/app/public-institutions` 返回 `cid_full_name/cid_short_name`。
- 残留扫描:未发现旧展示缓存列、旧简称字段、旧查重接口/函数或旧中文命名等机构全称/简称残留。`InstitutionNamed` 仅属于账户派生中的“自定义账户名”枚举,不是机构全称/简称字段。
