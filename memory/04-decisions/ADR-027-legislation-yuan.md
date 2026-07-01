# ADR-027 立法院模块(legislation-yuan)

## 标题

立法院模块:法律结构化上链 + 修法一律走投票引擎 + 严格按公民宪法表决模型落地。

状态:已接受,大部分已落地(2026-06-24)。立法院业务壳(idx=27)+ 立法投票 sub-pallet(idx=28)+ 宪法迁移(章>节>条>款 统一 + 创世注入 + 节点桌面端 re-point)均已实现验证;**双客户端 CitizenApp/CitizenWallet(另线程卡)与立法机构选举体系(election-vote 选举→admins 通道,独立卡)待续**。下文方向与边界已据实现校准。**OnChina 控制台「立法与表决」落地(2026-06-30,见第 10 节 + 卡 `20260630-onchina-legislation-console-framework`)**:法律案全链路(发起/表决/进度)operator 端 + 免登录大屏 display 端已交付;任免案/预算案链下 schema 预留(链端 kind 待另卡)。**正式创世前收口(2026-07-01,卡 `20260701-constitution-genesis-freeze-step1`)**:Law 主记录版本指针改为 `effective_version/latest_version/pending_version`,立法/修法 `effective_at` 改为毫秒时间戳;创世宪法 law_id=0、v1 直接生效且无待生效版;旧 HTML 真源与解析脚本删除,正式 raw 等 GitHub WASM CI 成功后用 `citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>` 烘焙。

> **重大修订(2026-06-25,用户逐条确认,见 `08-tasks/open/20260625-legislation-signing-5type-revision.md`)**:
> 1. **删常规案二审**(方案B 彻底删):本轮先改法案 7 条(44/45/73/75/79/81/118),官员任免 19 条另立专案。下文"四种表决/二审"表述以本修订为准。
> 2. **提案类型 4→5 类**:常规案/常规教育案/重要案/重要教育案/特别案(教育属性编进 vote_type,不另设内容分类字段)。`VoteType=Regular/RegularEducation/Major/MajorEducation/Special`(Important→Major)。阈值:常规系 >80%/≥60%、重要系 >90%/≥70%、特别 全员≥70%+公投。
> 3. **行政签署 + 否决救济**(宪法新增 44/45/73/79 + 122):市立法会通过→市长签署(否决=否决/30天超时=通过,单院无救济);省/国家参议会通过→省长/总统签署(否决或30天超时→退回立法院→院长+参议长+众议长三人会签,全同意生效/任一否决或超时否决);**特别案例外:公投通过即生效,任何人不再签署**。状态机加 `STAGE_LEG_SIGN`/`STAGE_LEG_OVERRIDE`。
> 4. **提案机构→表决院**:国家众议会/国家教委会/省众议会 本会通过→参议会;市立法会/市教委会/市自治会 委员直接进市立法会单院。提案方≠表决院(市级)。
> 5. **法定代表人**=机构首脑且为 admin 之一,即各级签署人(总统/省长/市长/院长/参议长/众议长);链上需新增字段。
> 6. **命名统一**(全工程):市公民立法委员会/市立法会、国家公民教育委员会/国家教委会、市公民教育委员会/市教委会、市公民自治委员会/市自治会、镇公民自治委员会/镇自治会;宪法全称首次出现用全称、其后简称。
> 7. **条号更正**:四种表决的宪法出处是**第44/45条**(非旧引用的"第18条");修订后为五类。
> **进度**:**A 宪法修订+重生 scale 已完成**(legislation-yuan 23/23);**B 链端已全部完成**——B1 VoteType 5类/阈值、B2 签署+会签状态机(STAGE_LEG_SIGN/OVERRIDE + executive_sign/override_sign + 30天超时 + 提案携带 executive/legislature)、B3 法定代表人(admins-change LegalRepresentatives + getter/setter)、B4 提案方↔表决院解耦 + ensure_routing(教育类⟺NED/CEDU)+ runtime 装配;验证:整 runtime cargo check 绿 + legislation-vote 20 + legislation-yuan 23 + 回归(votingengine/internal-vote 87/admins-change/grandpakey 17/multisig/organization)全过零回归。**C 命名统一 + 官员任免删二审已完成**(2026-06-25,合批改宪法):任免 19 条删二审(方案A/B)使全宪法 0 处二审/second-review(方案B彻底删完成)、命名首现严格审计修 3 处违规(市/镇自治会+校教委会)、重生 scale 217626 字节、不可修改 8 条逐字节不变、legislation-yuan 23 测试过。**护宪大法官修宪最终否决(2026-06-29,宪法第21条)已完成**:宪法新增第21条(护宪大法官对修宪享最终否决,重要案总统签署后/特别案公投后→3名及以上护宪大法官赞成生效/未获3名及以上赞成或30天超时否决)+ 旧21~140顺延22~141(141条)+ 第19条引用23/33/41→24/34/42(冻结条款创世前重定基准);链端 STAGE_LEG_CONSTITUTION_GUARD + ConstitutionGuardProvider接口(成员=国家司法院 `NJD` admins 中 `admin_role=护宪大法官` 的 5 人)+ guard_vote + needs_guard,护宪守卫改名;验证 legislation-vote 29 + internal-vote 88 + 公权管理员相关回归 + 整runtime cargo check 绿。**统一批已完成(2026-06-25)**:① 第20条第二款删「不隶属于任何机构」(护宪归口国家司法院);② **章节整体重排**新序 总则/政府/教委会/储委会/立法院/司法院/监察院(教委会72-86/储委会87-97/立法院98-113/司法院114-124/监察院125-141),**不可修改8条全在第一章不动、仍1/2/3/17/19/24/34/42逐字节不变**;③ **scale重生219064字节(141条)** + 不可修改常量→[24,34,42] + legislation-yuan测试140→141 + 守卫doc条号。验证:整runtime绿+legislation-yuan 23+legislation-vote 25+primitives 27+独立scale解码(不可修改内容@24/34/42正确)全过。命名公式化名保持现状(用户确认)。**D 双客户端固定治理识别已收口;重新创世真机QA按发布验收批执行**。

