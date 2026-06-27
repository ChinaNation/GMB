# 20260624 立法院模块 + 立法投票模块(两步共享卡)

依据:`memory/04-decisions/ADR-027-legislation-yuan.md`。
分两步实现,共享本卡。第1步做完验证通过后再进第2步。

## 背景 / 目标

- 法律从"写死在 runtime 代码"改为结构化上链,改法 = 发交易(投票通过),不再 setCode。
- 公民在 CitizenApp 可查看法律、可投票修法。
- 严格按公民宪法第三章(立法院)与第十七~十九条落地:立法权仅国家立法院 / 省立法院 / 市立法会;议员/委员 = 机构 admins;四种表决(常规/重要/二审/特别)。

## 总体架构(两个新 pallet)

- 业务壳 `citizenchain/runtime/public/legislation-yuan`(pallet_index=27,MODULE_TAG=b"leg-yuan"):法律数据 + 状态机 + 提案入口 + 通过回调 + 查询 API。
- 投票 sub-pallet `citizenchain/runtime/votingengine/legislation-vote`(pallet_index 暂 28):立法专属投票,复用投票引擎核心共享基础,只本地存计票账本;不改 internal-vote / joint-vote / citizen-vote。
- 解耦:业务壳 `Config` 注入 `type LegislationVoteEngine`,第1步装 `()`(返 NotConfigured),第2步装 `LegislationVote`。

## 第1步:立法院模块(legislation-yuan 业务壳)

范围:法律数据模型 + 状态机 + 三提案入口(立法/修法/废法) + 通过回调写入逻辑 + 不可修改条款硬拒 + 查询 API + `LegislationVoteEngine` 接口(trait + `()` 默认)。第1步独立编译、独立单测,不依赖第2步;`propose_*` 调引擎装 `()`,投票端到端流程留第2步打通。

任务清单:

1. 新建 crate `governance/legislation-yuan`(Cargo.toml + src/lib.rs + src/types.rs + src/executor.rs + src/tests/)。
2. 数据模型(types.rs):Tier(宪法/国/省/市)、LawStatus(Draft/Voting/Pending/Effective/Superseded)、VoteType(常规/重要/二审/特别)、Article/Clause/Item(条/款/项)、Law、LawVersion(整部全文快照 + content_hash + published_at + effective_at)。
3. Storage:NextLawId、Laws、LawVersions、LawsByScope(列表索引)、PendingActivation(生效调度)。in-flight 提案载荷沿用 votingengine ProposalData/ProposalObject(对标 runtime-upgrade,不本地存)。
4. 提案入口 propose_enact_law / propose_amend_law / propose_repeal_law:校验 origin 为 owner_body 机构 admin(走 votingengine InternalAdminProvider)→ 校验 tier/vote_type → 不可修改条款硬拒 → 编码 MODULE_TAG 载荷 → 调 `T::LegislationVoteEngine::create_legislation_proposal`。
5. 通过回调 executor:按 MODULE_TAG 认领 → approved 写新 LawVersion + current_version+1 + 入 PendingActivation;否决丢弃。拆 `write_law_version` 纯写入 helper 供第1步单测。
6. on_initialize:到 effective_at 把 Pending 翻 Effective,旧版本转 Superseded。
7. 不可修改条款:`primitives` 单一源 `IMMUTABLE_CONSTITUTION_ARTICLES = [1,2,3,17,19,23,33,41]`;tier=宪法且命中即 reject。
8. runtime 查询 API(apis.rs):list_laws / get_law / get_law_version。
9. runtime 装配:construct_runtime 注册 idx=27;configs `type LegislationVoteEngine = ()`;tests/cases.rs MODULE_TAG 唯一性。
10. 接口 trait:votingengine/traits.rs 加 `LegislationVoteEngine` + `()` 默认(additive,不动三 sub-pallet)。
11. 第1步测试:数据模型 round-trip / 状态机 / on_initialize / 不可修改条款逐条拒绝 / 非 admin 拒绝 / 合法提案到引擎返回 NotConfigured / write_law_version helper。

