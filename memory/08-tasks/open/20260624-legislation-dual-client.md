# 20260624 立法双客户端(CitizenApp + CitizenWallet)

依据:`memory/04-decisions/ADR-027-legislation-yuan.md` + 链端立法卡 `20260624-legislation-yuan-and-vote.md`(已完成)。
本卡是「分两步」的第1步(第2步=宪法迁移卡 + 最后检查收口,另卡)。

## 背景 / 目标

链端立法两步已完成并验证:`legislation-yuan`(pallet_index=27,法律数据/三入口/不可修改硬拒/`LegislationApi` 查询)+ `legislation-vote`(pallet_index=28,单院/两院/特别案强制公投)。本卡把双客户端补齐,让公民在 CitizenApp 浏览法律、发起/投票修法,CitizenWallet 能解码+签名立法二维码。属 chat-protocol §5「runtime + 扫码签名联动」客户端侧收口(runtime 已就绪)。

## 已拍板(2026-06-24)

- **与 ADR-028 整合(2026-06-24)**:本卡 CitizenApp 部分(一、)= ADR-028 五子 tab 的「立法 tab 内容 + 统一详情页立法机构提案入口」,**依赖 ADR-028 P1 统一机构层/详情页先落地**,对应整合计划 P3(读法律)/P4(发起)/P5(投票),见 `20260624-citizen-tab-5section-ui.md`。不另起独立立法界面,法律浏览即立法 tab,发起/投票即统一详情页提案入口/列表。CitizenWallet 部分(二、)独立并行(整合计划 P6)。
- **本卡范围(2026-06-25 用户拍板修订)**:CitizenApp **读 + 投票**;**发起不在 app 实现**(发起在区块链节点端,本批不管 node)。提案分两类:类A(admins-change/organization-manage/personal-manage,app 可提案+投票+查看)、**类B(立法/协议升级,只投票+查看;点「发起」弹展示页,照搬 `runtime_upgrade_page.dart`)。立法=类B。**
- **签名零新增 op_tag(2026-06-30 更新)**:全仓签名单一源 `primitives::sign`;cast_house_vote/cast_referendum_vote/prepare_population_snapshot/executive_sign/override_sign/guard_vote 全部走标准交易签名。CitizenWallet 仅解码展示 `proposal_id/approve` 或 `PopulationScope` 并执行两色拒签。
- CitizenApp 读法律走 **runtime API `LegislationApi`**(`list_laws(tier,scope)→[law_id]` / `law(id)→SCALE(Law)` / `law_version(id,ver)→SCALE(LawVersion)`),客户端镜像 Dart 类型解码。
- 立法 propose/cast 均为标准 extrinsic,**无需新 op_tag**;特别案公投的 CID 凭证 + 人口快照复用 joint 公投既有机制。
- 链端 `citizenchain/runtime/` 不改(若发现 LegislationApi 字段不够,另行二次确认后再动 runtime)。

## 一、CitizenApp(在线端,Flutter)

### 1a. 读法律(新建 `lib/legislation/`)
- LegislationApi 客户端封装(state_call → 解码)+ `Law`/`LawVersion`/`Article`/`Clause`/`Item` 镜像 Dart 解码类(与链端 SCALE 布局逐字段对齐)。
- 页面:法律列表(按 tier 宪法/国/省/市 + 行政区 scope 分组)、法律详情(渲染 条/款/项 + 版本号/状态/发布·生效时间)、版本历史。
- 复用 `lib/rpc/`(chain_rpc / onchain / chain_read_cache)。

### 1b. 发起 —— 类B,不在 app 实现(2026-06-25 修订)
- **取消 app 内提案编辑器**:立法/修法/废法发起在区块链节点端(node 桌面端)完成。
- 新建 `lib/governance/legislation-yuan/legislation_intro_page.dart`,照搬 `lib/governance/runtime-upgrade/runtime_upgrade_page.dart`(`StatelessWidget` + 信息卡):说明发起位置=电脑节点端,手机端只查看+投票。
- 统一详情页(`InstitutionDetailPage`)立法机构的「发起」入口 → 此展示页,不渲染条款编辑器。
- 可选:把「类A可提案/类B展示页」抽成 `governance/shared` 统一约定(每模块声明 `canProposeOnApp`),协议升级与立法共用展示页机制。

