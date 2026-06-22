# 全仓库机构分类统一为 CID 号机构码（T3/T4 单源，删 org_code + 链 ORG_xx）

## 状态

设计已锁定（2026-06-22，用户逐项确认）。实施未开始；当前阻塞点：并行线程仍在改 `citizenchain/runtime/primitives/china/china_*.rs` + `citizencode/backend/gov/service.rs`，链/gov 阶段须先协调。后端 `number/` 阶段不冲突，可先行。

## 任务需求

把全仓库**机构分类**收敛为**唯一真源 = CID 号里的机构码**（`citizencode/backend/number/institution_code.rs`），删除所有平行分类：
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

## 实施顺序

1. **后端 number/ 重写（不冲突，可先行）**：`institution_code.rs`(77 码 enum + from_str/as_code/label_zh + 盈利策略 + 档位映射) / `generator.rs`(删 K1、双布局、N9 去 K1、盈利按码、M1 合并) / `validator.rs`(双布局解析 + index3 分流 + 校验) / `category.rs`(classify 改由新码派生)。TDD。
2. **删 org_code（含 gov/service.rs，须等并行线程）**：消费方切 机构码+地区+名字；删 `subjects.org_code`/`gov.org_code` 列；删前端 `ORG_CODE_LABEL`。
3. **链端去 ORG_xx（重新创世，须等并行线程）**：删 `votingengine/types.rs` ORG 常量；分类字段存机构码/派生档；`china_*.rs` 全部 CID 重烤新机构码。
4. **重新创世收尾**：重跑公权机构数据包生成器（否则地址/余额/admins 断）。
5. **后续独立任务**：机构全称/简称统一。

## 关键文件:行（真实代码核对）
- `citizencode/backend/number/institution_code.rs` 现 16 码 enum:29-46 / from_str:50-70 / as_code:73-92 / label_zh:95-114
- `citizencode/backend/number/generator.rs` K1→T2 约束矩阵:65-111 / N9 元组:141-153 / 段二拼装:154-157 / p1 逻辑:65-69
- `citizencode/backend/number/validator.rs` 段长常量:31-35 / 双段切片:100-104 / 校验 cid_checksum:49-56 / payload:124-127
- `citizencode/backend/number/category.rs` SubjectProperty:20-72 / classify:116-136
- 链端 ORG 常量 `citizenchain/runtime/votingengine/src/types.rs:16-27` / 阈值 :44-56 / `admins-change/src/lib.rs` org 字段:118 ensure_account_kind_matches_org:594-616
- org_code 真源 `citizencode/backend/gov/service.rs:670-709` + 模板:144-343

## 阻塞与协调
- 并行线程 dirty：`china_cb/jc/lf/sf.rs` + `gov/service.rs` + `citizenwallet payload_decoder_test.dart`。阶段 2/3 必须先等其提交或协调，否则硬冲突。
- 阶段 1 (number/) 不碰这些文件，可立即开工。
