# 20260624 立法双客户端(CitizenApp + CitizenWallet)

依据:`memory/04-decisions/ADR-027-legislation-yuan.md` + 链端立法卡 `20260624-legislation-yuan-and-vote.md`(已完成)。
本卡是「分两步」的第1步(第2步=宪法迁移卡 + 最后检查收口,另卡)。

## 背景 / 目标

链端立法两步已完成并验证:`legislation-yuan`(pallet_index=27,法律数据/三入口/不可修改硬拒/`LegislationApi` 查询)+ `legislation-vote`(pallet_index=28,单院/两院/特别案强制公投)。本卡把双客户端补齐,让公民在 CitizenApp 浏览法律、发起/投票修法,CitizenWallet 能解码+签名立法二维码。属 chat-protocol §5「runtime + 扫码签名联动」客户端侧收口(runtime 已就绪)。

## 已拍板(2026-06-24)

- 本卡范围:**读 + 投票 + 发起 一次做齐**。
- CitizenApp 读法律走 **runtime API `LegislationApi`**(`list_laws(tier,scope)→[law_id]` / `law(id)→SCALE(Law)` / `law_version(id,ver)→SCALE(LawVersion)`),客户端镜像 Dart 类型解码。
- 立法 propose/cast 均为标准 extrinsic,**无需新 op_tag**;特别案公投的 CID 凭证 + 人口快照复用 joint 公投既有机制。
- 链端 `citizenchain/runtime/` 不改(若发现 LegislationApi 字段不够,另行二次确认后再动 runtime)。

## 一、CitizenApp(在线端,Flutter)

### 1a. 读法律(新建 `lib/legislation/`)
- LegislationApi 客户端封装(state_call → 解码)+ `Law`/`LawVersion`/`Article`/`Clause`/`Item` 镜像 Dart 解码类(与链端 SCALE 布局逐字段对齐)。
- 页面:法律列表(按 tier 宪法/国/省/市 + 行政区 scope 分组)、法律详情(渲染 条/款/项 + 版本号/状态/发布·生效时间)、版本历史。
- 复用 `lib/rpc/`(chain_rpc / onchain / chain_read_cache)。

### 1b. 发起 立法/修法/废法(新建 `lib/governance/legislation-yuan/`)
- 构建 `LegislationYuan(27)` 的 `propose_enact_law(0)` / `propose_amend_law(1)` / `propose_repeal_law(2)` call → 冷钱包扫码签名流程 → 提交。
- 入口仅对立法机构议员/委员(houses[0] 的 admin)可见(按 CID 登录态 + admin 身份门控)。
- 条/款/项结构化编辑器 + 院序列(houses)选择(单院/两院)。
- 范式照搬 `lib/governance/runtime-upgrade/`。

### 1c. 投票(新建 `lib/votingengine/legislation-vote/`)
- 院内表决:`LegislationVote(28).cast_house_vote(1)`,范式照搬 `lib/votingengine/internal-vote/`(query_service + vote_service + proposal_vote_widgets + pending_vote_store)。
- 特别案公投:`cast_referendum_vote(2)` + `prepare_population_snapshot(0)`,复用 `lib/votingengine/joint-vote/` 公投客户端(CID 凭证 + 人口快照),换 pallet/call 索引与阈值展示。
- 提案列表/详情接入(读 votingengine 核心 Proposals + legislation-vote LegMeta 计票账本)。

## 二、CitizenWallet(冷钱包,Flutter)

### 2a. pallet 注册(`lib/signer/pallet_registry.dart`)
- LegislationYuan = 27 → propose_enact_law(0) / propose_amend_law(1) / propose_repeal_law(2)
- LegislationVote = 28 → prepare_population_snapshot(0) / cast_house_vote(1) / cast_referendum_vote(2)

### 2b. 解码 + 标签(`payload_decoder.dart` + `action_labels.dart`)
- 6 个 call 补 decoder 分支,展示关键字段(法律标题/动作/院/表决类型/赞成反对/law_id 等)供核对。
- 补中文动作标签。
- 守两色严格模式铁律:解析失败或 QR action 与 payload 动作不一致 → 红色拒签。

### 2c.(可能)`lib/qr/bodies/` 补立法提案 QR body。

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
- 真实运行态:浏览真链法律(条/款/项 + 版本史)、发起一条修法提案、议员院内投票、(特别案)公投全流程跑通;两色签名核对正确(立法 call 绿色可签、不一致红色拒签)。

## 进度

- [ ] CitizenApp 读法律(lib/legislation)
- [ ] CitizenApp 发起 立/修/废法(governance/legislation-yuan)
- [ ] CitizenApp 投票(votingengine/legislation-vote)
- [ ] CitizenWallet 解码 + 签名(pallet_registry / payload_decoder / action_labels)
- [ ] 双端 flutter analyze + 真实运行态验收

## 第2步(本卡之后,另卡)

宪法迁移卡:`CitizenConstitution.html` → 结构化条文(tier=宪法),清理 `include_str!`/`citizen_constitution_html()` API;最后检查收口(全链立法体系端到端 + 残留扫描)。