## 背景

### 痛点

- 迁移前,公民宪法是 runtime 内嵌旧 HTML 文件(约 933KB),通过 `include_str!` 编进 WASM,只能由旧 runtime API 只读读取。改一个字 = 重新编译 WASM + setCode 升级,负担极重;当前目标态已迁为 `legislation-yuan` 链上法律 + 节点 RAW 读。
- 除宪法外还有大量普通法律(如「市长选举法」),不可能逐条写进 runtime 代码。
- 需求:法律以结构化数据上链,可按宪法流程修改,公民在 CitizenApp 上可查看、可投票修改。

### 宪法依据(权威,本模块严格据此设计)

公民宪法第三章标题即「立法院 / Legislative Yuan」,已把立法体系、表决程序、修宪流程全部写死。本模块不自创规则,只把宪法条文工程化。关键条文:

- 第十七条:立法权归属与层级(第一/二/三款)。
- 第十八条:四种表决程序及阈值(国家/省立法院第一~三款;市立法会第一~三款)。
- 第十九条:修宪流程与不可修改条款清单。
- 第三章第一/二/三节:国家立法院、省立法院、市立法会的机构结构与提案/终审权分离。

## 决策

### 1. 两个新 pallet:业务壳 + 立法专属投票 sub-pallet

本设计新增两个 pallet,职责分离(用户拍板 2026-06-24:不动现有三投票模块,新增立法专属投票模块):

#### 1a. 业务壳:`citizenchain/runtime/public/legislation-yuan`

- 与宪法「立法院」、用户命名字面一致;`pallet_index = 27`;`MODULE_TAG = b"leg-yuan"`;对外类型名 `LegislationYuan`。
- 只承载法律数据:Law/LawVersion storage、状态机、`propose_*`(admin 入口)、Executor(投票通过后按 MODULE_TAG 认领、写新法律版本)、runtime 查询 API、不可修改条款硬拒。
- 严守投票职责边界硬规则:本壳绝不自实现投票/计票/快照,一律调下面的立法投票 sub-pallet 接口。

#### 1b. 立法专属投票 sub-pallet:`citizenchain/runtime/votingengine/legislation-vote`