### 1c. 投票(新建 `lib/votingengine/legislation-vote/`)
- 院内表决:`LegislationVote(28).cast_house_vote(1)`,范式照搬 `lib/votingengine/internal-vote/`(query_service + vote_service + proposal_vote_widgets + pending_vote_store)。
- 特别案公投:`cast_referendum_vote(2)` + `prepare_population_snapshot(0)`,复用 `lib/votingengine/joint-vote/` 公投客户端(CID 凭证 + 人口快照),换 pallet/call 索引与阈值展示。
- 提案列表/详情接入(读 votingengine 核心 Proposals + legislation-vote LegMeta 计票账本)。

## 二、CitizenWallet(冷钱包,Flutter)

### 2a. pallet 注册(`lib/signer/pallet_registry.dart`)
- LegislationYuan = 27 → propose_enact_law(0) / propose_amend_law(1) / propose_repeal_law(2)(发起类来自节点端 QR)
- LegislationVote = 28 → prepare_population_snapshot(0) / cast_house_vote(1) / cast_referendum_vote(2) / **executive_sign(3) / override_sign(4) / guard_vote(5)**

### 2b. 解码 + 标签(`payload_decoder.dart` + `action_labels.dart`)
- **9 个 call** 补 decoder 分支,展示关键字段(法律标题/动作/院/5类表决/赞成反对/law_id 等)供核对。
- 补中文动作标签(发起立/修/废法、院内表决、行政签署、三人会签、护宪终审、公投、人口快照)。
- 守两色严格模式铁律:解析失败或 QR action 与 payload 动作不一致 → 红色拒签。
- **签名零新增 op_tag(2026-06-30 更新)**:cast_house_vote/cast_referendum_vote/prepare_population_snapshot/executive_sign/override_sign/guard_vote 纯 extrinsic 走标准交易签名。decoder 只负责字段展示与一致性校验,不构造签名域。

### 2c.(可能)`lib/qr/bodies/` 补立法投票/签署 QR body。发起类 QR 由节点端生成,本卡不做。

## 预计修改目录

- `citizenapp/lib/legislation/`(新建:读法律 + LegislationApi 客户端 + 镜像解码;代码)
- `citizenapp/lib/governance/legislation-yuan/`(新建:发起 立/修/废法 + 提案详情;代码)
- `citizenapp/lib/votingengine/legislation-vote/`(新建:院内表决 + 公投投票;代码)
- `citizenapp/lib/rpc/`(可能补 LegislationApi state_call 封装;代码)
- `citizenwallet/lib/signer/{pallet_registry,payload_decoder,action_labels}.dart`(加 pallet 27/28 + 6 call 解码 + 标签;代码)
- `citizenwallet/lib/qr/bodies/`(可能补立法 QR body;代码)
- `memory/`(本卡 + 文档回写)
- 链端 `citizenchain/runtime/`:不改(已完成)

## 硬规则约束

runtime+扫码签名联动(本卡即客户端侧收口)/ 两色严格拒签 / 禁止兼容 / 彻底改造 / 真实运行态验收 / 命名字面照抄 / 新建目录先确认。

## 验收

- CitizenApp `flutter analyze` 0 error;CitizenWallet `flutter analyze` 0 error。
- 真实运行态:浏览真链法律(章/节/条/款 + 版本史)、议员院内投票、行政签署、三人会签、护宪终审(依赖 E2)、(特别案)公投全流程跑通;类B「发起」点击弹展示页;两色签名核对正确(立法 call 绿色可签、不一致红色拒签)。

## 进度

