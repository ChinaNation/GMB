# 机构全称/简称统一(单一真源 + 拆开 短==全)

## 状态

**设计锁定(2026-06-22,用户逐项确认),执行未开始。** 执行**并入 [T3/T4 分类整改](20260622-cid-classification-unify-t3t4.md) 同一次 reconcile / 重新创世**,gated 在 T3/T4 把 `citizencode/backend/gov/service.rs` push 之后(现两线程同改该文件同结构体,先改会硬冲突)。名字是链下数据,不需 setCode;但既然 T3/T4 反正要重烤 CID + reconcile,名字对修正并进同批,不开第二轮。

## 背景(reconcile 后开发库终态)

PUBLIC 共 245,016,其中 `cid_short_name == cid_full_name` = 50,795,结构 = **5 真 bug + 50,661 模板故意短==全 + 129 本就最短**。

- 名字真源已是单源形态(符合 [机构名单一真源 2026-06-21](../../../.claude 见 auto-memory)):常量机构走 `official_name_pair(name)` 产出 `(full, short)`;模板机构走模板结构体 `suffix`/`full_suffix`;reconcile 写穿 `subjects.{cid_full_name, cid_short_name}`;所有 UI/auth 只读这两列。**不重新设计架构。**
- **病根**:`official_name_pair` 与 `org_code_for_constant_name` 都拿 china_*.rs 的中文全名当 match 键(见 service.rs:668-669 历史踩坑注释)。china 改名 → arm 静默落默认臂 → 简称=全称 / org_code=PUBLIC_ORG。国储会、4 联邦局、129 省两院都是这一个病根。

## 已确认决策

1. **4 总统府联邦局简称 = 去前缀**(注释明确「勿擅造」,本值经用户确认):
   - 总统府联邦安全局 → 简称 `联邦安全局`
   - 总统府联邦情报局 → 简称 `联邦情报局`
   - 总统府联邦特勤局 → 简称 `联邦特勤局`
   - 总统府联邦人事局 → 简称 `联邦人事局`
   - (全称保持,与「总统府联邦注册局 → 联邦注册局」一致)
2. **排期**:并入 T3/T4 同次 reconcile(见上「状态」)。

## 锁定的目标名字对

### A. 常量机构(`official_name_pair` 补 arm)
| 机构(china 全名) | 全称 | 简称 | 现状 |
|---|---|---|---|
| 总统府联邦安全局 | 总统府联邦安全局 | 联邦安全局 | 🔴 现落默认臂 短==全 |
| 总统府联邦情报局 | 总统府联邦情报局 | 联邦情报局 | 🔴 同上 |
| 总统府联邦特勤局 | 总统府联邦特勤局 | 联邦特勤局 | 🔴 同上 |
| 总统府联邦人事局 | 总统府联邦人事局 | 联邦人事局 | 🔴 同上 |
| 国家公民储备委员会 | 国家公民储备委员会 | 国储会 | 🟢 service.rs:636/698 当前工作区已修,差 reconcile |

### B. 模板机构(改 `suffix`,`full_suffix` 保持)
| org_code/模板 | full_suffix(保持) | suffix(改为) | 条数 |
|---|---|---|---|
| TOWN_GOV | 自治政府 | 政府 | 39,087 |
| CITY_GOV | 自治政府 | 政府 | 2,872 |
| CITY_EDU | 公民教育委员会 | 教委会 | 2,872 |
| PROVINCE_SENATE_COUNCIL | 参议员议政会 | 参议会 | 43 |
| PROVINCE_REPRESENTATIVE_COUNCIL | 众议员议政会 | 众议会 | 43 |
| CITY_COURT | 司法院 | 司法院(保持,已最短) | 2,872 |
| CITY_SUPERVISION | 监察院 | 监察院(保持,已最短) | 2,872 |

> 注:CITY_COURT/SUPERVISION 的 短==全 是合法终态(已最短),不是 bug,保留。

### C. 省两院 129 条(PROVINCE_LEGISLATURE/COURT/SUPERVISION,常量默认臂)
目标:全称带「联邦」(`X省联邦立法院/司法院/监察院`),简称去「联邦」(`X省立法院/…`)。
- ⚠️ **执行前先核对** china_lf/sf/jc.rs 省级常量实际存的全称:若已是「X省联邦立法院」则只需补 arm 让简称去联邦;若存的是「X省立法院」则全称需补联邦。注意 service.rs:702-704 `org_code_for_constant_name` 的 `ends_with("省联邦立法院")` 暗示常量应带「联邦」,需对齐确认,别盲改。

## 架构原则(2026-06-22 用户定稿 —— 反转早先"移出"方向)

**⚠️ 本节取代早先"把名字移出 china_*.rs"的建议。用户要的是相反:内置重要机构名字就该锁在常量库受保护,改它要 runtime 升级。**

**三条轴(各管各的)**:
- **创世轴(改=重新创世)**:`cid_number` + 由它派生的全部账户(`derive_account(OP_x, ss58, cid_number)`):OP_MAIN/OP_FEE/OP_STAKE(永久质押)/OP_AN(安全基金)/OP_HE(两和基金)+ china_zb 落给它们的创世余额。**单根=cid_number**(改号→全部派生账户平移→余额错位),外加派生原语 `core_const.rs:40-46/89`。
- **runtime 升级轴(改=setCode)**:**内置重要机构名称**(在 china_*.rs)。故意的保护摩擦。
- **链下轴(改=reconcile)**:普通机构(模板/用户)名称。