- 新增投票引擎 sub-pallet,与 internal-vote / joint-vote / election-vote 平级;`pallet_index = 28`(待实现时确认空号);对外类型名 `LegislationVote`。
- 定位:立法机构专属投票,承载宪法第十八条四种表决类型 + 两院顺序 + 强制公投,一处集中。
- `Config: frame_system::Config + votingengine::Config`,复用核心 crate 全部共享基础设施(见第 5 节),只本地保管自己的计票账本。
- **完全不修改 internal-vote / joint-vote / election-vote 三个 sub-pallet 的逻辑**:三者零改动、零回归;election-vote 空骨架原样保留供未来公职人员选举用。
- **更正(2026-06-24,第2步精读核心后)**:核心 `votingengine` crate 按 `kind`/`stage` 硬编码分发,未知 kind 直接 `Err`。要让立法投票成为头等模式并真正共享核心基础,**第2步必须扩展核心 crate**(新增 `PROPOSAL_KIND_LEGISLATION` + 立法 stage + Config 三关联类型 `LegislationFinalizer`/`LegislationCleanup`/`LegislationVoteResultCallback` + 分发分支),并在所有 `votingengine::Config` 实现补这三类型(测试 mock 装 `()`)。这是 additive 扩展,不改三个 sub-pallet 逻辑,但"纯加 sub-pallet 零核心改动"的说法作废。详见任务卡第2步 2a。
- legislation-yuan 业务壳通过本 sub-pallet 对外的 Engine trait(如 `LegislationVoteEngine`)创建/绑定投票,投票终态回调写回业务壳 Executor。

### 2. 立法机构权限矩阵(第十七条 / 第三章)

立法权只属于以下三类机构,其它机构无立法权:

| 机构 | 管辖范围 | 院制 | 提案/起草方 | 终审方 |
|---|---|---|---|---|
| 国家立法院 | 全国 + 修宪 | 两院 | 众议会起草发起(无终审权);教育类由国家教委会起草 | 参议会审议终审(无起草权) |
| 省立法院 | 本省(国家立法院授予立法权) | 两院 | 省众议会起草发起 | 省参议会审议终审 |
| 市立法会 | 本市(宪法直接赋予) | 单院 | 市自治会委员 / 市立法会委员 / 该市任意公民 / 该市所属镇自治会委员 | 市立法会委员表决 |

补充:

- 国家教委会不是独立立法机构,是「教育类法案」的起草方,起草并经教委会表决通过后交国家立法院参议会表决(第十八条第三款、第六十一条)。

机构与议员建模(用户拍板,2026-06-24):

- 「议员 / 委员」是现实世界表达;系统内只有一种身份 = 机构 admins。议员 / 委员 = 立法机构的 admins,不另建议员名册。
- 议员换届 = admins-change 模块换管理员,复用机构管理员单一真源,不新造换届机制。
- 众议会 / 参议会 = 两个独立机构(各自 admins=议员):
  - 国家立法院 = 国家众议会(institution) + 国家参议会(institution)
  - 省立法院(每省) = 省众议会(institution) + 省参议会(institution)
  - 市立法会(每市) = 单一 institution(admins=委员)
  - 国家教委会 = institution(admins=委员),教育类法案起草方
- 法案的链上提案恒由对应立法机构的 admin 发起,所有 `propose_*` 入口只认 admin,与其它治理模块一致。
- 市立法会公民提案门槛(第十八条市立法会第一款:≥1000 该市公民 + ≥5 公民团体联署,或集会单日参与 > 该市人口 10%)是现实世界前置义务——满足时市立法会委员有义务在链上发起提案;链上不做公民联署入口。

### 3. 法律层级与数据模型

法律分层 `tier`:`宪法 / 国家 / 省 / 市`(宪法为最高层级,见第 7 节迁移)。

**所有法律统一「章(Chapter)>节(Section)>条(Article)>款(Clause)」结构**(章节条做目录、条款做正文;用户 2026-06-24 拍板),链上 SCALE 编码:

```text
Law {
  law_id,
  tier,                 // 宪法/国家/省/市
  scope_code,           // 行政区 code,0 = 全国;省/市用 china.sqlite code(遵守 ADR-021 单源)
  houses,               // 立法机构院结构,提案携带(单院 1 项 / 两院 [众,参])
  effective_version,    // 当前真正生效的版本;新法待生效时为空
  latest_version,       // 已写入链上的最新版本
  pending_version,      // 已通过但未到生效时间的版本;同一法律最多一个
  status,
}

LawVersion {
  law_id, version,
  title, title_en?,                      // 宪法双语;普通法律可单语(_en 可选)
  chapters: BoundedVec<Chapter>,         // 所有法律必有目录
    Chapter { number, title, title_en?, sections: BoundedVec<Section> }
    Section { number, title, title_en?, articles: BoundedVec<Article> }
    Article { number, title, title_en?, body, body_en?, clauses: BoundedVec<Clause> }  // body 必填(正文不能空)
    Clause  { number, text, text_en? }   // 款,可空(不是所有条都有款)
  content_hash,          // blake2_256(规范化 SCALE);完整性 + 公投/签名绑定
  vote_type,             // 通过本版本所用的表决类型(常规/常规教育/重要/重要教育/特别)
  proposal_id,           // 投票引擎提案 ID(可回溯)
  published_at,          // 发布时间戳(毫秒,投票通过写入版本时记)
  effective_at,          // 生效时间戳(毫秒,可晚于发布)
}
```

