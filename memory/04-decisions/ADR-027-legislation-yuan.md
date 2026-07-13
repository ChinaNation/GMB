# ADR-027 立法院模块(legislation-yuan)

## 标题

立法院模块:法律结构化上链 + 修法一律走投票引擎 + 严格按公民宪法表决模型落地。

状态:已接受,大部分已落地(2026-06-24)。立法院业务壳(idx=27)+ 立法投票 sub-pallet(idx=28)+ 宪法迁移(章>节>条>款 统一 + 创世注入 + 节点桌面端 re-point)均已实现验证;**双客户端 CitizenApp/CitizenWallet(另线程卡)与立法机构选举体系(election-vote 选举→admins 通道,独立卡)待续**。下文方向与边界已据实现校准。**OnChina 控制台「立法与表决」落地(2026-06-30,见第 10 节 + 卡 `20260630-onchina-legislation-console-framework`)**:法律案全链路(发起/表决/进度)operator 端 + 免登录大屏 display 端已交付;任免案/预算案链下 schema 预留(链端 kind 待另卡)。**正式创世冻结收口(2026-07-01,卡 `20260701-constitution-genesis-freeze-step1`)**:Law 主记录版本指针改为 `effective_version/latest_version/pending_version`,立法/修法 `effective_at` 改为毫秒时间戳;创世宪法 law_id=0、v1 直接生效且无待生效版;旧 HTML 真源与解析脚本删除;GitHub WASM run `28492547251` 成功后已用 `citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>` 烘焙正式 raw,genesis hash `0x6c88667d43f5a2690f2cb176f5883e051a057db6bee5fa56bc8337becbf23417`。

> **重大修订(2026-06-25,用户逐条确认,见 `08-tasks/open/20260625-legislation-signing-5type-revision.md`)**:
> 1. **删除已废弃的重复表决流程**:本轮已改立法表决、教育类法案与签署救济相关条款,官员任免另立专案。正文只保留当前流程口径。
> 2. **提案类型 4→5 类**:常规案/常规教育案/重要案/重要教育案/特别案(教育属性编进 vote_type,不另设内容分类字段)。`VoteType=Regular/RegularEducation/Major/MajorEducation/Special`(Important→Major)。阈值:常规系 >80%/≥60%、重要系 >90%/≥70%、特别 全员≥70%+公投。
> 3. **行政签署 + 否决救济**(创世正文见第46/100/106条):市立法会通过→市长签署(否决=否决/30天超时=通过,单院无救济);国家/省立法院参议会通过→总统/省长签署(否决或30天超时→退回立法院→院长+参议长+众议长三人会签,全同意生效/任一否决或超时否决);**特别案例外:公投通过即生效,任何人不再签署**。状态机加 `STAGE_LEG_SIGN`/`STAGE_LEG_OVERRIDE`。
> 4. **提案机构→表决院**:国家立法院众议会/国家教委会/省联邦立法院众议会 本会通过→对应立法院参议会;市立法会/市教委会/市自治会 委员直接进市立法会单院。提案方≠表决院(市级)。
> 5. **法定代表人**是机构公开信息，不要求等同或从属于管理员集合；姓名、CID、钱包账户由 entity 的 `InstitutionInfo` 保存，立法签署只读取该唯一真源。
> 6. **命名统一**(全工程):市公民立法委员会/市立法会、国家公民教育委员会/国家教委会、市公民教育委员会/市教委会、市公民自治委员会/市自治会、镇公民自治委员会/镇自治会;宪法全称首次出现用全称、其后简称。
> 7. **条号更正**:创世正文中五类表决出处为**第45/46条**;教育类提案见第75/79条;国家/省签署与三人会签见第100/106条;不再引用旧条号口径。
> **进度**：签署与会签状态机保持不变。2026-07-13 已删除 admins 中旧法定代表人副本、setter 和首位管理员回退，签署人账户统一从 public/private entity `InstitutionInfo.legal_representative_account` 查询；姓名与 CID 同属该机构公开信息。其它历史实施与验收记录见对应任务卡。

## 背景

### 痛点