第1步验收:`cargo check -p legislation-yuan` + `cargo test -p legislation-yuan` 全过;runtime 整体 `cargo check` 过。

## 第2步:立法投票模块(legislation-vote sub-pallet)

**定稿设计(2026-06-24,精读核心后):legislation 作为投票引擎「头等模式」,需扩展核心 crate(不是纯加 sub-pallet)。三个投票 sub-pallet 逻辑零改动,但核心按 kind/stage 硬编码分发,未知 kind 直接 Err,故必须扩展核心。** 用户已拍板:① 认可按头等模式扩展核心;② 院结构由提案携带。

### 2a. 核心扩展(votingengine crate,additive)
- `src/types.rs`:`PROPOSAL_KIND_LEGISLATION=2`、`STAGE_LEG_HOUSE=10`、`STAGE_LEG_REFERENDUM=11`、VoteRule + 立法公投阈值纯函数(全整数,按宪法精确端点)。
- `src/traits.rs`:`LegislationProposalFinalizer` / `LegislationCleanupHandler` / `LegislationVoteResultCallback`(+ `()` 默认)。
- `src/lib.rs`:`Config` 加 `LegislationFinalizer` / `LegislationCleanup` / `LegislationVoteResultCallback`;分发分支补 legislation arm(`invoke_execution_callback` / `can_cancel...` / `notify_execution_failed_terminal` 按 kind;超时 finalize 按 stage;cleanup 按 kind)。
- **所有 `votingengine::Config` 实现(runtime + 各 pallet 测试 mock 约 12 处)补 3 个关联类型;mock 一律装 `()`**(机械)。

### 2b. 新 sub-pallet `votingengine/legislation-vote`(pallet_index=28)
- `Config: votingengine::Config`,复用核心 Proposals/allocate_proposal_id/AdminSnapshot/snapshot_institution_admins/schedule_proposal_expiry/set_status_and_emit/register_proposal_data/PopulationSnapshotVerifier/CidEligibility。
- 本地账本:`LegMeta`(vote_type/mode/houses/current_house/referendum_required/scope)、`LegHouseTally`+`LegHouseVotesByAdmin`、`LegReferendumTally`+`LegReferendumVotesByBindingId`、`UsedSnapshotNonce`+`PendingPopulationSnapshots`。
- 三模式:单院(市)/ 两院顺序(众→参;教委会→参议会)/ 特别案(内部全过→强制公投)。
- extrinsics:`prepare_population_snapshot`(特别案)/ `cast_house_vote` / `cast_referendum_vote`。
- trait impl:`LegislationVoteEngine`(create)、`LegislationProposalFinalizer`、`LegislationCleanupHandler`。
- VoteRule 阈值按 AdminSnapshot 现任 admins 总数算,投票期满 finalize 统一计票(可选赞成不可能时提前否决)。

### 2c. 院结构=提案携带(用户拍板)
- 扩展第1步 `LegislationVoteEngine::create_legislation_proposal` 签名携带院列表 `houses=[(code,account)]`;`legislation-yuan` 的 `Law` / `LawProposalSummary` 加院组成字段;`propose_*` 入口收集院列表。
- 单院=1 项,两院=[众议会, 参议会](发起院在前、终审院在后),教委会模式=[教委会, 参议会]。

### 2d. runtime 装配
- 注册 `LegislationVote` idx=28;`legislation_yuan::Config::LegislationVoteEngine` 由 `()` 换 `LegislationVote`;`votingengine::Config` 补 `LegislationFinalizer=LegislationVote` / `LegislationCleanup=LegislationVote` / `LegislationVoteResultCallback=LegislationYuan`;费率 cast_house_vote/cast_referendum_vote/prepare_population_snapshot → VoteFlat。

### VoteRule 阈值(宪法第十八条,精确端点)
- 常规案 `casted*100 > total*80` 且 `yes*100 ≥ casted*60`;重要案 `>90% / ≥70%`;二审 `casted==total` 且 `yes*100 ≥ total*50` 且 `no*100 < total*20`;特别案内部 `casted==total` 且 `yes*100 ≥ total*70`;特别案公投 `(yes+no)*100 ≥ eligible*70` 且 `yes*100 ≥ (yes+no)*70`。

