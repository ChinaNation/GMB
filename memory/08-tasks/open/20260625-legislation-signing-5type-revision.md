# 20260625 立法体系修订:签署/会签 + 5类提案 + 删二审 + 命名统一

依据:`memory/04-decisions/ADR-027-legislation-yuan.md`(需同步更新) + 用户 2026-06-25 多轮拍板。
承接已完成的 `20260624-legislation-yuan-and-vote.md`(链端两 pallet)+ `20260624-legislation-dual-client.md`(双客户端,未完)。

## 背景:为何再次修订

链端立法两 pallet(idx 27/28)、宪法迁移、不可修改守卫均已落地。但用户 2026-06-25 明确三块**宪法无、实现也无**的新增需求,且对表决类型做了重构。全部经用户逐条二次确认。

## 已拍板(2026-06-25,逐条确认)

1. **行政首长签署生效**:国家=总统、省=省长、市=市长,在参议会终审/市立法会通过后签署生效。
2. **否决/超时救济**:
   - 市级(单院,无救济):市长否决=否决;30天未表态=通过(生效)。
   - 省/国家级(有救济):省长/总统否决**或**30天超时 → 退回立法院 → 院长+参议长+众议长**三人会签**;三人全同意=生效;任一否决**或**三人30天未完成会签=否决。
   - 特别案例外:特别案经公民投票通过即直接生效、不通过即否决,**任何人不再签署**。
3. **删除常规案二审**:方案B(彻底删)。**本轮只改法案 7 条(44/45/73/75/79/81/118)**;官员任免的 19 条(52/54/56/58/63/65/87/88/92/96/98/99/101/104/107/111/114/133/135)**另立专案**。
4. **提案类型 4→5 类**:常规案/常规教育案/重要案/重要教育案/特别案。教育属性编进 `vote_type`,**不另设内容分类字段**。
   - `VoteType` 枚举:`Regular / RegularEducation / Major / MajorEducation / Special`(重要案 `Important`→`Major` 对齐宪法 "major")。
   - 阈值:常规/常规教育 `>80%参与,≥60%赞成`;重要/重要教育 `>90%,≥70%`;特别 `全员,≥70%+强制公投(全国/省/市≥70%/≥70%)`。
5. **提案机构→表决院路由**(委员/admin 直接提案):
   | 提案机构 | 层级 | 可发起类型 | 表决院序列 |
   |---|---|---|---|
   | 国家众议会 | 国家 | 常规/重要/特别 | [国家众议会→国家参议会] |
   | 国家教委会 | 国家 | 常规教育/重要教育 | [国家教委会→国家参议会] |
   | 省众议会 | 省 | 常规/重要/特别 | [省众议会→省参议会] |
   | 市立法会/市自治会 | 市 | 常规/重要/特别 | [市立法会] 单院 |
   | 市教委会 | 市 | 常规教育/重要教育 | [市立法会] 单院 |
   - 链上校验:提案机构 ⟺ 可发起类型一一对应;表决院由提案机构推导;市级提案方≠表决院(直接进市立法会单院)。
6. **选区**:仅特别案有公民投票,按立法机构 全国/省/市。提案两字段=`title`(提案名称,发起人输入)+`vote_type`(提案类型),链端均已有。
7. **法定代表人**:每个机构设法定代表人=机构首脑(总统府=总统、省政府=省长、市政府=市长、立法院=院长、参议会=参议长、众议会=众议长),且必为该机构 admin 之一。签署人即各机构法定代表人。链上当前**无此字段**,需新增。
8. **命名统一**(全称/简称,全工程零例外;宪法中机构全称首次出现用全称、其后用简称):
   市公民立法委员会/市立法会、国家公民教育委员会/国家教委会、市公民教育委员会/市教委会、市公民自治委员会/市自治会、镇公民自治委员会/镇自治会。
9. 数据结构 `Law`/`LawVersion` 不变,仅改 `VoteType` 枚举。
10. 签署期限=30天,复用 `primitives::count_const::VOTING_DURATION_DAYS=30`。

## 宪法修订(8条双语终稿已确认)

第44/45/73/75/79/81/118/122 条,删二审+5类+签署/会签+教育路由,均以改写现有条文落地、不新增条号。终稿见本卡确认记录(对话 2026-06-25)。任免类二审(19条)本轮不动。

## 执行分卡(本卡统管)

- **A 宪法修订(foundation,先行)**:改 `CitizenConstitution.html`(从 git 8a08acc3^ 恢复) 8 条 + 命名规则 → 跑 `scripts/parse_constitution.py` 重生 `constitution.scale` → 验证 decode + legislation-yuan 测试(款数从 129→132 须同步任何硬编码计数) → 同步节点守卫创世基准(仅非不可修改条款,8 条不可修改条款未动,守卫摘要不受影响)。
- **B 链端**:`votingengine/types.rs` VoteType 5类+阈值纯函数;`legislation-vote` 阈值改、加 `STAGE_LEG_SIGN`/`STAGE_LEG_OVERRIDE` 两阶段+`executive_sign`/`override_sign` extrinsic+30天超时;`legislation-yuan` 提案方↔表决院解耦+路由校验(机构⟺类型);法定代表人字段(`organization-manage`/`admins-change`)+签署人定位。runtime 二次确认 + 守卫范围。
- **C 命名统一**:全工程(链端/CID 机构码/双客户端字面)按全称/简称表零例外。
- **D 双客户端**:CitizenApp 发起(governance/legislation-yuan)+投票(votingengine/legislation-vote)+签署/会签入口;CitizenWallet 注册 pallet 27/28 + decoder(含 executive_sign/override_sign 新 op_tag,ADR-026)+两色拒签。
- **E 收尾**:ADR-027 条号更正(四种表决引用"第18条"→"第44/45条";现已 5 类)、立官员任免二审专案占位卡、文档/注释/残留清理。