- 迁移前,公民宪法是 runtime 内嵌旧 HTML 文件(约 933KB),通过 `include_str!` 编进 WASM,只能由旧 runtime API 只读读取。改一个字 = 重新编译 WASM + setCode 升级,负担极重;当前目标态已迁为 `legislation-yuan` 链上法律 + 节点 RAW 读。
- 除宪法外还有大量普通法律(如「市长选举法」),不可能逐条写进 runtime 代码。
- 需求:法律以结构化数据上链,可按宪法流程修改,公民在 CitizenApp 上可查看、可投票修改。

### 宪法依据(权威,本模块严格据此设计)

公民宪法已把立法体系、五类表决程序、修宪流程与护宪终审全部写死。本模块不自创规则,只把宪法条文工程化。关键条文:

- 第十七条:立法权归属与层级(第一/二/三款)。
- 第十九条:修宪流程与不可修改条款清单。
- 第二十一条:护宪大法官对修宪提案的最终否决。
- 第四十五/四十六条:国家/省立法院与市立法会五类表决程序、阈值和市级签署规则。
- 第七十五/七十九条:国家教委会、市教委会教育类法案起草与表决。
- 第一百/一百零六条:国家/省立法院行政签署与三人会签救济。
- 立法院相关章节:国家立法院、省立法院、市立法会的机构结构与提案/终审权分离。

## 决策

### 1. 两个新 pallet:业务壳 + 立法专属投票 sub-pallet

本设计新增两个 pallet,职责分离(用户拍板 2026-06-24:不动现有三投票模块,新增立法专属投票模块):

#### 1a. 业务壳:`citizenchain/runtime/public/legislation-yuan`

- 与宪法「立法院」、用户命名字面一致;`pallet_index = 25`(原定 27,2026-07-12 号段连续化后为 25);`MODULE_TAG = b"leg-yuan"`;对外类型名 `LegislationYuan`。
- 只承载法律数据:Law/LawVersion storage、状态机、`propose_*`(admin 入口)、Executor(投票通过后按 MODULE_TAG 认领、写新法律版本)、runtime 查询 API、不可修改条款硬拒。
- 严守投票职责边界硬规则:本壳绝不自实现投票/计票/快照,一律调下面的立法投票 sub-pallet 接口。

#### 1b. 立法专属投票 sub-pallet:`citizenchain/runtime/votingengine/legislation-vote`

- 新增投票引擎 sub-pallet,与 internal-vote / joint-vote / election-vote 平级;`pallet_index = 26`(原定 28,2026-07-12 号段连续化后为 26);对外类型名 `LegislationVote`。
- 定位:立法机构专属投票,承载宪法第四十五/四十六条五类表决类型 + 两院顺序 + 强制公投 + 行政签署/会签救济,一处集中。
- `Config: frame_system::Config + votingengine::Config`,复用核心 crate 全部共享基础设施(见第 5 节),只本地保管自己的计票账本。
- **完全不修改 internal-vote / joint-vote / election-vote 三个 sub-pallet 的逻辑**:三者零改动、零回归;election-vote 空骨架原样保留供未来公职人员选举用。
- **更正(2026-06-24,第2步精读核心后)**:核心 `votingengine` crate 按 `kind`/`stage` 硬编码分发,未知 kind 直接 `Err`。要让立法投票成为头等模式并真正共享核心基础,**第2步必须扩展核心 crate**(新增 `PROPOSAL_KIND_LEGISLATION` + 立法 stage + Config 三关联类型 `LegislationFinalizer`/`LegislationCleanup`/`LegislationVoteResultCallback` + 分发分支),并在所有 `votingengine::Config` 实现补这三类型(测试 mock 装 `()`)。这是 additive 扩展,不改三个 sub-pallet 逻辑,但"纯加 sub-pallet 零核心改动"的说法作废。详见任务卡第2步 2a。
- legislation-yuan 业务壳通过本 sub-pallet 对外的 Engine trait(如 `LegislationVoteEngine`)创建/绑定投票,投票终态回调写回业务壳 Executor。

### 2. 立法机构权限矩阵(第十七条 / 立法院章节)

立法权只属于以下三类机构,其它机构无立法权:

