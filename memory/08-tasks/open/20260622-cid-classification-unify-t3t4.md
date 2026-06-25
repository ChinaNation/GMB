# 全仓库机构分类统一为 CID 号机构码（T3/T4 单源，删 org_code + 链 ORG_xx）

## 状态

> **✅ 全栈代码完成并验证（2026-06-22）——仅剩用户部署重新创世。** 详见 [[ADR-025]]。
> Phase 1（subject_property）+ Phase 2（org_code）+ Phase 3（链端 ORG_xx + china re-bake + 客户端线格式）全部落地。
> 链端 `cargo test --workspace` ~695 测试 0 失败；node 174；冷钱包 81/81；热钱包 88+`analyze` 0；后端 76；前端 `tsc` 0。
> **全产品旧分类残留 grep = 0**（链端旧 u8 分类、后端旧分类列、前端旧短码、Dart 旧分类字段）。
> 残留命名清理完成：`ProposalsByOrg`→`ProposalsByCode` + 4 error 名 `*Org*`→`*Code*`；阈值/票数/人数硬编码改派生 `primitives::count_const`；citizencode 建机构表单旧短码→新码（GOV `ZF/LF/SF/JC`→`CGOV/CLEG/CJUD/CSUP`、非法人→UNIN、教育按公私×级别→GUN/SUN/GSCH/SFSC、分校→UNIN、私权→SFxx）；机构码中文名统一来自 `cid_short_name`。
> **剩**：① 重新创世（重生 `citizenchain.raw.json`+出 deb+重启 6 节点+重跑公权数据包/机构注册表生成器，**用户部署**，china/账户/代码就绪）② 提交（待授权）。
> 工程脚本：`scripts/rebake_china_codes.py`（china cid_number 重烤，282 内建）+ `scripts/gmb.py --apply`（账户重派生）。

**Phase 1 后端 完成 + 二轮整改（2026-06-22）。** `cargo check` 0 错误 0 警告,76 测试全过,**零旧码残留(单+双引号+注释+文档全清)**,26 文件,未提交未推送。

二轮整改(用户 5 点):
1. `number/code.rs` 改为 `primitives::code` 薄封装;全仓库机构码唯一真源收敛到 `citizenchain/runtime/primitives/src/code.rs`,number/ 继续负责 CID 号生成、解析、校验。
2. 大学改 3 位:GUN(公立)/SUN(私立);放开 3 位布局盈利位 0/1(私立大学可变,校验仍 mod-36 全强度);大学联邦+市注册局都可创建(教育绕过层级门自动满足)。终态:3 位=国家/省部+公私大学+国家教委;4 位=市镇+私权+市教委+初小中
3. 镇级补全部门码(92 码总):TDEF/THSC/TCOM/TENR/TTRN(国防/国安/商贸/能源/交通科),具体启用由市注册局管理员运行期增删(不进自动模板)
4. **16 处 SQL 单引号残留全清**(我首轮漏了单引号):gov 列表排除全部教育码(NED/CEDU/GUN/SUN/GSCH/SFSC,进独立教育 tab)、排序 CASE 改分支秩、admin 父级搜索改 UNIN 通用模型(学校同市∪非校 S 全国∪非校 G 按层级)、model.rs/db.rs/main.rs/public_institution 全清
5. K1 残迹清:删 `SubjectProperty::from_str`(M|公民 解析),registration 主体属性一律从机构码派生,derive_category 改 (code,name) 内部派生

**subject_property(旧 K1)已彻底消除(三轮整改 2026-06-22)**:删 SubjectProperty 枚举 + DB 列(subjects/private 两表) + ~50 处 SQL/DTO/结构体/参数;全部由机构码派生(InstitutionCode 新增 is_person/is_public_legal/is_private_legal/is_unincorporated;classify(institution_code,cid_short_name);uninorg 函数改吃 institution_code)。SQL 过滤映射:`=G`→category IN(GOV/PUBLIC_SECURITY)、`IN(S,F)`→category=PRIVATE、`=F`→institution_code IN(SFGT/SFGP/UNIN)、`=S`→institution_code IN(私法人码)。已独立核验 INSERT 占位符/绑定数 + reader↔SELECT 列序对齐无错位。全 crate subject_property 残留=0。number/admin.rs 旧主体属性下拉也已删。**0 错误 0 警告,76 测试,29 文件,未提交。**