## 硬规则约束

runtime 二次确认 / 改宪法走重新创世 + 不可修改守卫 / runtime+扫码签名联动(双客户端) / 投票职责边界 / 禁止兼容 / 彻底改造 / 命名零例外 / 真实运行态验收。

## 进度

- [x] **A 宪法修订 + 重生 scale(2026-06-25 完成并验证)**:`CitizenConstitution.html`(从 git 8a08acc3 恢复)改 8 条法案条文(44/45/73/75/79/81/118/122):删法案二审、4→5 类(常规/常规教育/重要/重要教育/特别)、市长/省长/总统签署款、三人会签救济款、市级教育禁入款、市教委会教育提案权并入。任免类二审(19条)未动。`scripts/parse_constitution.py` 重生 `constitution.scale`(221425B,7章28节140条132款,款 129→132)。验证:`cargo test -p legislation-yuan --manifest-path citizenchain/Cargo.toml` **23/23 过**(constitution_scale_decodes/genesis_seeds_constitution 含内)。不可修改 8 条未动→守卫摘要不受影响。**遗留**:① 命名"全称首次/简称其后"全局规范化(需全局首现分析)留 C 卡;② HTML 源持久化策略待定(单一真源=scale,HTML 作可复算输入,git 历史已陈旧);③ 重新创世/setCode 后真机 QA。
- [~] B 链端(进行中):
  - [x] **B1 VoteType 5类 + 阈值(2026-06-25 完成)**:`votingengine/types.rs` 常量改 `LEG_VOTE_REGULAR=0/REGULAR_EDU=1/MAJOR=2/MAJOR_EDU=3/SPECIAL=4`(删 SECOND_READING)+ 阈值函数教育变体并入同级 + 新增 `STAGE_LEG_SIGN=12`/`STAGE_LEG_OVERRIDE=13` 常量(逻辑待 B2);`legislation-yuan/types.rs` VoteType 5 变体 + as_u8 + is_education;lib.rs 宪法允许类型 Important→Major;测试更新。验证:legislation-vote 12 + legislation-yuan 23 全过,零警告。
  - [x] **B3 法定代表人(2026-06-25 完成,编译干净)**:`votingengine/traits.rs` InternalAdminProvider 加 `legal_representative()` 默认方法(additive,mock 不破);`admins-change` 加 `LegalRepresentatives` 存储 + `legal_representative()` getter(显式指定校验∈现任admins,否则回退 admins[0]=首脑占位)+ `set_legal_representative()` setter(治理用,校验∈admins);`configs/mod.rs` RuntimeInternalAdminProvider 委托。验证:cargo check admins-change+votingengine 9.3s 零错误。**遗留**:治理 setter 的 extrinsic 入口(目前仅 pallet 方法)留 B4/D 接入。
  - [ ] B2 签署+会签状态机(STAGE_LEG_SIGN/OVERRIDE 逻辑 + executive_sign/override_sign + 30天超时 + 提案携带 executive/legislature)— **下一步大块**,需扩 votingengine finalizer trait + dispatch 两新 stage + legislation-vote 状态机 + create 签名
  - [ ] B4 提案方↔表决院解耦 + 路由校验(机构⟺类型) + runtime 装配 + ~12 mock

### B2/B3 待确认设计:签署机构如何识别
签署人=各机构法定代表人,但相关机构**不在 `houses`** 内(houses=表决院=众议会/参议会/市立法会):
- 行政签署人:市级=市政府(市长)、省级=省政府(省长)、国家=总统府(总统)。
- 三人会签(省/国家):立法院(院长)+参议会(参议长)+众议会(众议长);参/众已在 houses,**立法院另需**。
**设计已锁定(2026-06-25 用户确认)**:
1. **提案显式携带签署机构**(同 houses 携带模式):提案除 houses 外携带 `executive=(机构码,账户)`(总统府/省政府/市政府)+ 两院级 `legislature=(机构码,账户)`(国家立法院/省立法院,供院长);参议长/众议长取 houses[1]/houses[0] 机构的法定代表人。客户端提供,链端按 tier/scope 路由校验。
2. **法定代表人=机构指定字段**:institution 加 `legal_representative: AccountId`(必为 admins 之一),经 admins-change 治理设置;签署时查该机构当任法定代表人。
3. **立法院为独立机构**(宪法第71条:国家立法院设院长,由参议员互选):立法院是含众/参的上级独立机构、有自己的法定代表人(院长)。三人会签=立法院(院长)+参议会(参议长)+众议会(众议长)三机构法定代表人。

B2/B3 实现要点:
- 签署状态机:非特别案内部全过 → `STAGE_LEG_SIGN`(executive 机构法定代表人签署,30天)。市级:签署=EXECUTED / 否决=REJECTED / 30天超时=EXECUTED(通过)。省国级:签署=EXECUTED / 否决或30天超时 → `STAGE_LEG_OVERRIDE`(立法院+参议会+众议会三法定代表人,30天):三签同意=EXECUTED / 任一否决或超时=REJECTED。特别案:公投通过即 EXECUTED,不进签署。
- 新 extrinsic:`executive_sign(proposal_id, approve)`、`override_sign(proposal_id, approve)`;新本地账本记签署进度;30天超时走 on_initialize/finalize。
- 提案携带签署机构:扩展 `LegislationVoteEngine::create_legislation_proposal` 签名 + `legislation-yuan` Law/summary 加 executive/legislature 字段。
- [ ] C 命名统一
- [ ] D 双客户端
- [ ] E 收尾(ADR/任免二审占位/文档注释清理)
