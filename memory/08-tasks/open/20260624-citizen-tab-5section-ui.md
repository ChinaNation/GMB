# 公民 tab 五子 tab 重构 + 统一机构层(UI 设计窗口)

## 状态

**设计阶段(2026-06-24)。** 本任务卡是「方案架构 + CitizenApp UI 设计」窗口的产出载体。决策见 [ADR-028](../../04-decisions/ADR-028-citizen-tab-public-power-unified.md)。本窗口**只产出可评审的设计与目录/命名方案,不编码、不改 runtime、不实现投票引擎**。执行(链端/后端/Flutter 编码)拆后续任务卡。

## 需求(用户拍板汇总)

1. 公民 tab 三子 tab → 五子 tab:`广场 / 立法 / 选举 / 治理 / 公权`。
2. 公权与治理两套并行实现合并为一套机构模型/详情页/链态服务/目录仓库;子 tab = `institution_code` 过滤视图(ADR-028 决策 2/3)。
3. 选举走直选提案(Option B):不建选举委员会机构,总统选举由总统府发起,删宪法选举委员会描述;删 `CSLF`/`TSLF`,市/镇自治会改非法人挂政府(ADR-028 决策 4/5)。
   - 选举 ≠ 立法(正交):选举=电选机构管理员/法定代表人,立法=立法律条文;机构不在选举 tab 重复列。
   - 废除互选,所有机构管理员/法定代表人改公民分行政区层级普选;护宪大法官归国家立法院(NLG)。← 大宪法改写,OUT。
   - 选举 tab = 选举活动视图(按层级),非机构子集。
4. 统一详情页规格(ADR-028 决策 6):顶部=简称、右上=关注图标;信息卡=全称/身份ID/主账户/主账户余额/法定代表人/所属地(非法人 +「所属上级法人全称」);机构账户/管理员/提案列表/提案入口(按权限点亮)。
5. 广场=订阅 + 本地区(CID 省/市)+ 用户管理员机构 三类动态流并集(ADR-028 决策 7)。

## CitizenApp UI 设计(本窗口核心产出)

### 五子 tab(同一份机构目录按 `institution_code` 过滤,全走 ADR-018 轻节点规则)

| tab | 内容 | 过滤/数据 |
|---|---|---|
| 广场 | 关注 + 本地区 + 我是管理员 的机构动态流 | 订阅表 ∪ CID 省/市 ∪ `isInstitutionAdmin` |
| 立法 | 国家/省立法院(含参众议会)、市立法会、国家公民教育委员会 | `NLG,NSN,NRP,PLG,PSN,PRP,CLEG,NED` |
| 选举 | 选举活动视图:按行政层级(国家/省/市/镇),每条=某机构管理员/法定代表人的选举 | citizen-vote 提案(占位);机构本体在公权浏览 |
| 治理 | 国储会、省储会、省储行 | `NRC,PRC,PRB` |
| 公权 | 全部机构(省→市→机构 地理浏览 + 关注组) | 全集(超集) |

### 统一详情页

顶部:机构简称 + 右上角关注图标。
信息卡:全称、身份 ID(内含机构码)、主账户、主账户余额、法定代表人、所属地;非法人加一行「所属上级法人全称」。
分区:机构账户(全部账户)/ 管理员(全部列表)/ 提案入口(按 `is_action_allowed(机构码,动作)` 点亮:基础 转账/管理员更换/手续费划转 通用,扩展按权限,引擎未就绪置灰占位)/ 提案列表(该机构发起的全部提案)。

### 占位策略(衔接「引擎不在本窗口」)

提案入口结构本窗口设计齐备,但所有依赖投票引擎/能力层的动作渲染为「待引擎/未开放」置灰态(沿用现有公权详情页占位惯例);选举活动流空区占位。本窗口不写任何投票/提案链路。

## 仓库目录结构 + 文件命名(提案,待 review 锁定)

命名遵守:目录 kebab-case(对齐链端 `governance/organization-manage` 等)、组件 PascalCase、文件 snake_case、与链端/用户命名字面一致。统一机构层下沉,五 tab 各一视图目录:

```
citizenapp/lib/citizen/
├── citizen_tab_page.dart              # 五子 tab 壳(改:3→5 tab)
├── institution/                       # 【新增】统一机构层(模型+详情页+目录仓库+链态+分类)
│   ├── institution.dart               # 统一机构实体(合并 PublicInstitutionEntity + InstitutionInfo)
│   ├── institution_classification.dart# institution_code → tab 分组 + 标签(单一源)
│   ├── institution_detail_page.dart   # 统一详情页(替代公权/治理两套)
│   ├── institution_accounts_page.dart # 机构账户页
│   ├── institution_admin_list_page.dart
│   └── data/                          # 目录仓库(CID-BFF+Isar) + 链态服务(由现 public/data 迁入归并)
├── square/                            # 【改名自 vote/】广场:订阅+地区+管理员 动态流
│   └── square_view.dart               # 由 vote_view.dart 重构
├── legislation/                       # 【新增】立法 tab 视图(机构码过滤)
├── election/                          # 【新增】选举 tab 视图 + 选举活动流占位
├── governance/                        # 【新增】治理 tab 视图(NRC/PRC/PRB;注意与顶层 lib/governance/ 区分)
└── public/                            # 公权 tab 视图(保留地理浏览,data/ 迁入 institution/data)
```