**admins 非硬保护**:真源=链上 `admins-change::AdminAccounts`(service.rs:577-586),china admins 仅创世种子/止血兜底,改管理员走治理。⇒ 创世轴不含 admins。

**WASM 绑定现状(要扩展不是移除)**:CB(国储会/省储会)/CH(省储行)被生产 pallet 读 `.main_account`,整数组含名字进 WASM⇒改名已吃 setCode;但 ZF/LF/SF/JC/JY 运行期无 pallet 触达,数组不进生产 WASM(`pub const` 未被编译进的代码引用→globaldce 裁掉)⇒改名不动哈希。**保护不一致,要补齐成全部内置统一吃 setCode。**

## 设计(用户定稿,随 T3/T4 一并做)

**① china_*.rs 结构体补 `cid_short_name` 字段**(紧贴 `cid_full_name` 下一行):built-in 全称+简称都成常量数据。`official_name_pair` 对 built-in 不再字符串匹配(直接读两字段)⇒ **匹配臂全删,国储会/4联邦局/省两院 bug 类根除**,简称也一起受保护。

**② 内置名摘要锚点(✅ 定稿就此方案,不做返回目录变体)**:
- `const fn` 折叠所有 CHINA_*(全称+简称)字节 → `BUILTIN_NAME_DIGEST: [u8;32]`,经 `runtime/src/apis.rs` 一行 runtime API 暴露(单点引用即强制编译进 WASM)。
- 改任一内置名 → 摘要变 → WASM blake2 变 → **活链必须 setCode**;统一覆盖 ZF/LF/SF/JC/JY/CB/CH,不再看碰巧哪个 pallet 引用哪数组。
- 摘要是 const-eval 32 字节,名字字符串 eval 后被裁剪 → **名字不进链上状态、不触发重新创世(setCode 轴非创世轴)**,仍是链下数据。用户确认:此机制已足够保护重要机构名不被随意改。
- (已否决:runtime API 直接返回目录的"链上可读"变体——不要,名字保持链下。)

**③ 两层(✅ 定稿:只保护现有 china_*.rs 成员,不扩)**:
- **Tier 1(内置,改链上承诺名要 setCode)= 凡在 china_*.rs 的常量机构**(总统府/10部委/5联邦局/两院+监察/教育/国储会/CB 省储会/CH 省储行/LF·SF·JC 省两院监察)。
- **Tier 2(其他,全部 CID 系统自由改)= 后端模板区划机构(省厅/市局/镇政府)+ 用户注册**。**省厅不提升、不搬进 china_*.rs。**
- **关键语义(用户定)**:Tier 1 机构在 CID 系统**照样能改名,改了只在 CID 系统(链下)生效**;要让区块链侧也变,必须改 china_*.rs 常量 → runtime 升级。⇒ china 常量 = Tier 1 的**链上承诺名(被摘要锚定)**;subjects 那份 = 运营展示名,**两份允许不一致**。

**reconcile/check-gov**:china 仅作 Tier 1 的初始种子 + 链上锚,**不硬锁链下名**(不得断言 subjects==china 否则卡死"CID 系统单独改名")。🟠 执行期小行为点:reconcile 对 Tier 1 是"仅缺失时种入"还是"每次覆盖",默认**不覆盖运营改名**;check-gov 对 Tier 1 名漂移最多 WARN 不 FAIL。

## 落地步骤(T3/T4 push 后)
1. **链端(citizenchain)**:china_*.rs 各结构体补 `cid_short_name` 字段并填值(含 4 联邦局简称去前缀 + 省两院全称带联邦/简称去联邦);加 `BUILTIN_NAME_DIGEST` const fn + `apis.rs` 一行 runtime API。属链改→走 setCode(本就并入 T3/T4 重新创世批次,届时一并生效)。
2. **后端(Tier 1)**:`official_name_pair` 对 built-in 改为直接读 china 两字段,**删字符串匹配臂**;`federal_registry_*` finder 改按机构码/cid_number 定位;`china/mod.rs` 测试文案保留(用 cid_full_name 仅做断言文案,OK)。
3. **后端(Tier 2)**:模板按上表 B 改 7 类 `suffix`(2 类保持)。
4. 升 `GOV_TEMPLATE_VERSION`(现 `gov-deterministic-v6`)→ 否则 catalog_hash 不变,strict 不触发更新。
5. `reconcile-gov --changed-only` + `check-gov --strict`(并入 T3/T4 重新创世同批);Tier 1 名漂移 check-gov 最多 WARN 不 FAIL(不硬锁链下,允许 CID 系统单独改名)。
6. 前端零改动:auth/UI 已读 `cid_short_name` 投影,reconcile 写对后自动生效。

## 关键文件:行
- `citizencode/backend/gov/service.rs` official_name_pair:588-664 / org_code_for_constant_name:670-709 / 模板 PROVINCE:144-211 CITY:213-310 TOWN:312-343 / mismatch 比对:1225-1238
- 链端常量:`citizenchain/runtime/primitives/china/china_{zf,lf,sf,jc,jy,cb,ch}.rs`(补 cid_short_name)/ `core_const.rs:40-46,89`(OP 派生)/ `runtime/src/apis.rs`(挂摘要 API)
- 省级常量全称待核对:`china_{lf,sf,jc}.rs`(省两院带不带「联邦」)

## 阻塞与协调
- 🔴 与 T3/T4 同改 `gov/service.rs` 同结构体,**必须等其 push 后在干净基线改**。
- CI 当前:CitizenWallet 红(等 T3/T4 push)、WASM 红(基础设施 SSH key 被拒,与本卡无关)。
