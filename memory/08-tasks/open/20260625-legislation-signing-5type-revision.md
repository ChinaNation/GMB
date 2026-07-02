# 20260625 立法体系修订:签署/会签 + 5类提案 + 删除废弃重复表决流程 + 命名统一

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
3. **删除废弃重复表决流程**:当前创世正文的五类表决以第45/46条为主,教育类提案见第75/79条,国家/省签署与三人会签见第100/106条；官员任免流程已在后续批次收口。
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

当前创世正文已落地五类表决、签署/会签与教育路由:第45/46条为五类表决主规则,第75/79条为教育类提案,第100/106条为国家/省签署与三人会签。终稿以 `citizenchain/runtime/public/legislation-yuan/src/constitution.scale` 为唯一真源。

## 执行分卡(本卡统管)

- **A 宪法修订(foundation,先行)**:改 `旧宪法 HTML`(从 git 8a08acc3^ 恢复) 8 条 + 命名规则 → 跑 `scripts/旧 HTML 解析脚本` 重生 `constitution.scale` → 验证 decode + legislation-yuan 测试(款数从 129→132 须同步任何硬编码计数) → 同步节点守卫创世基准(仅非不可修改条款,8 条不可修改条款未动,守卫摘要不受影响)。
- **B 链端**:`votingengine/types.rs` VoteType 5类+阈值纯函数;`legislation-vote` 阈值改、加 `STAGE_LEG_SIGN`/`STAGE_LEG_OVERRIDE` 两阶段+`executive_sign`/`override_sign` extrinsic+30天超时;`legislation-yuan` 提案方↔表决院解耦+路由校验(机构⟺类型);法定代表人字段(`organization-manage`/`admins-change`)+签署人定位。runtime 二次确认 + 守卫范围。
- **C 命名统一**:全工程(链端/CID 机构码/双客户端字面)按全称/简称表零例外。
- **D 双客户端**:CitizenApp 发起(governance/legislation-yuan)+投票(votingengine/legislation-vote)+签署/会签入口;CitizenWallet 注册 pallet 27/28 + decoder(含 executive_sign/override_sign 新 op_tag,ADR-026)+两色拒签。
- **E 收尾**:ADR-027 条号更正(创世正文第45/46、75/79、100/106条;现已 5 类)、官员任免流程专案占位卡、文档/注释/残留清理。

## 硬规则约束

runtime 二次确认 / 改宪法走重新创世 + 不可修改守卫 / runtime+扫码签名联动(双客户端) / 投票职责边界 / 禁止兼容 / 彻底改造 / 命名零例外 / 真实运行态验收。

## 进度