注:exact 文件名在执行任务卡 kickoff 时随读源码微调;`citizen/governance/`(治理 tab 视图)与顶层 `lib/governance/`(治理业务模块:organization-manage/admins-change/personal-manage/runtime-upgrade)是两回事,前者只读后者能力,不复制。

## 预计修改 / 新增目录(本窗口仅文档;以下为执行期预判,逐条中文注释)

- `memory/04-decisions/` —【本窗口·新增】ADR-028(决策记录)。文档。
- `memory/08-tasks/open/` —【本窗口·新增】本任务卡。文档。
- `citizenapp/lib/citizen/institution/` —【执行·新增】统一机构层(模型/详情页/账户/管理员/目录仓库/链态/分类)。代码 + 残留清理(合并删除两套旧详情页)。
- `citizenapp/lib/citizen/{square,legislation,election,governance}/` —【执行·新增/改名】四个子 tab 视图(square 由 vote 改名)。代码。
- `citizenapp/lib/citizen/public/` —【执行·改】保留地理浏览,`data/` 迁入 `institution/data/`,解除对治理静态注册表耦合。代码 + 残留清理。
- `citizenapp/lib/citizen/citizen_tab_page.dart` —【执行·改】3→5 tab。代码。
- `citizenapp/lib/governance/` —【执行·改】删静态烘焙注册表与重复详情页,治理 tab 改读统一目录。代码 + 残留清理。
- （OUT,后续卡/ADR-027 轨道,触 runtime 二次确认）`citizenchain/runtime/{votingengine/citizen-vote,primitives/src/code.rs,primitives/src/CitizenConstitution.html}`、`citizencode/backend/{subjects,gov,number}`。

## 范围

- IN(本窗口):ADR-028 + 本任务卡 + 上述目录/命名/UI 设计方案。产出=设计评审。
- OUT(后续):citizen-vote 选举引擎、能力层 `is_action_allowed`、宪法 HTML 改动、后端删 CSLF/TSLF + 自治会 UNIN 注册、CitizenApp 实际编码。每项另立执行任务卡;涉 runtime 先二次确认。

## 验收(本窗口)

- ADR-028 与本任务卡通过 review,五子 tab 分组/选举 tab 成员/统一详情页规格/广场三类源/目录命名 全部确认无歧义,即本窗口完成,转执行任务卡。

## 整合:立法链端已完成(2026-06-24,另线程)

链端 `legislation-yuan`(pallet 27)+ `legislation-vote`(pallet 28)+ **宪法迁移**已全部完成、测试绿。重大变化:
- **公民宪法已迁为结构化链上法律**:`CitizenConstitution.html` 已删除;宪法 = `law_id=0, tier=宪法`(`constitution.scale` 创世注入),`ImmutableManifest` 冻结 8 条不可修改条款 `[1,2,3,17,19,23,33,41]`(=「宪法守卫」)。节点展示宪法走 `constitution_getDocument` RAW storage RPC,普通法律浏览可走 `LegislationApi`,不再有 HTML 真源。
- **客户端零实现**:citizenapp/citizenwallet 无任何 pallet 27/28 引用,全是新建。
- **整合点**:立法 tab = 法律浏览(含宪法 law_id=0)+ 立法机构;统一详情页对立法机构的「提案入口」= `legislation-yuan` propose;**宪法修改 = 对 law_id=0 的 `propose_amend_law`**(经立法投票引擎,特别案→公投),即本 app 立法发起流的一个实例,不再是「另一线程改 HTML」。立法详见 `20260624-legislation-dual-client.md`(其 CitizenApp 部分并入下表 P3–P5,依赖 P1)。

## 执行分步(整合 ADR-028 + 立法客户端)

前端 P1–P5、P7–P8 + 冷钱包 P6 全部**现在可做**(用现有数据/已完成链端,0 链改);P9–P11 选举/自治会需 runtime 二次确认 + 重新创世 bake。

