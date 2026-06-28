# ADR-028 公民 tab 公权机构统一(五子 tab + 统一机构层)

## 标题

CitizenApp 公民 tab 从三子 tab 重构为五子 tab(广场/立法/选举/治理/公权),公权与治理两套并行实现合并为「一套机构模型 + 一套详情页 + 一套链态服务 + 一套目录仓库」,子 tab 退化为 `institution_code` 过滤视图;选举走机构自组织提案(不建选举委员会机构),自治会改非法人挂政府。

状态:草案(待 review)。本 ADR 只定方向与边界,不含实现;落地按任务卡 `20260624-citizen-tab-5section-ui`(UI 设计窗口)+ 后续执行任务卡逐张进行。

## 背景

### 痛点

- 公民 tab 现为三子 tab `公权 / 广场 / 治理`(`citizenapp/lib/citizen/citizen_tab_page.dart`),其中:
  - 公权 = `PublicPage`(CID-BFF + Isar 目录,省→市→机构 地理浏览,机构详情页接入统一提案主体能力表;管理员激活复用统一管理员列表)。
  - 治理 = `GovernanceListPage`(静态烘焙注册表 `kNationalCouncil/kProvincialCouncils/kProvincialBanks`,账户 hex 写死,链上读 admins/提案/投票/余额)。
  - 广场 = `VoteView`(全局 NRC/PRC/PRB 提案投票流)。
- 公权与治理是两套并行实现:两套机构模型(`PublicInstitutionEntity` vs `InstitutionInfo`)、两套详情/账户/管理员页,靠注释保持"看起来一样",改一处要改两处、必然漂移。
- `institution_code` 已在公权 DTO/entity 上但 UI 从不消费(连 admin 查询都硬编码 `'CGOV'`);治理的静态注册表与 `public_provinces.dart` 互相硬耦合(省名/省码复用)。
- 机构分类唯一真源 = CID `institution_code`(92 码表,`citizenchain/runtime/primitives/cid/code.rs` + Registry `cid` 模块),后端 `main.rs` 已按机构码分支分组(立法/司法/监察/储备/行政),但 app 端没用上。

### 权威依据

- 公民宪法**已迁为结构化链上法律**(ADR-027,2026-06-24 完成):`CitizenConstitution.html` 是仓库内创世宪法生成源,`constitution.scale` 由脚本生成并作为 `legislation-yuan` 的 `law_id=0, tier=宪法` 创世注入,`ImmutableManifest` 冻结 8 条不可修改条款。节点运行态真源是链上 law_id=0;节点展示走 `constitution_getDocument` RAW storage RPC,普通法律浏览可走 `LegislationApi`。第八条「一府两会三院」:政府 / 公民教育委员会 / 公民储备委员会 / 立法院 / 司法院 / 监察院,相互无隶属、职权独立。
- 第十条/四十四条/七十三条/一百一十八条:国家公民教育委员会负责所有教育类法案草案的起草与初审 → 立法职能 → 归立法 tab。
- 第四十四条二款:总统选举由总统府组织,不再设「国家立法院选举委员会」。
- 第六十条/六十六条:市政府设市公民自治委员会(市自治会)、镇政府设镇公民自治委员会(镇自治会),民选监督机构。
- ADR-027(立法院模块):立法投票走 `legislation-vote` sub-pallet；选举投票走 `election-vote` sub-pallet,本 ADR 不重复造投票流程。
- ADR-018:CitizenApp 轻节点统一查询低负载规则(列表短 key、批量、缓存)。

## 决策

### 1. 公民 tab 五子 tab

`广场 / 立法 / 选举 / 治理 / 公权`。其中「治理」专指区块链/货币治理(储备体系),**与宪法「自治政府」无关**(术语澄清,见决策 8)。

### 2. 统一机构层(核心)

所有机构本质都是按 `institution_code` 分类的公权多签账户,差异只在权责。合并为:
- 一套机构模型(合并 `PublicInstitutionEntity` 与 `InstitutionInfo`);
- 一套目录仓库(CID-BFF + Isar,本地优先秒开,身份与分类唯一来源);
- 一套链态服务(按机构主账户读 admins/提案/投票/余额);
- 一套详情页(替代现公权/治理两套详情/账户/管理员页);
- 删除治理静态烘焙注册表 —— 治理改为「目录按 `institution_code ∈ {NRC,PRC,PRB}` 过滤」。

子 tab = `institution_code` 过滤谓词(视图),非独立数据管线。

### 3. tab 分组定义(institution_code 过滤)

| tab | 机构码过滤 |
|---|---|
| 立法 | `NLG, NSN, NRP, PLG, PSN, PRP, CLEG, NED`(国家/省立法院含参众议会、市立法会、国家公民教育委员会) |
| 治理 | `NRC, PRC, PRB`(国储会/省储会/省储行) |
| 选举 | 见决策 5 |
| 公权 | 全集(GOV_INSTITUTION + 公安,含上述全部) |
| 广场 | 非机构列表 —— 动态流,见决策 7 |

### 4. 选举走机构自组织提案(Option B),不建选举委员会机构