### 第2步落地进度

- [x] **Phase A 核心扩展(2026-06-24 完成)**:`votingengine/src/types.rs`(PROPOSAL_KIND_LEGISLATION=2 / STAGE_LEG_HOUSE=10 / STAGE_LEG_REFERENDUM=11 / LEG_VOTE_* + `legislation_house_final_passed`/`legislation_house_decided`/`legislation_referendum_final_passed` 纯函数 + PendingCleanupStage 两阶段);`traits.rs`(LegislationProposalFinalizer / LegislationCleanupHandler / LegislationVoteResultCallback + `()` 默认);`lib.rs`(Config +3 类型 + 两处 finalize 按 stage 加臂 + 三处回调按 kind 加臂 + 清理状态机插 legislation 两阶段 + FinalCleanup 补 cleanup_legislation_terminal);**11 处 `votingengine::Config` 实现(runtime + 10 mock)补 3 类型暂装 `()`**。验收:`cargo check -p votingengine`/`-p citizenchain` 通过,legislation-yuan 14 + grandpakey 17 测试无回归(行为中性,legislation kind 尚未被创建)。
- [x] **Phase B 新 sub-pallet `votingengine/legislation-vote`(2026-06-24 完成)**:Cargo.toml + src/{lib.rs,weights.rs,tests/}。本地账本 LegMeta/LegHouseTally/LegHouseVotesByAdmin/LegReferendumTally/LegReferendumVotesByBindingId/UsedSnapshotNonce/PendingPopulationSnapshots;三 extrinsic(prepare_population_snapshot/cast_house_vote/cast_referendum_vote);三模式(单院/两院顺序众→参/特别案强制公投);实现 LegislationVoteEngine(create,院携带)+ LegislationProposalFinalizer + LegislationCleanupHandler;MAX_HOUSES 单一源 votingengine。
- [x] **Phase C legislation-yuan 院携带(2026-06-24 完成)**:Law/LawProposalSummary `owner_body+owner_code` → `houses: HousesOf`(单一真源,houses[0]=发起院);propose_enact_law 入参 houses;ensure_legislator 校验 houses[0];dispatch_to_engine 传 houses;新增 EmptyHouses 错误;加 `impl LegislationVoteResultCallback for Pallet`(回调接 apply_legislation_vote_result);测试 houses() helper。
- [x] **Phase D runtime 装配(2026-06-24 完成)**:construct_runtime 注册 LegislationVote idx=28;configs `legislation_yuan::Config::LegislationVoteEngine = LegislationVote` + `votingengine::Config` 三类型改接(LegislationVoteResultCallback=LegislationYuan / LegislationFinalizer=LegislationVote / LegislationCleanup=LegislationVote)+ impl legislation_vote::Config;费率 `RuntimeCall::LegislationVote(_) => VoteFlat`;Cargo.toml/workspace/std 注册。

第2步验收(2026-06-24 通过):`cargo check -p votingengine`/`-p legislation-vote`/`-p legislation-yuan`/`-p citizenchain` 全过(std + no_std);测试 legislation-vote 12(纯函数阈值 + 单院通过/反对超限提前否决/两院推进/特别案→公投端到端)+ legislation-yuan 14 + internal-vote 87 + grandpakey 17 无回归;runtime 测试二进制编译 + MODULE_TAG 唯一性通过。
真实运行态(整链节点端到端)留待整套上链(重新创世/setCode)后由 user 验证;双客户端(CitizenApp 浏览+投票 / CitizenWallet 扫码签名)= 另卡。

## 硬规则约束

投票职责边界(立法投票一律走引擎)/ runtime 二次确认 / runtime+扫码签名联动(双客户端另卡)/ 禁止兼容 / 彻底改造 / 真实运行态验收。

## 预计修改目录