- [x] **A 宪法修订 + 重生 scale(2026-06-25 完成并验证)**:`旧宪法 HTML`(从 git 8a08acc3 恢复)改立法表决、教育路由与签署救济相关条文:删除废弃重复表决流程、4→5 类(常规/常规教育/重要/重要教育/特别)、市长/省长/总统签署款、三人会签救济款、市级教育禁入款、市教委会教育提案权并入。`scripts/旧 HTML 解析脚本` 重生 `constitution.scale`(221425B,7章28节140条132款,款 129→132)。验证:`cargo test -p legislation-yuan --manifest-path citizenchain/Cargo.toml` **23/23 过**(constitution_scale_decodes/genesis_seeds_constitution 含内)。不可修改 8 条未动→守卫摘要不受影响。**遗留**:① 命名"全称首次/简称其后"全局规范化(需全局首现分析)留 C 卡;② HTML 源持久化策略待定(单一真源=scale,HTML 作可复算输入,git 历史已陈旧);③ 重新创世/setCode 后真机 QA。
- [x] **B 链端(2026-06-25 全部完成,测试通过零回归)**:
  - [x] **B1 VoteType 5类 + 阈值(2026-06-25 完成)**:`votingengine/types.rs` 常量改 `LEG_VOTE_REGULAR=0/REGULAR_EDU=1/MAJOR=2/MAJOR_EDU=3/SPECIAL=4`(移除 SECOND_READING)+ 阈值函数教育变体并入同级 + 新增 `STAGE_LEG_SIGN=12`/`STAGE_LEG_OVERRIDE=13` 常量(逻辑待 B2);`legislation-yuan/types.rs` VoteType 5 变体 + as_u8 + is_education;lib.rs 宪法允许类型 Important→Major;测试更新。验证:legislation-vote 12 + legislation-yuan 23 全过,零警告。
  - [x] **B3 法定代表人(2026-06-25 完成,编译干净)**:`votingengine/traits.rs` InternalAdminProvider 加 `legal_representative()` 默认方法(additive,mock 不破);`admins-change` 加 `LegalRepresentatives` 存储 + `legal_representative()` getter(显式指定校验∈现任admins,否则回退 admins[0]=首脑占位)+ `set_legal_representative()` setter(治理用,校验∈admins);`configs/mod.rs` RuntimeInternalAdminProvider 委托。验证:cargo check admins-change+votingengine 9.3s 零错误。**遗留**:治理 setter 的 extrinsic 入口(目前仅 pallet 方法)留 B4/D 接入。
  - [x] **B2 签署+会签状态机(2026-06-25 完成,测试通过)**:votingengine `traits.rs` LegislationProposalFinalizer 加 `finalize_legislation_sign_timeout`/`override_timeout`(默认 Ok)+ create 签名加 executive/legislature;`lib.rs` 两处 finalize dispatch 补 STAGE_LEG_SIGN/OVERRIDE 臂。legislation-vote:LegislationMeta 加 executive/legislature、新增 `LegOverrideSigns` 存储、`advance_to_sign`/`advance_to_override`/`transition_stage`、`do_executive_sign`/`do_override_sign`(法定代表人实时查 + 三人会签去重)、`do_finalize_sign_timeout`(市级超时=通过/省国级→会签)/`do_finalize_override_timeout`(否决)、2 extrinsic(executive_sign idx3/override_sign idx4)+ 4 events + 4 errors。特别案公投通过即生效不签署。
  - [x] **B4 提案解耦 + 路由校验 + 装配(2026-06-25 完成,测试通过)**:legislation-yuan propose_enact/amend/repeal 加 `proposer_body`/`executive`/`legislature` 参;`ensure_legislator` 改对 proposer_body 校验(市级 市自治会/市教委会 委员可提案,表决院恒 houses[0]=市立法会);新增 `ensure_routing`(教育类⟺NED/CEDU、特别案禁教育、院数⟺tier〔宪法豁免〕、两院级 proposer=houses[0])+ `is_education_committee`;`dispatch_to_engine` 透传 executive/legislature。legislation-vote 解耦:删 do_create 硬 NotLegislator + 改自动投票为「发起人∈表决院才投」(市级提案人不在表决院不自投)。runtime:费率 LegislationVote(_) catch-all 已覆盖新 call,InternalAdminProvider mock 默认 legal_representative 不破。**验证**:整 runtime cargo check 绿 + legislation-vote 20 + legislation-yuan 23 测试全过。

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
- [x] **C 命名统一 + 官员任免重复表决流程删除(2026-06-25 完成,与任免流程专案合批改宪法+重生 scale)**:
  - **官员任免重复表决流程删除**:相关任免条款改为常规案驳回后重试口径;中英文同步。**全宪法 0 处废弃重复表决流程(法案条款 Phase A + 任免条款本轮)**。
  - **命名首现严格审计(全机构)**:扫 26 机构对,修 3 处短在前违规(art45 市自治会→市公民自治委员会、art16 镇自治会→镇公民自治委员会、art117 校教委会→学校公民教育委员会),仅替换最早简称那一处、定义句不动;参议会等定义结构产物非违规正确跳过;不碰 8 冻结条款。全工程 5 名写法已一致(零代码改动)。
  - 重生 `constitution.scale` 217626 字节(7章28节140条132款)。**验证(独立复核)**:废弃重复表决流程 0/0、不可修改 8 条逐字节 vs 原始版一致、140 条连续、任免改写通顺、命名修复生效、legislation-yuan 23 测试全过。原文件备份 /tmp。**改后 scale 须重新创世/setCode 生效**。