**待续**:~~org_code 列(同类活,Phase 2)/ Phase 3 链端去 ORG_xx + china CID 重烤(重新创世)~~ → **已全部完成（见顶部状态横幅 + [[ADR-025]]）**。

历史(基础阶段)已通过编译 + 70 单测:
- ✅ `number/` 全模块重写：`code.rs`(引用 `primitives::code` 的 92 码 + 盈利策略 + 主体属性派生 + ALL) / `validator.rs`(双布局 index3 分流 + mod-36/10/26 校验 + 新 CidNumberParts) / `generator.rs`(删 K1、按码盈利、N9 去 K1、双布局) / `category.rs`(classify 改用 Cpol 码)
- ✅ gov 赋码：省厅 11 + 市级 16 + 镇级 5 模板 institution_code 全改新码;CITY_POLICE→CPOL;国家两院 NSN/NRP;`parse_cid_institution_parts` 改用新格式派生
- ✅ 消费方:3 个 generate_cid_number 调用点删 subject_property;binding 公民→CTZN;`default_account_names_for_codes` CB/CH→NRC/PRB;`derive_category` 测试改新码
- ✅ 私权模块 `private/{common,sole,welfare,association,corporation,partnership,company}`:institution_code/identity_code GT/GP/LP/GQ/GF/GY/AS → SFxx
- ✅ `number/admin.rs` institution_options 改为 primitives 92 码单源派生

**教育码缺口已解决（当前口径）:** 92 码总表包含 `GUN` 公立大学、`SUN` 私立大学、`GSCH` 公立学校、`SFSC` 私立学校、`JUN` 教会大学、`JSCH` 教会学校;初/小/中三级靠 `education_type` 字段区分,大学独立码;非法人学校复用 UNIN。

**历史 Phase 1 剩余已清理:** 教育注册、UNIN 从属模型、旧短码、旧公权判定和旧私权码残留均已并入后续阶段和当前 92 码表,不再作为本任务待办。

**后续 Phase（~~未开始~~ → 已完成 2026-06-22）:** ~~Phase 2 删 org_code 列 + 全消费方;Phase 3 链端去 ORG_xx + china_*.rs CID 重烤(重新创世) + 重跑公权数据包生成器~~ → 全部落地，见顶部状态横幅 + [[ADR-025]]。之后机构全称/简称统一为独立 follow-up（[[project_institution_name_single_source_2026_06_21]]）。

## 任务需求

把全仓库**机构分类**收敛为**唯一真源 = CID 号里的机构码**（`citizencode/backend/number/code.rs`），删除所有平行分类：
- 删后端 `org_code`（~62 串）——消费方改读 机构码 + 省/市/镇码 + 名字
- 删链端 `ORG_NRC..ORG_OTH` 整套枚举——链上分类字段改存机构码/由其派生的档位
- 删 K1 主体属性段（折进机构码，码自带公/私/个人语义）
- 个人多签不发号，仅一个分类码取代 `ORG_REN`
- 之后再做机构全称/简称统一（独立后续任务）

前置事实（已多智能体验证）：机构名是链下数据，改类/改名**不需 setCode**；但改 CID 号 = 改 `china_*.rs` 常量 = 派生账户变 = **重新创世**（用户已同意，现 pre-genesis 无迁移）。

## 锁定的编码方案