- [x] CitizenApp 读法律(lib/legislation,2026-06-25):law_models 加 `VoteType` 5 类枚举(+isEducation/label)+ `LawVersion.voteTypeEnum`;修陈旧注释(项已删、不可修改条号→[…24,34,42]);law_reader_page 加版本史下拉(PopupMenu,多版本切换)+ 表决类型徽章。
- [x] CitizenApp 类B 发起展示页 + 入口(2026-06-25):`lib/governance/legislation-yuan/legislation_intro_page.dart`(照搬 runtime_upgrade_page,4 信息卡「发起在节点端」);**入口=机构详情页(institution_detail_page)「发起立法」entry,门控 `_lawTarget(inst)!=null`(立法机构),与「法律原文」并列**——机构 hub 页放操作入口,不放阅读页(见下「入口放置纠正」)。
- [x] CitizenApp 投票/签署/会签/护宪(2026-06-25):`lib/votingengine/legislation-vote/` 新建 `legislation_vote_service.dart`(castHouseVote/executiveSign/overrideSign/guardVote 四纯 extrinsic [28][call][pid][approve],入块后回读对应 storage 确认)+ `legislation_vote_query_service.dart`(LegMeta/LegHouseTally/LegHouseVotesByAdmin/LegReferendumTally/LegOverrideSigns/LegGuardSigns + 核心 Proposals 阶段/状态镜像解码)+ `legislation_vote_page.dart`(按阶段 house/sign/override/guard 渲染动作,复用 ProposalStatusBadge/ProposalVoteActions,冷钱包 QR 签名流照搬 institution_manage_detail_page)。`qr_protocols.dart` 加 9 个立法 QrActions((pallet<<8)|call:0x1b00..02/0x1c00..05)+ fromDecodedAction 映射。
- [x] CitizenWallet 9 call 解码 + 标签 + 两色拒签(2026-06-25,子代理):pallet_registry 注册 27(0/1/2)+28(0..5);payload_decoder 9 分支(含 chapters 递归扫描摘要、全分支 `_hasValidSigningTail`);action_labels 9 中文标签;qr_protocols 两色码与 citizenapp **逐一对齐**((pallet<<8)|call 通用校验);新增 11 立法单测。**零新增 op_tag**。
- [x] 双端 flutter analyze 0 error(CitizenApp 0 / CitizenWallet 0 + 92/92 signer 测试)。
- [ ] **真实运行态验收**(护宪真机依赖 E2 生产成员):待重新创世后整链端到端跑(浏览/投票/签署/会签/护宪/两色)。

### 入口放置纠正(2026-06-25,用户指出)

- **错误**:曾在「法律原文阅读页」(law_list_page)右下角加「发起立法」FAB——**放置严重错误,已撤销**。法律原文页是**查看**法律的地方,不该混入「发起」动作;且立法本就是类B(发起在节点端),手机端更不该在阅读页显眼推「发起」。
- **铁律**:「发起X」类入口只归属**操作/发起页**(机构 hub / 提案发起菜单),绝不放在法律/数据的**阅读/浏览页**。
- **已重接(2026-06-25)**:立法发起入口放到**机构详情页 institution_detail_page** 的 `_legislationProposeEntry()` entry 卡,门控同「法律原文」(`_lawTarget(inst)!=null`,即立法机构 NLG/NRP/NSN/NED/PLG/PRP/PSN/CLEG),与「法律原文」并列 → LegislationIntroPage。机构详情页本就是该机构操作 hub(已有 发起提案/管理员/法律原文 入口),放此处正确。flutter analyze 0。
- 通用教训:加任何入口前先确认该页面的**职责**(阅读页 vs 操作/发起页),不要图省事就近塞按钮。

### 落地记录 / 遗留(2026-06-25)

- **卡内"joint-vote 公投客户端"措辞更新**:公投客户端只提交标准交易,资格和人口分母来自 runtime 链上公民身份。
- **遗留 1:特别案公投(referendum 阶段)端到端未做**——legislation_vote_page 在 referendum 阶段只展示进度 + 提示"请走公民投票入口";cast_referendum_vote/prepare_population_snapshot 的 CID 凭证投票流(复用 api_client)是独立子系统,未接。院内/签署/会签/护宪四阶段已完整。
- **遗留 2:vote 页入口未接全局提案列表**——`legislation_vote_page` 可按 proposalId 直达,但 `governance_proposals_page` 尚未把 legislation(kind=2)路由进列表;当前入口仅 law_list 的「发起立法」展示页。接 kind=2 路由 = 下一步小集成。

## 第2步(本卡之后,另卡)

宪法迁移卡:`旧宪法 HTML` → 结构化条文(tier=宪法),清理 `include_str!`/`旧宪法 HTML API()` API;最后检查收口(全链立法体系端到端 + 残留扫描)。