| 机构 | 管辖范围 | 院制 | 提案/起草方 | 终审方 |
|---|---|---|---|---|
| 国家立法院 | 全国 + 修宪 | 两院 | 国家立法院众议会起草发起(无终审权);教育类由国家教委会起草 | 国家立法院参议会审议终审(无起草权) |
| 省立法院 | 本省(国家立法院授予立法权) | 两院 | 省联邦立法院众议会起草发起 | 省联邦立法院参议会审议终审 |
| 市立法会 | 本市(宪法直接赋予) | 单院 | 市自治会委员 / 市立法会委员 / 该市任意公民 / 该市所属镇自治会委员 | 市立法会委员表决 |

补充:

- 国家教委会不是独立立法机构,是「教育类法案」的起草方,起草并经教委会表决通过后交国家立法院参议会表决(第十条、第七十五条及第一百条)。

机构与议员建模(用户拍板,2026-06-24):

- 「议员 / 委员」是现实世界表达;系统内只有一种身份 = 机构 admins。议员 / 委员 = 立法机构的 admins,不另建议员名册。
- 议员换届 = admins-change 模块换管理员,复用机构管理员单一真源,不新造换届机制。
- 众议会 / 参议会 = 立法院下设两院(各自 admins=议员,各自作为链上表决院):
  - 国家立法院 = 国家立法院众议会 + 国家立法院参议会
  - 省立法院(每省) = 省联邦立法院众议会 + 省联邦立法院参议会
  - 市立法会(每市) = 单一 institution(admins=委员)
  - 国家教委会 = institution(admins=委员),教育类法案起草方
- 法案的链上提案恒由对应立法机构的 admin 发起,所有 `propose_*` 入口只认 admin,与其它治理模块一致。
- 市立法会公民提案门槛(≥1000 该市公民 + ≥5 公民团体联署,或集会单日参与 > 该市人口 10%)是现实世界前置义务——满足时市立法会委员有义务在链上发起提案;链上不做公民联署入口。

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

### 4. 五类表决类型 → 投票引擎映射(第四十五/四十六条)

宪法把立法表决固定为五类,条件是「参与率 + 赞成率」,特别案叠加立法公投:

| 表决类型 | 立法机构内部条件 | 是否叠加立法公投 |
|---|---|---|
| 常规案 | > 80% 现任议员/委员参与,≥ 60% 赞成 | 否 |
| 常规教育案 | > 80% 现任委员参与,≥ 60% 赞成 | 否 |
| 重要案 | > 90% 参与,≥ 70% 赞成 | 否 |
| 重要教育案 | > 90% 现任委员参与,≥ 70% 赞成 | 否 |
| 特别案 | 全体参与,≥ 70% 赞成 | 是,且必须通过(国家级:全国 ≥70% 投票权公民参与 + ≥70% 赞成;省级:本省;市级:本市) |

全部立法表决统一走新 `legislation-vote` sub-pallet,由它按"立法院院制 + 表决类型"内部分流(表决院=立法院下设单院/两院,议员=admins):

- 市立法会(单院):常规/重要 → 单院模式,一段内部表决(委员,按参与率/赞成率计票)。
- 国家立法院 / 省立法院(下设众议会→参议会):常规/重要 → 两院模式,两段顺序内部表决(众议会按表决类型通过 → 参议会按表决类型终审通过),无公投。
- 教育类法案:教委会(发起,内部表决) → 对应立法院终审院(国家/省为参议会,市级为市立法会) → 按常规教育案/重要教育案阈值表决。
- 特别案 + 核心修宪 → 特别案模式:内部阶段(单院=委员;两院=众→参,全员 ≥70% 赞成) + 强制公投阶段(人口快照,国家=全国、省=本省、市=本市,≥70% 参与 + ≥70% 赞成),两阶段都必须通过。

关键:这三种模式(单院 / 两院 / 特别案)全部在 `legislation-vote` 一个 sub-pallet 内实现,复用核心 crate 的提案生命周期、`AdminSnapshot`、人口快照验签 trait、清理、反向索引(第 5 节),只本地写自己的计票账本。现有 internal-vote / joint-vote / election-vote 零改动。