### 号码新格式（总长 26 位不变，段二恒 5 字符）
```
旧:  R5(5) - K1 T2 P1 C1 (5) - N9(9) - D4(4)      段二 = 主体属性1 + 机构码2 + 盈利1 + 校验1
新A(国家/省部): R5 -  XYZ  P  C  - N9 - D4         3位机构码 + 盈利位(恒0) + 校验(mod-36)
新B(其他):      R5 -  WXYZ    M1 - N9 - D4         4位机构码 + M1(类型=盈利,值=校验)
解析分流: 段二 index 3 = 数字 ⇒ A 布局; = 字母 ⇒ B 布局
```
- A 布局校验 = `cid_checksum` mod-36 over `R5+码(3)+盈利(1)+N9+D4`
- B 布局 M1：盈利⇒数字(校验 mod-10)，非盈利⇒字母(校验 mod-26)；over `R5+码(4)+N9+D4`
- N9 哈希元组去掉 K1 → `(pubkey, 机构码, province, city, year)`
- 盈利已接受由 mod-36 降级到 mod-10/26（仅 B 布局）；A 布局保持 mod-36

### 完整码表（77 个，字母已确认锁定，不可变）
**A 国家级单体（26，3 位，公法人，盈利位恒 0）**
PRS 总统府 / FSC 联邦安全局 / FIB 联邦情报局 / FSS 联邦特勤局 / FPR 联邦人事局 / FRG 联邦注册局 / MFA 外事交流部 / MDF 国家防务部 / MHS 国土安全部 / MCW 公民生活保障部 / MHU 住房与城镇建设部 / MAG 农业与农村发展部 / MCM 商务与市场贸易部 / MFT 财政与税务部 / MEN 能源与环保发展部 / MTR 交通运输部 / NLG 国家立法院 / NJD 国家司法院 / NSP 国家监察院 / FAC 联邦廉政署 / FAU 联邦审计署 / FIV 联邦调查署 / NED 国家公民教育委员会 / **NRC 国家公民储备委员会(→NRC 档)** / NSN 国家参议会 / NRP 国家众议会

**B 省级类型（17，3 位，43 省共用，R5 省码区分，盈利位恒 0）**
PGV 省政府 / PLG 省立法院 / PJD 省司法院 / PSP 省监察院 / **PRC 省储会(→PRC 档)** / **PRB 省储行(→PRB 档)** / PDF 防务厅 / PHS 国安厅 / PCW 民生厅 / PHU 住建厅 / PAG 农业厅 / PCM 商贸厅 / PFT 财税厅 / PEN 能源厅 / PTR 交通厅 / PSN 省参议会 / PRP 省众议会

**C 市级类型（17，4 位，非盈利）**
CGOV 市政府 / CLEG 市立法委 / CSUP 市监察院 / CJUD 市司法院 / CEDU 市教委 / CSLF 市自治委 / CDEF 国防局 / CHSC 国安局 / CCWF 民生局 / CHUD 住建局 / CAGR 农业局 / CCOM 商贸局 / CFIN 财税局 / CENR 能源局 / CTRN 交通局 / CREG 市注册局 / CPOL 市公安局

**D 镇级类型（5，4 位，非盈利）**
TGOV 镇政府 / TCWF 民生科 / THUD 住建科 / TAGR 农业科 / TFIN 财税科

**E 私权机构（7，4 位，SF 前缀）**
SFGT 个体经营(盈利) / SFGP 无限合伙(盈利) / SFLP 有限合伙(盈利) / SFGQ 股权公司(盈利) / SFGF 股份公司(盈利) / SFGY 公益组织(非盈利) / **SFAS 注册协会(可盈利可不,按实例)**

**F 个人主体（3，4 位）**
CTZN 公民人(盈利) / NATP 自然人(盈利) / **SMTP 智能人(可盈利可不,按实例)**

**G 非法人组织（1，4 位）**
**UNIN 非法人组织** —— 挂父级法人，盈利**完全继承父级**（原 uninorg p1 继承规则）

**H 个人多签（1，4 位，不发号）**
PMUL —— 仅链上/后端分类常量，不进 CID 号