| 步 | 模块 | 内容 | 依赖/约束 |
|---|---|---|---|
| **P1** | CitizenApp | 统一机构层 + 统一详情页;公权/治理两 tab 切到统一实现(行为保持),删一套重复 | 纯前端,0 链改 |
| P2 | CitizenApp | 五子 tab 壳 + 治理/公权 机构视图(institution_code 过滤);删治理静态注册表残留 | 依赖 P1 |
| P3 | CitizenApp | 立法基础 + 法律浏览(含宪法 law_id=0)= 立法 tab 内容:state_call 封装 + 立法 codec(Law/版本/章节条款项/ImmutableManifest 镜像解码)+ LegislationApi + 列表/详情/版本史 + 宪法不可修改条款徽章 | 依赖 P1;链端已就绪 |
| P4 | CitizenApp | 立法发起/修法/废法 = 统一详情页立法机构提案入口:`LegislationYuan(27)` propose 0/1/2 + 条款项编辑器 + 院序列 + 冷签;门控 houses[0] admin | 依赖 P1/P3 |
| P5 | CitizenApp | 立法投票(院内 28.1 复用 internal-vote / 公投 28.2 复用 joint-vote 凭证 + 快照 28.0)+ 计票/阈值/院进度展示 | 依赖 P3/P4 |
| P6 | CitizenWallet | 冷钱包立法解码:pallet_registry 27/28 + payload_decoder 6 call + action_labels + 两色严格签名 | 并行,链端已就绪 |
| P7 | CitizenApp | 广场重构(关注 + 本地区 + 我是管理员 三类动态流并集) | 依赖 P1 |
| P8 | CitizenApp | 选举 tab 活动视图骨架 + 统一详情页提案入口结构(基础动作接现有 internal-vote,选举置灰占位) | 依赖 P1 |
| P9 | 后端 CID | 删 CSLF/TSLF 模板 + 市/镇自治会 UNIN 注册 + 可识别标记 + purge + reconcile | 与 P10 协调 |
| P10 | 链端 Blockchain | 删 CSLF/TSLF 码(92→90)+ 能力层 `is_action_allowed` + sweep 推广 + `citizen-vote` 选举引擎(按层级电选机构管理员/法定代表人) | 二次确认 + 重新创世 |
| P11 | CitizenApp | 选举 tab 接选举引擎 + 提案入口接能力层(去占位) | 依赖 P10 |
| 并行 | 立法发起 | 宪法修改(总统府选总统 + 删选举委员会 + 废互选改分级普选 + 护宪大法官归 NLG)= 对 law_id=0 `propose_amend_law`,经特别案公投;受 ImmutableManifest 冻结条款约束 | 经 P4/P5 落地后可在 app 内发起 |

每步 IN 时另立执行任务卡。P1 完整方案见本卡下节。

## P1 完整方案:统一机构层 + 统一详情页(CitizenApp,纯前端)

目标:公权(`PublicInstitutionEntity`/`PublicInstitutionDetailPage`)与治理(`InstitutionInfo`/governance 详情页)两套并行实现合并为一套统一机构层 + 统一详情页;公权/治理两 tab 切到统一实现,**行为保持**(公权仍 省→市→机构 + 关注;治理仍 NRC/PRC/PRB);删重复一套。**不碰 runtime/后端/链上数据/投票引擎。**

预计修改目录(逐项中文注释):
- `citizenapp/lib/citizen/institution/` —【新增】统一机构层:`institution.dart`(合并实体,非法人带 parentCidNumber)、`institution_classification.dart`(institution_code→标签/orgType 单一源)、`data/`(由 `public/data/` 迁入归并的目录仓库)、`institution_chain_state.dart`(统一链态读服务,修 `'CGOV'` 硬编码)、`institution_detail_page.dart` + `institution_accounts_page.dart` + `institution_admin_list_page.dart`(合并两套)。代码 + 残留清理。
- `citizenapp/lib/citizen/public/` —【改】公权 tab 消费统一层,data 迁出,删自身重复详情/账户/管理员页。代码 + 残留清理。
- `citizenapp/lib/citizen/citizen_tab_page.dart` —【改】治理 tab 由统一目录过滤 NRC/PRC/PRB(P1 暂保持 3 tab,P2 扩 5 tab)。代码。
- `citizenapp/lib/governance/` —【改/清】删静态烘焙注册表对"列表/详情"的承载(`institution_registry`/`*.generated`/`GovernanceListPage`/governance 详情页),治理改读统一层;`findInstitutionByAccountId` 反查所需最小常量迁入统一分类。代码 + 残留清理。
- `citizenapp/test/` —【新增/改】统一实体/分类/详情页测试 + 行为保持回归。测试。
- 文档:更新模块说明 + 任务卡回写 P1 完成摘要。文档。

设计要点:① 统一实体身份来自目录,`mainAccount` 普通机构走 `deriveInstitutionMainAccountId`,NRC/PRC/PRB 走 china 固定账户(小覆盖表,行为保持),`orgType` 由 institutionCode 派生;② 目录已含 NRC/PRC/PRB(NRC×1/PRC×43/PRB×43 已 seed)→ 治理身份不再依赖静态注册表;③ 链态读服务统一"按主账户读 admins/提案/余额",公权治理同路径;④ 统一详情页按锁定规格,P1 提案入口仅搭结构(治理保留现有发起/投票,公权保持占位,能力门控留 P4/P6)。

验收(行为保持,真实运行态):公权/治理两 tab 改造前后行为一致;`flutter analyze` 0 + widget/单测过 + 真实 smoldot/真机验证;Grep 确认旧两套 + 静态注册表零残留。改完即更新文档/注释/清残留(死规则)。