结论(回应"是否只用内部/联合投票"):常规案/常规教育案/重要案/重要教育案只有内部表决(单院一段、两院两段);特别案与核心修宪按宪法第四十五/四十六条、第十九条强制叠加立法公投且通过 → 由 legislation-vote 特别案模式的"内部阶段 + 强制公投阶段"承载。五类档位同壳,不借用也不修改其它投票模块。

### 5. 新增 legislation-vote sub-pallet(本项目核心,additive 不改存量)

不修改 internal-vote / joint-vote / election-vote;新增一个立法专属投票 sub-pallet,把宪法第四十五/四十六条的表决规则集中实现,复用核心 crate 共享基础。

(A) 复用核心 crate(`votingengine`)共享基础设施(`Config: votingengine::Config`,零拷贝):

- 提案生命周期:`Proposals` / `NextProposalId` / `ProposalData` / `ProposalOwner` / `ProposalMeta` / 状态机(`finalize_proposal` / `set_status_and_emit` / `mark_proposal_passed_at`)。
- 管理员快照:`AdminSnapshot`(提案创建时锁定立法机构现任 admins,即现任议员/委员名册——不另建名册)。
- 公投基础:`PopulationSnapshotVerifier` 人口快照验签 trait + `CidEligibility` 资格 trait。
- 到期清理、反向索引(`ProposalsByCid / ByOwner / ByYear`)、互斥锁、ID 生成。

(B) 本 sub-pallet 本地新增(只是计票账本,对标 joint-vote 的 `JointTallies` / `ReferendumTallies` / `UsedPopulationSnapshotNonce`):

- `VoteType=Regular/RegularEducation/Major/MajorEducation/Special`,五类表决类型与投票引擎 `LEG_VOTE_*` 常量一一对应。
- 立法内部表决计票账本:按 `AdminSnapshot` 现任 admins 总数算参与率/赞成率;提前否决只做"现有票数已使赞成永不可能达标"的数学短路,不保留额外反对率条件。
- 两院顺序内部阶段:可配置「机构阶段序列」,单院 = 一段,两院 = 众议会段 → 参议会段,每段独立计票 + 独立阈值。
- 强制公投计票账本:内部阶段全部通过后强制进入(AND,不是否决救济);门槛 ≥70% 参与 + ≥70% 赞成;作用域全国/省/市;复用核心人口快照验签。
- 立法专属 Engine trait(如 `LegislationVoteEngine`)+ 终态回调,供 legislation-yuan 业务壳调用与回写。

(C) 与存量关系:internal-vote / joint-vote / election-vote 零改动零回归。joint-vote 的三储机构加权模式与本 sub-pallet 各管各的,不存在"两套模式并存于一 pallet"的问题。

### 6. 修宪特别约束(第十九条)

- 第一章总则核心条款修改 → 国家立法院特别案(= 必须立法公投)。
- 不可修改条款硬清单:第 1、2、3、17、19、24、34、42 条(单源 `primitives::count_const::IMMUTABLE_CONSTITUTION_ARTICLES`)。
- 其它章节(第二章起)修改 → 国家立法院重要案。
- **上述「章→档位」绑定为代码强制(2026-07-09 落地),不再依赖提案人自选 + 表决机构把关,详见 §6.3。**

#### 6.1 不可修改条款「真不可修改」三层守卫 —— 已落地(2026-06-24)

要求:这 8 条「改代码、改 runtime 升级都改不动,改了只能重新创世」。纯 runtime 校验可被一次 setCode 解除,
故把关必须搬出可升级的 runtime,放到节点共识层并锚定创世。三层纵深:

- **L1 运行时提案守卫(`legislation-yuan::ensure_immutable_preserved`)**:`propose_amend_law` 时逐条逐字比对,
  碰这 8 条即 `ImmutableArticleViolation`。第一线、报错干净,但可被 runtime 升级绕过 → 不是最终保证。
- **L2 节点共识守卫(`node/src/core/constitution/guard.rs::ConstitutionGuard`,最终保证)**:作为独立最外层包装器包住 `NodeGuard<PowBlockImport>`,
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