- 必填约束:章/节/条恒在(所有法律都有目录),条 `body` 必填(空正文的条无意义),款可空。
- 体积:结构化条文比 HTML 小一个量级;每层设 `BoundedVec` 与字节上限。
- 双语:宪法中英双语,普通法律可只中文 → `_en` 字段可选,不强制双语。
- 修改粒度:提案针对「(law_id, 第 N 条)」做增 / 改 / 删(`find_article` 遍历 章>节>条 按 number),version+1,diff 干净。
- 客户端:CitizenApp 渲染「章>节>条>款」目录+正文;节点桌面端据结构化法律重建 HTML(复用原宪法 CSS 外壳,样式不变),无需解析/内置 HTML。

状态机:

```text
提案创建 → 投票中 → 投票通过后写入 LawVersion
  ├─ effective_at <= 当前链上时间戳:立即写入 effective_version
  └─ effective_at > 当前链上时间戳:写入 pending_version,到时间后自动切换 effective_version
```

历史版本由 `LawVersions[law_id][version]` 保留;当前生效版、最新写入版、待生效版不再靠旧的版本号减一规则推断。

### 4. 四种表决类型 → 投票引擎映射(第十八条)

宪法把每个机构的表决固定为四种,条件是「参与率 + 赞成率」,特别案叠加公民投票:

| 表决类型 | 立法机构内部条件 | 是否叠加公民投票 |
|---|---|---|
| 常规案 | > 80% 现任议员/委员参与,≥ 60% 赞成 | 否 |
| 重要案 | > 90% 参与,≥ 70% 赞成 | 否 |
| 常规案二审 | 全体参与,≥ 50% 赞成 且 反对 < 20% | 否 |
| 特别案 | 全体参与,≥ 70% 赞成 | 是,且必须通过(国家级:全国 ≥70% 投票权公民参与 + ≥70% 赞成;省级:本省;市级:本市) |

全部立法表决统一走新 `legislation-vote` sub-pallet,由它按"机构构成 + 表决类型"内部分流(机构=独立 institution、议员=admins):

- 市立法会(单院):常规/重要/二审 → 单院模式,一段内部表决(委员,按 quorum%/approve%/oppose_cap% 计票)。
- 国家立法院 / 省立法院(两机构 众议会→参议会):常规/重要/二审 → 两院模式,两段顺序内部表决(众议会按表决类型通过 → 参议会按表决类型终审通过),无公投。
- 国家教委会教育类法案:教委会(发起,内部表决) → 参议会(终审,内部表决) → 两院模式两段内部表决。
- 特别案 + 核心修宪 → 特别案模式:内部阶段(单院=委员;两院=众→参,全员 ≥70% 赞成) + 强制公投阶段(人口快照,国家=全国、省=本省、市=本市,≥70% 参与 + ≥70% 赞成),两阶段都必须通过。

关键:这三种模式(单院 / 两院 / 特别案)全部在 `legislation-vote` 一个 sub-pallet 内实现,复用核心 crate 的提案生命周期、`AdminSnapshot`、人口快照验签 trait、清理、反向索引(第 5 节),只本地写自己的计票账本。现有 internal-vote / joint-vote / election-vote 零改动。

结论(回应"是否只用内部/联合投票"):常规案/重要案/二审只有内部表决(单院一段、两院两段);特别案与核心修宪,宪法第十八条第二款、第十九条强制必须叠加公民投票且通过 → 由 legislation-vote 特别案模式的"内部阶段 + 强制公投阶段"承载。三种档位同壳,不借用也不修改其它投票模块。

### 5. 新增 legislation-vote sub-pallet(本项目核心,additive 不改存量)

不修改 internal-vote / joint-vote / election-vote;新增一个立法专属投票 sub-pallet,把宪法第十八条的表决规则集中实现,复用核心 crate 共享基础。