- `citizenchain/runtime/public/legislation-yuan/`(第1步新建,核心)
- `citizenchain/runtime/votingengine/src/traits.rs`(第1步加引擎 trait;第2步加回调装配)
- `citizenchain/runtime/votingengine/legislation-vote/`(第2步新建)
- `citizenchain/runtime/primitives/`(不可修改条款常量单一源)
- `citizenchain/runtime/src/lib.rs` / `src/configs/mod.rs` / `src/apis.rs` / `src/tests/cases.rs`(注册 + 装配 + 查询 API + 唯一性)
- `memory/04-decisions/ADR-027` / `memory/05-modules/`(文档回写)

## 进度

- [x] 第1步:立法院模块(2026-06-24 完成)
- [x] 第2步:立法投票模块(2026-06-24 完成,Phase A/B/C/D 全落地)

## 后续(本卡外)

- 双客户端联动卡:CitizenApp(法律列表/详情/版本史/投票 + 调 LegislationApi)+ CitizenWallet(修法/投票二维码 decoder + 签名,ADR-026 新 op_tag)。
- 宪法迁移卡:CitizenConstitution.html → 结构化条文(tier=宪法),清理 include_str!/citizen_constitution_html() API。
- 上链:新增 pallet_index=27/28 随重新创世或 setCode 生效;真实整链端到端验收。
- 待细化:议员名册若需换届频繁更新,确认 admins-change 路径覆盖立法机构;市立法会公民联署门槛(现实前置)是否需链上辅助。

## 第1步落地记录(2026-06-24)

新增/改动:

- 新建 crate `runtime/public/legislation-yuan/`:`Cargo.toml` + `src/{lib.rs,types.rs,weights.rs,tests/{mod.rs,cases.rs}}`(executor 逻辑并入 lib.rs,未单建 executor.rs)。
- `votingengine/src/traits.rs`:加 `LegislationVoteEngine` trait + `()` 默认(additive,三 sub-pallet 零改)。
- `primitives/src/count_const.rs`:加 `IMMUTABLE_CONSTITUTION_ARTICLES = [1,2,3,17,19,23,33,41]` 单一源。
- `primitives/src/genesis.rs`:加 `LegislationApi`(list_laws/law/law_version,返回 Vec<u64> / SCALE 字节)。
- `runtime/src/lib.rs`:construct_runtime 注册 `LegislationYuan` pallet_index=27。
- `runtime/src/configs/mod.rs`:装配 `legislation_yuan::Config`(`LegislationVoteEngine = ()`)+ 边界常量 + 费率分类 `RuntimeCall::LegislationYuan(_) => VoteFlat`。
- `runtime/src/apis.rs`:实现 `LegislationApi`。
- `runtime/src/tests/cases.rs`:MODULE_TAG 唯一性表加 `legislation_yuan`(9 项)。
- Cargo workspace + runtime Cargo.toml + std feature 注册新成员。

数据模型:Tier(宪法/国/省/市)/LawStatus(Pending/Effective/Repealed)/VoteType(常规/重要/二审/特别)/Article(number+title+clauses)/Clause/Item/Law/LawVersion(整部全文快照+content_hash)。三入口 propose_enact/amend/repeal_law 只认机构 admin(走 votingengine InternalAdminProvider);宪法不可修改条款硬拒;宪法只能特别案/重要案;宪法不可整体废止。in-flight 提案载荷存 votingengine ProposalData/Object(对标 runtime-upgrade)。

验收:`cargo test -p legislation-yuan` 14 passed / 0 warning;`cargo check -p legislation-yuan --no-default-features`(no_std/WASM)通过;`cargo check -p citizenchain` 通过;runtime 测试二进制编译通过 + `governance_module_tags_are_globally_unique` 通过;`cargo fmt` 干净。

第2步前置提醒:真实引擎接入后,`votingengine::Config::MaxProposalDataLen` 需容纳 LawProposalSummary(含 title 256B),`MaxProposalObjectLen` 需容纳整部条文;legislation-yuan 的 `apply_legislation_vote_result` 回调待第2步接入 votingengine 回调装配后才被触发。上链:新增 pallet_index=27 需随重新创世或 setCode 生效。