- 不创建任何「选举委员会」机构。选举 = `election-vote` 引擎的提案,由职位所属或对应机构的管理员依法发起。
- 总统选举由**总统府(PRS)**组织(PRS 已是 `china_zf.rs` 内置机构,自带管理员);宪法第四十四条第二款已改为总统府组织。
- 删除 `CSLF`/`TSLF` 两个法人机构码(92→90),市/镇自治会改为**非法人(UNIN)**从属市政府(CGOV)/镇政府(TGOV);自治会由后端 gov reconcile 生成(非公民自助注册)。
- **选举 ≠ 立法(正交)**:选举 = 电选机构的**管理员/法定代表人**(选人);立法 = 立**法律条文**(立法)。机构按功能归立法/治理/公权,其管理员的选举是「选举活动」,不在选举 tab 重复列机构。
- **选举保留普选 + 互选**:普选由公民按宪法/选举法规定的行政区和职位范围选举；互选由机构现任成员在成员快照内选举院长、主席、参议长、众议长等职位。国家立法院参议员/众议员已确定为省行政区公民普选:各省公民分别在本省省参议会/省众议会现任成员中选出。
- 理由:Option A 需凭空建 ~3,229 个委员会机构 + 新增宪法机构条文且与机构自组织条文矛盾;Option B 零新增机构体、契合「投票职责边界硬规则(业务只发起、引擎管投票)」、与 election-vote 普选/互选双模式一致。

### 5. 选举 tab = 选举活动视图(按行政层级 + 机构),非机构子集

因选举是职位活动而不是机构目录,选举 tab 不再是机构子集,而是**选举活动视图**:按行政层级(国家/省/市/镇)和机构分组,每条 = 某职位的一场普选或互选(候选人/投票窗口/计票/当选)。机构本体在公权(及立法/治理功能视图)浏览,不在选举 tab 重复。链端 `election-vote` 已接入普选/互选框架;本窗口选举活动流仍是前端占位,待业务规则解释、选举法细则和 admins 权限写入接入后再展示发起入口。

### 6. 统一详情页规格

- 顶部标题 = 机构**简称**;右上角 = 关注图标。
- 信息卡只显:全称、身份 ID(内含机构码)、主账户、主账户余额、法定代表人、所属地;**非法人加一行「所属上级法人全称」**。
- 机构账户 = 该机构所有账户;管理员 = 该机构所有管理员列表;提案列表 = 该机构发起的所有提案。
- 提案入口 = 该主体拥有的提案权限,由 CitizenApp `ProposalSubject + ProposalCapabilityRegistry` 集中判断。机构码仍参与规则,但页面不得散落 `NRC/PRC/CREG` 判断;基础动作按主体类型和管理员模块路由,治理扩展动作按链端真实能力限定;runtime 未启用的能力不展示,不做假置灰入口。

### 7. 广场:订阅 + 地区 + 管理员 动态流

广场 = 三类机构的动态流并集:① 用户关注(订阅)的机构;② 用户本地区(CID 省/市)的机构;③ 用户钱包是其管理员的机构(复用现有 `ProposalContextResolver.isInstitutionAdmin` / adminWallets,治理动态已有此能力)。新用户空态。通用事件模型(转账/管理员变更/激活等)后置,v1 先复用提案模型。

### 8. 能力层方向(链端,本窗口不实现)

链端长期方向仍是制度能力表收敛,但 CitizenApp 当前先落前端能力注册表:输入是 `ProposalSubject`(个人多签/创世治理机构/公权机构/私权机构/非法人机构)而不是裸机构码;输出是可展示的提案能力。runtime 仍是最终权限边界,`OffchainTransaction`、`OnchainIssuance`、`ElectionVote` 未启用时前端不得展示可发起入口。非法人机构码本身不能决定 public/private 管理员模块,必须由 CID 注册归属显式路由。

### 9. 术语澄清

tab「治理」= 区块链/货币治理(NRC/PRC/PRB);≠ 宪法第八条「自治政府」。自治会民选,归选举 tab。文档与代码注释须写明此区分。

## 边界(本窗口 OUT,走 ADR-027 轨道 + 后续执行任务卡)

- `election-vote` 已接入普选/互选框架;OUT 调整为选举法规则解释、业务模块创建封装、admins/法定代表人结果写入和能力层 `is_action_allowed` 实现;
- 宪法修改(总统府组织总统选举 + 国家立法院参众议员改省行政区公民普选 + 选举法承接细则)已直接更新 HTML 真源并重新生成 `constitution.scale`;后续普通修宪仍走 law_id=0 的 `propose_amend_law`。
- 后端删 CSLF/TSLF 码表 + 自治会 UNIN 注册 + purge;
- CitizenApp 实际编码。

以上任何 `citizenchain/runtime/` 改动均先发起 runtime 二次确认。

## 影响

- 正面:删一套并行实现、激活死字段 `institution_code`、过滤为 Isar 索引查询近零成本、广场更聚焦本地区、宪法/机构体大幅简化。
- 风险:统一详情页要调和公权(只读占位)与治理(完整能力)的能力差,依赖能力层先行;`public_provinces.dart` 对治理注册表的硬耦合要解;缓存 shape 改语义需 bump 版本;选举/能力层落地属重新创世 bake 范畴。