2026-07-10 第二轮节点加固进一步冻结：manifest 每次全检都必须与 block#0 编码逐字一致；`Law` 内部
`law_id/status/版本指针`、`LawVersion` 内部 `law_id/version/content_hash`、全文条号唯一性、历史版本修改、
warp 版本集合连续性全部由节点原生复核。无 body 的 `ApplyChanges(Changes)` 不再走快路径；无 body 的执行型
导入无法证明后置状态时 fail-closed。启动/warp 枚举真实版本 key，不按不可信 `latest_version` 做超大循环，
避免恶意状态利用守卫制造 CPU DoS。

#### 6.2 加固五项(2026-06-24,卡 `20260624-constitution-immutable-guard`,review 发现的绕过面)

- **H1 守卫补 Law 元数据 + 唯一性**(堵"只校验条文字节"):`check_immutable_articles` 除 8 条条文外,断言 `Laws[0]`
  `tier==Constitution`、`scope==0`、`status!=Repealed`(**不钉 Effective**,放行合法修宪 Pending 窗口)、`houses==创世`,
  并断言 `LawsByScope[宪法][0]==[0]`(挡 migration 新立第二部宪法 / 隐藏 law_id=0)。判别值由 `enum_discriminants_match_node_guard` 测试钉死。
- **H2 warp/状态导入校验**(堵"warp 落到篡改态"):`import_block` 对 `with_state()` 块**提交前**校验。
  ⚠️ **二次 review 修正(P1)**:vendored GRANDPA 在 `inner.import_block` 内即把状态置 finalized 落库,**post-import `KnownBad` 无法回滚**;
  故改为 `verify_imported_state` 从 `params.state_action` 的 `ImportedState` 抽立法院前缀键、**调用 inner 之前**跑全套校验,
  违规/无法抽取 → `KnownBad`(不调用 inner,什么都不落库)。
- **H2.1 导入形态与全历史加固(2026-07-10)**:有 body 时独立预执行；无 body 但携带预计算 delta 时直接检查
  delta；`Execute/ExecuteIfPossible` 缺 body 时 fail-closed；`Skip` 仅允许不导入状态。启动和 warp 要求
  `LawVersions[0]` key 集严格连续为 `1..=latest_version` 并逐版本复核；普通块精确复核 delta 触及的历史版本，
  同时禁止 `latest_version` 回退和超范围隐藏版本。
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

#### 6.3 修宪「章→档位」硬规则 —— 已落地(2026-07-09,卡 `20260709-constitution-amend-tier-hardrule`)

第十九条把修宪分三档:**不可修改条款(禁改)/ 第一章总则核心条款(特别案+公投)/ 第二章起一般条款(重要案)**。
原实现只强制「不可修改条款禁改」与「宪法不得用常规案」,而「核心章→特别案、一般章→重要案」的绑定
仅靠提案人自选 `vote_type` + 表决机构人工把关,代码未强制。本次将该绑定做成硬规则,分两步:

- **判定单源(`primitives::constitution`)**:纯函数 `classify(changed, core, immutable) -> AmendmentScope`
  (`NoChange/ImmutableViolation/CoreChapter/GeneralOnly`),与泛型 `T`、存储解耦,runtime 与节点守卫**共用同一份**,
  靠交叉测试锁死语义一致。核心章 = 第一章总则(`CONSTITUTION_CORE_CHAPTER_INDEX=0`)。

- **第一步 runtime 强制(`legislation-yuan`)**:`propose_amend_law` 与**提交层复校验** `ensure_write_law_version_allowed`
  统一走 `ensure_constitution_amend_ok`:对新旧全文逐条 diff 得改动范围 →
  核心章条款改动必须 `Special`(否则 `CoreClauseRequiresSpecial`)、一般章改动必须 `Major`(否则 `GeneralClauseRequiresMajor`,
  **只许 Major,不许自愿升格 Special**)、空改动拒(`EmptyAmendment`)。两处入口共用,防回调/内部路径绕过。

- **第二步 node 背书(`node/src/core/constitution/guard.rs`,setCode 改不动)**:守卫 `ImmutableReference` 从 block#0 **另派生
  核心章非禁改条款基准** `core_articles`(无需改创世 manifest、无需重新创世);逐块校验 `check_core_chapter_tier`:
  任一创世核心条款相对基准被**修改/删除/移出核心章**,则承载它的版本必须记录 `vote_type==Special`(`LEG_VOTE_SPECIAL=4`,
  由 `enum_discriminants_match_node_guard` 钉死),否则 `CoreClauseNotSpecial(n)` 拒块。使 setCode 无法把核心章修改静默降级为重要案。