### 盈利策略（机构码 → 盈利）
- A/B 公权（国家/省部/市镇公权）：非盈利（A 布局盈利位 = 0；C/D 在 B 布局 M1 = 字母）
- SFGT/SFGP/SFLP/SFGQ/SFGF、CTZN/NATP：固定盈利（M1 = 数字）
- SFGY：固定非盈利（M1 = 字母）
- SFAS、SMTP：按实例可变（M1 类型即权威）
- UNIN：继承父级法人盈利属性

### 链端档位映射（取代 ORG_xx，单一映射函数）
NRC→NRC 固定档 / PRC→PRC / PRB→PRB / 其余公法人单体+省市镇公权→动态公权档(原 PUP) / SFxx→动态私权档(原 OTH) / PMUL→个人档(原 REN)

## 当前关键文件

- `citizenchain/runtime/primitives/src/code.rs`:国家码、省级行政区码、92 个机构码、`cid_short_name`、盈利策略、行政层级和治理谓词唯一真源。
- `citizencode/backend/number/code.rs`:primitives 薄封装,供 CID 号生成、解析、校验引用。
- `citizencode/backend/number/generator.rs`:双布局 CID 号生成。
- `citizencode/backend/number/validator.rs`:双布局解析、校验位计算和 `CidNumberParts`。
- `citizencode/backend/number/category.rs`:按机构码派生主体分类。
- `citizencode/backend/china/store.rs`:校验 SQLite 省表与 primitives 省表一致,市镇继续从 SQLite 加载。
- org_code 真源 `citizencode/backend/gov/service.rs:670-709` + 模板:144-343

## 待处理发现:CID 种子约定 + 撞号重试应收进 number/(2026-06-22，另一线程核查)

**问题:** `number/generate_cid_number` 把 `account_pubkey` 当不透明输入；而**确定性、可复现的种子直接决定最终 CID**，却散在 number/ 外：
- gov `GOV-{scope}-省码-市码-镇码-机构码`（`gov/service.rs:689`，确定性，无重试）
- 公民绑定 `wallet_pubkey` [+ `#retry`]（`citizens/binding.rs:824`，确定性身份派生，自带 1000 次 DB 查重循环）
- 动态注册 随机 UUID（`subjects/registration.rs:407`，纯熵，自带 1000 次循环）

generator.rs 注释还明写"调用方 1000 次重试逃逸碰撞" = 把重试策略也推到 number/ 外。**当前 = 3 套种子约定 + 2 份重复重试循环散落。**

**做法（彻底单源）:** number/ 拥有"决定一个号的全部" = 种子约定 + 撞号重试，对外按用途暴露构造器：
- `number::official_institution_cid(scope, 省码, 市码, 镇码, 机构码, exists_fn)`（确定性 GOV 种子；T3/T4 china 重烤 + federal 常量种子也走它）
- `number::citizen_cid(wallet_pubkey, 省, exists_fn)`（确定性身份种子 + #retry）
- `number::dynamic_institution_cid(省, 市, 机构码, p1, exists_fn)`（随机熵 + 重试）

gov / binding / registration 改成只调这些。**唯一边界例外:** DB 查重不能进 number/（保持存储无关），用回调 `exists_fn` 传入——number/ 拥有重试循环，调用方只提供查重谓词（依赖倒置）。

**顺带 doc rot:** `number/generator.rs:5-9` 头注释列调用方 `cpms/subjects/citizens::binding/core::runtime_ops` 已过期，实际 = `registration/binding/gov`（列了不存在的 cpms/runtime_ops、漏了 gov），整改时一并修。

## 阻塞与协调
- 并行线程 dirty：`china_cb/jc/lf/sf.rs` + `gov/service.rs` + `citizenwallet payload_decoder_test.dart`。阶段 2/3 必须先等其提交或协调，否则硬冲突。
- 阶段 1 (number/) 不碰这些文件，可立即开工。
- federal 常量 CID 种子约定 + 上面的 `number::official_institution_cid` 收敛，都在 Phase 3 china 重烤一并落（决策 B：CID 域归本卡）。