- [x] **F 护宪大法官修宪最终否决(2026-06-29 完成,2026-07-02 按 7 人制更新,宪法第21条)**:
  - **宪法 HTML 已改**(子代理执行+6验证):新增**第二十一条**(护宪大法官对修宪享最终否决权:重要案总统签署后/特别案公投后→4名及以上护宪大法官赞成生效,未获4名及以上赞成或30天超时否决)、**原21~140顺延为22~141(共141条)**、**第19条冻结条款引用顺延至24/34/42**(创世前重定基准,用户拍板)。
  - **链端护宪状态机已落地验证**:votingengine `STAGE_LEG_CONSTITUTION_GUARD=14` + `InternalAdminProvider::constitution_guard_members()` 生产按国家司法院 `NJD` 的 `AdminProfile.admin_role=护宪大法官` 过滤 7 人 + finalizer/dispatch 加 guard 臂;legislation-vote `needs_guard`字段 + `LegGuardSigns`存储 + `advance_to_guard`/`finalize_or_guard`(4成功终态点 exec签/会签/签署超时/公投 经此分流)+ `do_guard_vote`(4名及以上赞成生效/4名及以上反对否决)+ `do_finalize_guard_timeout`(超时否决)+ `guard_vote`extrinsic(idx5);legislation-yuan `dispatch_to_engine` 算 `needs_guard=tier==宪法` 透传。验证:legislation-vote **29**、internal-vote **88**、公权管理员相关回归、整 runtime cargo check 全过。
  - **护宪守卫改名**:node 宪法守卫→**护宪守卫**(4处字符串)。
  - **成员解析**:护宪大法官归口国家司法院,生产按 `AdminProfile.admin_role=护宪大法官` 过滤 `NJD` admins 且必须恰好 7 人;测试 mock 注 7 人。
  - **统一重生已完成(2026-06-25,见下 §E 统一重生 scale 批)**:`constitution.scale` 已重生(磁盘 219711 字节,7章141条)、不可修改常量已为 `[1,2,3,17,19,24,34,42]`(`count_const.rs:45`)、legislation-yuan 测试已断言 141 条、节点守卫条号已对齐 24/34/42。**HTML 与 scale 已统一到 141,原"有意分叉"状态已闭合。**护宪生产成员解析已闭合:国家司法院进入创世机构,15 名创世治理账户按 7 名护宪大法官、1 名首席大法官、2 名次席大法官、5 名大法官写入角色;国家司法院固定阈值 8/15。
- [~] **D 双客户端**(2026-06-25 核心落地,双端 analyze 0 + CitizenWallet 92 测试;详见 `20260624-legislation-dual-client.md`):CitizenApp 读层补齐(5类VoteType+版本史)+ 类B发起展示页+「发起立法」FAB + 院内/签署/会签/护宪投票服务+查询+表决页(冷钱包QR) + 9 立法 QrActions;CitizenWallet 9 call 解码/标签/两色(零新增op_tag,两色码逐一对齐)。**遗留**:特别案公投(referendum 阶段端到端)未接、vote 页接全局提案列表(kind=2 路由)未做、真机验收待重新创世。原拍板定稿:
  - **立法发起在 CitizenApp 按 B 类实现**:app 内**只投票+查看,不发起**;发起在区块链节点端(node 桌面端,本批不管)。提案分两类——类A(admins-change/organization-manage/personal-manage,app 可提案)、类B(立法/协议升级,只投票查看;点「发起」弹展示页,范式照搬 `runtime_upgrade_page.dart`)。
  - CitizenApp:法律列表/详情/版本史(LegislationApi,已基本就绪)+ 院内投票(cast_house_vote)+ 行政签署(executive_sign)+ 三人会签(override_sign)+ **护宪终审(guard_vote,真机依赖 E2 生产成员)** + 特别案公投(cast_referendum_vote/prepare_population_snapshot)+ 类B 发起展示页。**不做 app 内发起编辑器。**
  - CitizenWallet:pallet 27/28 注册 + call decoder(发起类来自节点端 QR)+ 动作标签 + 两色拒签。**签名零新增 op_tag(2026-06-30 更新)**:cast_house_vote/cast_referendum_vote/prepare_population_snapshot/executive_sign/override_sign/guard_vote 全部走标准交易签名;人口快照只展示 `PopulationScope`。