- **第三步 公投凭据背书(设计 B,2026-07-09 同批落地)**:仅记录 `vote_type==Special` 可被撒谎的 runtime 伪造,
  故再落一层**永久公投凭据**——`write_law_version` 对核心章改动版本,经 `LegislationVoteEngine::referendum_result`
  取公投计票 `(eligible, yes, no)`、过口径 `primitives::constitution::referendum_passed`,写入 legislation-yuan **永久**
  `ConstitutionAmendmentProof[version]`(不受 votingengine 90 天清理影响);缺失/未过 → `ReferendumProofMissing`/`ReferendumNotPassed`。
  节点 `check_core_referendum_proof` 对核心章有改动的版本(生效/待生效)逐块读该凭据并复核口径,缺/不过 →
  `CoreClauseReferendumMissing/NotPassed` 拒块。**永久存储 + 同 pallet 读**,故无需转移块检测、无跨 pallet 布局漂移、无需重新创世。
  口径单源迁入 `primitives::constitution::referendum_passed`(votingengine `legislation_referendum_final_passed` 改为转发)。

- **第四步 护宪大法官终审凭据背书(第21条,2026-07-09 同批落地)**:第21条要求**一切修宪**(含一般章重要案)最终经
  护宪大法官 4/7 终审才生效——覆盖面比公投更广(公投仅核心章)。同设计 B:`LegislationVoteEngine::guard_review_result`
  取护宪赞成票数(数 `LegGuardSigns` 里 approve=true),`write_law_version` 对**所有** tier=宪法 Amend 版本经口径
  `primitives::constitution::guard_review_passed`(≥4)后写入 legislation-yuan **永久** `ConstitutionGuardVoteProof[version]`;
  缺/不过 → `GuardReviewProofMissing`/`GuardReviewNotPassed`。节点 `check_guard_review_proof` 对**每个** `v>创世` 的修宪版本
  逐块读凭据 + 复核口径,缺/不过 → `GuardReviewMissing/NotPassed(v)` 拒块。阈值单源 `CONSTITUTION_GUARD_APPROVAL_THRESHOLD=4`
  迁入 primitives(legislation-vote 引用之)。**护宪成员真源 = admins-change(`constitution_guard_members()` 查 NJD role=护宪大法官),
  无论普选/互选/联邦特权/阈值票产生,终态都在此真源**——本层锚定它即可,不绑普选(普选生命周期是上游、另议)。

- **天花板(honest)**:节点只读状态、runtime 产出状态,一个完全恶意的 runtime 仍可伪造自洽的通过计票(甚至选民集/护宪成员集);
  故本层是**纵深防御**(抬高伪造成本、抓漏做/半做/只改 vote_type 不伪造计票),**非**对抗完全恶意 runtime 的密码学保证——
  后者的真正门是 setCode 本身受治理闸(joint vote / NRC admin)+ 二进制分发/社会层。真不可伪造的只有冻结到创世的不可修改 8 条。

- 验收:primitives 11(含 constitution 9)+ legislation-yuan 30(三档强制 5 + 公投凭据 fail-closed 1 + 护宪凭据 fail-closed 1)+
  legislation-vote 29 + node 38(含真实 runtime 创世、manifest/身份/哈希/重复条号/历史版本/warp 恶意态)+
  votingengine 回归 + no_std(WASM)+ clippy(零新增告警)+ fmt 全过。

- **follow-up(另窗口评估)**:护宪凭据背书的强度封顶在 admins-change 真源的完整性上,而该真源本身节点层无守卫。
  「admins-change 真源本身能否加锚」已派生独立评估任务(不在本卡范围)。

#### 6.4 admins-change 真源加锚:固定治理骨架守卫(档 A)—— 已落地(2026-07-09,卡 `20260709-governance-skeleton-guard`)

