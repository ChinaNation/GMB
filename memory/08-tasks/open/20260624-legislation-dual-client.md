# 20260624 立法双客户端(CitizenApp + CitizenWallet)

依据:`memory/04-decisions/ADR-027-legislation-yuan.md` + 链端立法卡 `20260624-legislation-yuan-and-vote.md`(已完成)。
本卡是「分两步」的第1步(第2步=宪法迁移卡 + 最后检查收口,另卡)。

## 背景 / 目标

链端立法两步已完成并验证:`legislation-yuan`(pallet_index=27,法律数据/三入口/不可修改硬拒/`LegislationApi` 查询)+ `legislation-vote`(pallet_index=28,单院/两院/特别案强制公投)。本卡把双客户端补齐,让公民在 CitizenApp 浏览法律、发起/投票修法,CitizenWallet 能解码+签名立法二维码。属 chat-protocol §5「runtime + 扫码签名联动」客户端侧收口(runtime 已就绪)。

## 已拍板(2026-06-24)

- **与 ADR-028 整合(2026-06-24)**:本卡 CitizenApp 部分(一、)= ADR-028 五子 tab 的「立法 tab 内容 + 统一详情页立法机构提案入口」,**依赖 ADR-028 P1 统一机构层/详情页先落地**,对应整合计划 P3(读法律)/P4(发起)/P5(投票),见 `20260624-citizen-tab-5section-ui.md`。不另起独立立法界面,法律浏览即立法 tab,发起/投票即统一详情页提案入口/列表。CitizenWallet 部分(二、)独立并行(整合计划 P6)。
- **本卡范围(2026-06-25 用户拍板修订)**:CitizenApp **读 + 投票**;**发起不在 app 实现**(发起在区块链节点端,本批不管 node)。提案分两类:类A(admins-change/organization-manage/personal-manage,app 可提案+投票+查看)、**类B(立法/协议升级,只投票+查看;点「发起」弹展示页,照搬 `runtime_upgrade_page.dart`)。立法=类B。**
- **签名零新增 op_tag(2026-06-25 实证)**:全仓签名单一源 `primitives::sign`;cast_house_vote/executive_sign/override_sign/guard_vote=纯 extrinsic 标准交易签名;cast_referendum_vote/prepare_population_snapshot 复用现有 `OP_SIGN_VOTE=0x11`/`OP_SIGN_POP=0x12`。CitizenWallet 仅解码展示+两色拒签,不新增签名域。
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
- **签名零新增 op_tag(2026-06-25 实证)**:cast_house_vote/executive_sign/override_sign/guard_vote 纯 extrinsic 走标准交易签名;referendum/pop 复用 `OP_SIGN_VOTE=0x11`/`OP_SIGN_POP=0x12`。decoder 只负责字段展示与一致性校验,不构造签名域。

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

- [ ] CitizenApp 读法律(lib/legislation,已基本就绪,补版本史 + 5类/新字段解码)
- [ ] CitizenApp 类B 发起展示页(legislation_intro_page,照搬 runtime_upgrade_page)
- [ ] CitizenApp 投票/签署/会签/护宪/公投(votingengine/legislation-vote)
- [ ] CitizenWallet 9 call 解码 + 标签 + 两色拒签(零新增 op_tag)
- [ ] 双端 flutter analyze + 真实运行态验收(护宪真机依赖 E2)

## 第2步(本卡之后,另卡)

宪法迁移卡:`CitizenConstitution.html` → 结构化条文(tier=宪法),清理 `include_str!`/`citizen_constitution_html()` API;最后检查收口(全链立法体系端到端 + 残留扫描)。