(A) 复用核心 crate(`votingengine`)共享基础设施(`Config: votingengine::Config`,零拷贝):

- 提案生命周期:`Proposals` / `NextProposalId` / `ProposalData` / `ProposalOwner` / `ProposalMeta` / 状态机(`finalize_proposal` / `set_status_and_emit` / `mark_proposal_passed_at`)。
- 管理员快照:`AdminSnapshot`(提案创建时锁定立法机构现任 admins,即现任议员/委员名册——不另建名册)。
- 公投基础:`PopulationSnapshotVerifier` 人口快照验签 trait + `CidEligibility` 资格 trait。
- 到期清理、反向索引(`ProposalsByInstitution / ByOwner / ByYear`)、互斥锁、ID 生成。

(B) 本 sub-pallet 本地新增(只是计票账本,对标 joint-vote 的 `JointTallies` / `ReferendumTallies` / `UsedPopulationSnapshotNonce`):

- `VoteRule { quorum_pct, approve_pct, oppose_cap_pct: Option, require_referendum, referendum_scope }`,四种表决类型各一组常量。
- 立法内部表决计票账本:按 `AdminSnapshot` 现任 admins 总数算参与率/赞成率/反对率;二审加反对率上限(< 20%)判定。
- 两院顺序内部阶段:可配置「机构阶段序列」,单院 = 一段,两院 = 众议会段 → 参议会段,每段独立计票 + 独立阈值。
- 强制公投计票账本:内部阶段全部通过后强制进入(AND,不是否决救济);门槛 ≥70% 参与 + ≥70% 赞成;作用域全国/省/市;复用核心人口快照验签。
- 立法专属 Engine trait(如 `LegislationVoteEngine`)+ 终态回调,供 legislation-yuan 业务壳调用与回写。

(C) 与存量关系:internal-vote / joint-vote / election-vote 零改动零回归。joint-vote 的三储机构加权模式与本 sub-pallet 各管各的,不存在"两套模式并存于一 pallet"的问题。

### 6. 修宪特别约束(第十九条)

- 第一章总则核心条款修改 → 国家立法院特别案(= 必须公民投票)。
- 不可修改条款硬清单:第 1、2、3、17、19、23、33、41 条(单源 `primitives::count_const::IMMUTABLE_CONSTITUTION_ARTICLES`)。
- 其它章节修改 → 国家立法院重要案。

#### 6.1 不可修改条款「真不可修改」三层守卫 —— 已落地(2026-06-24)

要求:这 8 条「改代码、改 runtime 升级都改不动,改了只能重新创世」。纯 runtime 校验可被一次 setCode 解除,
故把关必须搬出可升级的 runtime,放到节点共识层并锚定创世。三层纵深:

- **L1 运行时提案守卫(`legislation-yuan::ensure_immutable_preserved`)**:`propose_amend_law` 时逐条逐字比对,
  碰这 8 条即 `ImmutableArticleViolation`。第一线、报错干净,但可被 runtime 升级绕过 → 不是最终保证。