承 §6.3 follow-up。护宪 4/7 终审的可信度封顶在 `AdminAccounts[NJD]` 完整性上,而 §6.3 的 `ConstitutionGuardVoteProof`
只锚了「4」(赞成数),从未锚「7」(法庭规模);且整个 admins-change 真源节点层零守卫,setCode 可任意改写。本档把
**永不合法变更的结构骨架**冻到节点二进制 + 创世,补齐结构性缺口。

- **根本不对称**:宪法不可修改 8 条能冻创世是因为**永不合法变更**;管理员集**天生要变**(普选/互选/联邦特权/阈值票),
  故只能冻**结构**、不能冻**成员**。档 A 冻的是"有几把椅子、椅子归谁管、多少人",不是"椅子上坐着谁"。

- **规格单源**:`primitives::governance_skeleton`(编译常量,genesis 播种 / runtime 校验 / node 守卫三端共读):
  `fixed_institutions()`(NRC/PRC/PRB/NJD 主账户 + 名额 + NJD 护宪席位)、`frg_province_groups()`、
  `NJD_CONSTITUTION_GUARD_SEATS=7`、`KIND_PUBLIC_INSTITUTION/STATUS_ACTIVE` 判别值、`ROLE_CONSTITUTION_GUARD`
  (admin-primitives re-export、创世 role-by-index、守卫三处逐字节共用)。

- **逐块不变式(I1..I7,`node/src/core/node_guard/governance_skeleton.rs`)**:对每个固定机构与 43 个 FRG 省组——
  I1 `AdminAccounts[主账户]`/`FederalRegistryProvinceGroups[省码]` 恒存在;I2 机构码不变;I3 `kind==PublicInstitution`;
  I4 `status==Active`;I5 名额不变(19/9/9/15/5);I6 **NJD 护宪计数恒 7**(补上 §6.3 里没锚的「7」)。判定路径
  由统一 `NodeGuard` 只读执行取后置变更 → 触 `PublicAdmins` 前缀或 `:code` 才全量校验 → 违规 `KnownBad`;
  warp 提交前校验;守卫取数/解码失败 fail-closed 拒块;启动期从 block#0 双锚(创世 state 必须已满足规格)。
  旧治理骨架独立包装器已删除，固定治理骨架改为 `NodeGuard` 内部纯策略；网络导入和挖矿导入均固定为
  `ConstitutionGuard<NodeGuard<PowBlockImport>>`，宪法守卫独立、最外层、最高优先级，其他永久规则不得再新增平行包装器。
  2026-07-10 全节点 PoW 发行作为第二个内部策略接入：`NodeGuard` 共享 finalize 前/后只读执行，从 PoW digest、
  编译期奖励常量、累计审计和 Balances 净变化逐块复算；runtime 升级改变金额、区间、作者或停止规则时，节点按 fail-closed 拒块。
  同日公民认证发行作为第四类内部永久策略接入：身份登记 extrinsic 只生成本块待发凭据，runtime 在同块
  `on_finalize` 实际铸发；节点从首次身份、CID 反向索引、永久/临时双重防重、累计人数和编译期档位常量
  独立复算。全节点与公民奖励汇总进入共享 `FinalizeIssuancePlan`，统一核对账户和总发行，禁止未登记的
  finalize 铸发。省储行利息因现有 Root `force_advance_year` 可合法跳过到期年度，尚不具备“必须发行”的
  永久语义；resolution/onchain 发行属于治理结果，交易费、分账与 PoW 难度也未被正式冻结，均不在本次守卫范围。

- **runtime 侧同步 I6**(`public-admins::ensure_court_composition`):NJD 管理员集变更(propose + 执行终态)强制护宪恰 7,
  消除「runtime 放行、节点拒块」裂缝;新 Error `InvalidCourtComposition`。**等长换人保持 7 席即放行**(不冻成员)。

- **天花板(honest)**:档 A 冻「7 这个数」,不冻「这 7 个人」。挡得住稀释/灌水/删机构/改码/关闭等结构攻击,挡**不住**
  "保持恰 7 人、整体换成攻击者密钥"的成员劫持(节点无独立预言机判合法当选)。成员劫持须档 B(创世根验签链:换届
  由旧护宪 ≥4 人对新集签名、节点自验签),有状态、须改签名模型,**缓做**。不冻**阈值**(固定治理阈值是
  `fixed_governance_pass_threshold` 计票逻辑、不落 state,守卫锚不到)。