- [~] **E 收尾**(2026-06-25 重新拆分):
  - **E1+E2 管理员字段扩展 + 护宪成员解析 → 已拆独立卡** `20260625-admin-fields-and-guard-members.md`(用户指定 E1/E2 合并一卡)。admins 从「账户/SS58」扩为 `AdminProfile{account, admin_cid_number, name, admin_role, term_start, term_end, source}`;护宪成员解析=过滤 `NJD` admins 中 `admin_role=护宪大法官` 的 7 人。
  - **E3 重新创世 + 真机 QA → 用户指定本批跳过**,随整套上链统一处理。
  - **[x] E4 残留文档清理(2026-06-25 完成;2026-07-02 复核补正条号)**:① F `[~]` 陈旧 bullet「HTML141 vs scale140 有意分叉」闭合(磁盘已统一 141);② 立法代码注释「省政府」→「省联邦政府」(legislation-vote `lib.rs:94` + `tests/mod.rs:73`,与 china_zf.rs 规范名一致);③ ADR-027 已按创世正文补正条号:五类表决第45/46条,教育第75/79条,签署/会签第100/106条。
  - **[x] 统一重生 scale 批(2026-06-25 完成并验证)**:
    - **章节整体重排已落地**:新序「一总则/二政府/**三教委会/四储委会/五立法院/六司法院/七监察院**」(子代理执行+6验证);教委会72-86/储委会87-97/立法院98-113/司法院114-124/监察院125-141;**不可修改 8 条全在第一章总则不移动,仍 1/2/3/17/19/24/34/42 逐字节不变、第19条引用24/34/42不变(guard-safe)**。
    - **第20条第二款已改(方案甲)**:删「不隶属于任何机构」。
    - **scale 重生 219064 字节**(独立解码验证:7章141条,不可修改8条内容@24/34/42正确);**不可修改常量 [23,33,41]→[24,34,42]**(count_const.rs,节点守卫单源propagate);legislation-yuan **测试140→141**;节点守卫模块doc条号→24/34/42。验证:legislation-yuan **23**+primitives **27**+独立scale解码全过。
    - **命名首现严格审计(2026-06-25,全机构逐字复核完成,结论=已全部合规零改动)**:**关键教训**——首次审计用 `简称 in 文本` 判定,因**简称是全称的子串**(如"国家立法院"⊂"中华民族联邦共和国国家立法院")产生大量假阳性"违规";子串感知(mask 全称后再找裸简称)+ 枚举例外(第8条一府两会三院结构定义段"X由…组成/分为…/隶属于…" + 第22条职位列表 均保留简称)复核后:全部 20 机构的全称都已在各自**首个非枚举独立句**首现,**宪法零改动即合规**。省立法院/省司法院/省监察院 在第8条"隶属于"句的简称按枚举例外保留(用户裁定)。**铁律:机构名首现合规检查必须 mask 全称后查裸简称,否则子串误判。**
    - **储委会节标题改简称(2026-06-25)**:第四章(储委会)第二/三/四节目录由全称改简称——国家公民储备委员会→**国储会**、省公民储备委员会→**省储会**、省公民储备银行→**省储行**(与第一节「储委会联合会议」一致);中英+TOC+节标题同步,正文全称保留;不在第一章故不可修改条款不受影响;scale 重生 **218995 字节**;legislation-yuan 23 测试过。
  - 全链端到端真机 QA(重新创世后)。
  - 小残留:立法代码注释「省政府」对齐宪法「省联邦政府」。