- **L2 节点共识守卫(`node/src/core/constitution.rs::ConstitutionGuard`,最终保证)**:包住 PoW `BlockImport`,
  对携带 body 的区块在父状态上**只读执行**得到后置存储变更,若变更触及立法院模块存储,则据「变更 ∪ 父状态」
  RAW 重建宪法相关键(`Laws[0]`/`LawVersions[0][v]`,硬编码 `Blake2_128Concat` key,**不读链上 metadata**),
  逐条比对**创世(block#0)基准**;命中违规 → `Ok(ImportResult::KnownBad)`(内层永不调用,块不入库、不成最佳块)。
  装配在 `service.rs` 两处导入(网络导入队列 + 本地挖矿),故诚实节点既不接受也不产出违规块。
- **L3 创世锚 + 二进制锚 + 链上 manifest**(均已落地):内容基准从 block#0 状态派生(创世哈希为之背书,改它=换链);
  不可修改条款**清单**为 `primitives::count_const::IMMUTABLE_CONSTITUTION_ARTICLES` 常量、**编译进节点二进制**(链上 WASM 改不到节点副本)。
  另:`legislation-yuan` 新增**只读 storage** `ConstitutionImmutableManifest`(清单 + 逐条 blake2_256 摘要),仅 `genesis_build` 写、无 setter;
  `genesis_build` 同时**逐条强断言**不可修改条款存在(缺即 panic,烤不出非法创世)。节点 `ConstitutionGuard::new` 启动期从 block#0
  **交叉校验**:创世 manifest 清单 == 二进制清单,且逐条摘要 == 创世条文摘要;任一不符 → **节点拒绝启动**(把"清单"从单锚二进制升级为双锚 + 启动一致性闸)。
  改这 8 条的唯一路径 = 改创世(新创世哈希=新链)+ 改节点二进制(硬分叉),即「只能重新创世」。

威胁覆盖:普通 amend→L1 拒;setCode 删 L1 / migration 直写存储 / 改清单常量 / 改版本指针指向篡改版本 /
改 pallet 名让 key 落空(fail-safe 拒)→ 全部 L2 拒块。`detect_violation` 自身执行/取数出错时也 fail-closed 拒块,
不保留未经校验的导入路径。**待用户多节点真机 QA**:构造恶意改第一条的块,验证全网 orphan。

#### 6.2 加固五项(2026-06-24,卡 `20260624-constitution-immutable-guard`,review 发现的绕过面)

- **H1 守卫补 Law 元数据 + 唯一性**(堵"只校验条文字节"):`check_immutable_articles` 除 8 条条文外,断言 `Laws[0]`
  `tier==Constitution`、`scope==0`、`status!=Repealed`(**不钉 Effective**,放行合法修宪 Pending 窗口)、`houses==创世`,
  并断言 `LawsByScope[宪法][0]==[0]`(挡 migration 新立第二部宪法 / 隐藏 law_id=0)。判别值由 `enum_discriminants_match_node_guard` 测试钉死。
- **H2 warp/状态导入校验**(堵"warp 落到篡改态"):`import_block` 对 `with_state()` 块**提交前**校验。
  ⚠️ **二次 review 修正(P1)**:vendored GRANDPA 在 `inner.import_block` 内即把状态置 finalized 落库,**post-import `KnownBad` 无法回滚**;
  故改为 `verify_imported_state` 从 `params.state_action` 的 `ImportedState` 抽立法院前缀键、**调用 inner 之前**跑全套校验,
  违规/无法抽取 → `KnownBad`(不调用 inner,什么都不落库)。
- **H3 RPC 改 RAW 读 + 取生效版本**(堵"展示信任可升级 API" + "提前显示 Pending 版"):`constitution_getDocument` 直接
  `StorageProvider` RAW 读 `Laws[0]`/`LawVersions[0][v]`(不走 runtime API),版本取显式 `effective_version`,
  `source="legislation-raw"`。
- **H4 禁止新立第二部宪法**(堵"立法入口造第二部宪法"):`propose_enact_law` 拒 `tier==Constitution`(`CannotEnactConstitution`),宪法只能创世存在。
- **二次 review 修正三项(P2/P2-P3/P3)**:
  - **P2 fail-closed**:`detect_violation` 自身失败(无法读父状态/执行/取变更)→ 改为 `KnownBad` 拒块(由旧放行口径改为安全优先);
    宪法读/解码/比对本就 fail-closed。
  - **P2/P3 setCode 强制全检**:快路径放行条件加"且未升级 runtime"——delta 含 `:code` 即强制走全量不变式校验。
  - **P3 至多一个待生效版本**:`propose_amend_law`/写入层拒 `pending_version.is_some()`(`AmendmentAlreadyPending`),
    避免多个待生效版本互相覆盖;现行生效版直接读 `effective_version`,不再做减一推断。
- 验收:node 21 单测 + legislation-yuan 23 单测(含禁宪法/判别值/拒重叠 Pending/写入层复校验/warp 预校验纯函数)+ no_std + fmt 全过。
  **待 QA**:H2 warp 多节点真机(提交前抽取 `ImportedState` 的实测,实现风险最高项)+ 双执行 PoW 性能。

### 7. 宪法迁移(并入本模块)—— 已完成(2026-06-24,卡 `20260624-constitution-migration.md`)

- 宪法纳入本模块,作为 `tier = 宪法` 的最高层级法律,实现「宪法可按条修改(走特别案/重要案)」。
- 已落地:结构化 `章>节>条>款 + 中英双语` 产物 `legislation-yuan/src/constitution.scale` 作为正式创世种子;创世注入为 `law_id=0 tier=宪法 effective_version=Some(1) latest_version=1 pending_version=None status=Effective`。
- 唯一真源 = 链上立法院模块。旧 HTML 文件、旧 runtime API 与以 HTML 为输入的解析脚本均已按禁止兼容硬规则删除。
- 节点桌面端「公民宪法」tab **保持原样式**:`constitution_getDocument` RPC 直接 RAW 读 `LegislationYuan::Laws[0]` 与当前生效版 `LawVersions[0][v]`(不走可升级 runtime API),`node/src/core/constitution.rs` 据链上结构化法律 + 抽出的原 CSS 外壳(单文件 `constitution_shell.html`,含 `<!--CONSTITUTION_TOC-->`/`<!--CONSTITUTION_CONTENT-->` 两占位标记,渲染时替换)重建完整 HTML,前端 iframe 零改动 → 样式与迁移前逐字一致。节点端宪法能力(渲染 + 下述守卫)统一收口此单文件。

### 8. 上链时机

- 新增 pallet = runtime 变更,新增 `pallet_index=27`。
- 两条路径:搭车现有待重新创世队列(CID T3/T4、两和基金、账户派生等),或走 setCode 链上升级。
- 一切 `citizenchain/runtime/` 改动遵守 runtime 二次确认硬规则:实现前单独报完整路径 + 改动内容 + 原因,取得第二次确认。

### 9. 双客户端联动(chat-protocol §5,强制同任务范围)

修法提案要冷钱包扫码签名,涉及 runtime + 扫码签名联动,必须把双客户端纳入同一执行范围:

- CitizenApp:法律列表 / 详情 / 版本史 / 投票页(读 runtime API + 发提案)。
- CitizenWallet:修法提案二维码 decoder + 签名展示;按 ADR-026 统一签名协议新增 op_tag。
- 不允许「先改 runtime,双端后补」。

### 10. OnChina 控制台「立法与表决」落地(2026-06-30,卡 `20260630-onchina-legislation-console-framework`)

立法院的**管理端 + 大屏**在 OnChina 控制台落地(链端 `legislation-yuan`/`legislation-vote` 本轮**零改动**,只读核对)。核心决策:

- **提案三分类维度(`ProposalCategory`)**:法律案(Law)/ 任免案(Personnel)/ 预算案(Budget),以提案类型为可扩展维度。
  - **法律案**:本轮**全链路实现**——发起(章>节>条>款编辑器 + 冷签 QR)→ 院内/两院表决(一人一票冷签)→ 进度(六阶段 + 计票)。后端 `domains/legislation/{law,chain_read_proposal}`,前端 `legislation/operator/law/`。
  - **任免案 / 预算案**:本轮**仅锁链下 schema**(`personnel/model.rs` = 任免职书;`budget/model.rs` = 类>款>项>目,金额分 u128 字符串出线防 JS 精度丢失)。**链端现状核实:无 `PROPOSAL_KIND_PERSONNEL/BUDGET`、无任免/预算 pallet/extrinsic**(仅 kind 0-3);**禁**借道 `PROPOSAL_KIND_LEGISLATION`(会污染 leg-yuan 回调,该回调只写 `LawVersion`)。发起/表决/读链待链端支持后**另卡**(新增 kind + 业务 pallet + 重新创世,含 runtime 二次确认)。
- **不改宪法(结论性前提)**:法律提案权宪法只给立法机关;政府的**人事任免**由宪法第 100/106 条直授(提交人事任免职书 → 参议会/立法会常规案表决任免),**预算**由《预算法》(普通法)授权——二者走独立提案类型,非法律案。给政府普通立法提案权才需修宪(不必要)。
- **operator / display 双路由(契合 ADR-030)**:`operator/`(议员登录鉴权操作端)+ `display/`(大厅大屏**免登录只读**;经 `#/display` 顶层分流,机构由节点绑定 `active_node_binding` 唯一确定、**不接受请求参数**、fail-closed;单飞 + 短 TTL 缓存抑制无鉴权链读放大;逐席投票读 `LegHouseVotesByAdmin` 双 Map 部分键迭代 + `storage_key_suffix::<32>` 还原议员账户)。
- **onchina 职责边界**:只做「组织提案数据 + 扫码冷签 + 提交 extrinsic + 读链展示」,**绝不计票/推进状态机**(全归投票引擎);读路径 scope fail-closed。
- **遗留(均有卡/已登记)**:双客户端 CitizenApp/CitizenWallet(卡 `20260624-legislation-dual-client`,端到端 scan+submit)、任免/预算链端(`PROPOSAL_KIND_PERSONNEL/BUDGET` 另卡)、行政签署人登录 + `executive_sign`、议员 admins 灌入(election-vote)、大屏跨院活跃提案可见性细化、前端 ESLint(react-hooks/jsx-a11y)。

## 影响

预计涉及目录(实现阶段,非本 ADR 改动):

- `citizenchain/runtime/public/legislation-yuan/`:新建业务壳 pallet(代码)。
- `citizenchain/runtime/votingengine/legislation-vote/`:新建立法专属投票 sub-pallet(代码,核心)。
- `citizenchain/runtime/votingengine/`(核心 crate):仅在确需时补共享 trait/类型(如 `LegislationVoteEngine` 注册);internal-vote / joint-vote / election-vote 不改。
- `citizenchain/runtime/src/lib.rs`、`src/configs/`、`src/apis.rs`、`src/tests/`:注册两 pallet、装配 Config、回调路由、法律查询 API、唯一性测试(代码)。
- `citizenchain/runtime/primitives/`:法律数据类型;宪法迁移与旧 html/API 清理(代码 + 残留清理)。
- `citizenchain/node`:若涉及重新创世 / setCode 发布(构建/部署)。
- `citizenapp/`:法律浏览 + 投票 UI(代码)。
- `citizenwallet/`:修法提案 decoder + 签名展示(代码)。
- `citizencode/`(可能):立法机构身份来源(机构 + admins,复用现有 CID 机构体系)。
- `memory/`:本 ADR + 后续任务卡 + 文档同步。

硬规则约束清单:投票职责边界 / runtime 二次确认 / runtime+扫码签名联动 / 禁止兼容 / 彻底改造 / 新增文件先确认 / 真实运行态验收。

## 备选方案

- 法律只存哈希 + 全文链下:被否。客户端取全文麻烦、防篡改成本高,且公民 App 要"方便查看"。
- 宪法不并入、保留 WASM 内嵌:被否。违反单一真源与禁止兼容;宪法仍不可按条修改。
- 简化阈值模型(不做参与率/反对率精确计票):被否。用户明确要求「按宪法完整实现」。
- 法律存为整块文本 blob:被否。无法按条修改、diff 不清晰、不利渲染。

## 后续动作(任务卡拆分,待 ADR 定稿后逐张创建)

1. 卡1 新增 legislation-vote sub-pallet(核心,先行):`Config: votingengine::Config` 复用核心共享基础 + 本地 `VoteRule`/立法计票账本 + 单院/两院/特别案三模式 + 强制公投 + `LegislationVoteEngine` trait + 测试;internal-vote / joint-vote / election-vote 零改动。
2. 卡2 legislation-yuan 业务壳:数据模型 + 状态机 + `propose_*`(admin 入口) + Executor + runtime API + 不可修改条款硬拒;调 legislation-vote。
3. 卡3 双客户端:CitizenApp(浏览+投票)+ CitizenWallet(decoder+签名,ADR-026 新 op_tag)。
4. 卡4 宪法迁移:HTML → 结构化条文(中英双语),清理旧 include_str!/API。
5. 上链:按 runtime 二次确认,协调重新创世 / setCode。

## 已拍板(2026-06-24)

- 新增立法专属投票 sub-pallet `votingengine/legislation-vote`,不修改 internal-vote / joint-vote / election-vote;立法投票共享投票引擎核心基础设施,只本地存计票账本。
- 议员/委员 = 机构 admins,不另建名册;换届走 admins-change。
- 众议会/参议会 = 两个独立机构;两院法案 = 两段顺序内部表决(众→参)。
- 特别案/核心修宪 = legislation-vote 特别案模式(内部阶段 + 强制公投),不借用 election-vote。
- 提案恒由 admin 发起;市立法会公民联署门槛为现实前置,不做链上联署入口。
- 投票引擎按宪法完整实现(参与率%/赞成率%/反对率上限%/强制公投)。

## 待确认问题(review 时拍板)

- 两个新 pallet 的 `pallet_index`(暂定业务壳 27 / sub-pallet 28),实现时确认空号。
- 任务卡执行顺序:建议卡1(legislation-vote)先行,业务壳依赖其 Engine trait。
- 上链时机:搭车现有待重新创世队列,还是独立 setCode。