- **落地代价**:纯节点二进制 + primitives 只读常量 + public-admins 一条校验;守卫逻辑在 runtime 之外,**无需 migration**;
  含 public-admins 改动按链开发期规则重新创世即可。验收:primitives(governance_skeleton 4)+ admin-primitives 2(判别值/字面量交叉钉死)
  + public-admins 回归 + node(node_guard 11)+ no_std + fmt。

- **遗留**:节点管理员展示解码器(`codec.rs`、`institution_read/chain.rs`)字段序疑似落后于当前 `admin-primitives`
  (缺 `cid_number` + `role_code/role_name/admin_source_ref`),非本卡引入,已派生独立任务核对 deployed↔source 后对齐。

### 7. 宪法迁移(并入本模块)—— 已完成(2026-06-24,卡 `20260624-constitution-migration.md`)

- 宪法纳入本模块,作为 `tier = 宪法` 的最高层级法律,实现「宪法可按条修改(走特别案/重要案)」。
- 已落地:结构化 `章>节>条>款 + 中英双语` 产物 `legislation-yuan/src/constitution.scale` 作为正式创世种子;创世注入为 `law_id=0 tier=宪法 effective_version=Some(1) latest_version=1 pending_version=None status=Effective`。
- 唯一真源 = 链上立法院模块。旧 HTML 文件、旧 runtime API 与以 HTML 为输入的解析脚本均已按禁止兼容硬规则删除。
- 节点桌面端「公民宪法」tab **保持原样式**:`constitution_getDocument` RPC 直接 RAW 读 `LegislationYuan::Laws[0]` 与当前生效版 `LawVersions[0][v]`(不走可升级 runtime API)。`node/src/core/constitution/render.rs` 据链上结构化法律与 `constitution_shell.html` 重建完整 HTML；`guard.rs` 独立承载共识守卫，展示调整不得触碰最高规则执法。

### 8. 上链时机

- 新增 pallet = runtime 变更,新增 `pallet_index=25`(原定 27,2026-07-12 重排)。
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

1. 卡1 新增 legislation-vote sub-pallet(核心,先行):`Config: votingengine::Config` 复用核心共享基础 + 本地五类表决计票账本 + 单院/两院/特别案三模式 + 强制公投 + `LegislationVoteEngine` trait + 测试;internal-vote / joint-vote / election-vote 零改动。
2. 卡2 legislation-yuan 业务壳:数据模型 + 状态机 + `propose_*`(admin 入口) + Executor + runtime API + 不可修改条款硬拒;调 legislation-vote。
3. 卡3 双客户端:CitizenApp(浏览+投票)+ CitizenWallet(decoder+签名,ADR-026 新 op_tag)。
4. 卡4 宪法迁移:HTML → 结构化条文(中英双语),清理旧 include_str!/API。
5. 上链:按 runtime 二次确认,协调重新创世 / setCode。

## 已拍板(2026-06-24)

- 新增立法专属投票 sub-pallet `votingengine/legislation-vote`,不修改 internal-vote / joint-vote / election-vote;立法投票共享投票引擎核心基础设施,只本地存计票账本。
- 议员/委员 = 机构 admins,不另建名册;换届走 admins-change。
- 众议会/参议会 = 立法院下设两院;两院法案 = 两段顺序内部表决(众→参)。
- 特别案/核心修宪 = legislation-vote 特别案模式(内部阶段 + 强制公投),不借用 election-vote。
- 提案恒由 admin 发起;市立法会公民联署门槛为现实前置,不做链上联署入口。
- 投票引擎按宪法完整实现(参与率%/赞成率%/强制公投/行政签署与会签救济/护宪终审)。

## 待确认问题(review 时拍板)

- 两个新 pallet 的 `pallet_index`(业务壳 25 / sub-pallet 26;原暂定 27/28,2026-07-12 号段连续化后重排)。
- 任务卡执行顺序:建议卡1(legislation-vote)先行,业务壳依赖其 Engine trait。
- 上链时机:搭车现有待重新创世队列,还是独立 setCode。
