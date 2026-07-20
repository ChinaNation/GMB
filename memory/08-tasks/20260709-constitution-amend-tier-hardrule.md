# 任务卡：宪法修改「章→档位」分类硬规则(第十九条落地)

## 任务需求

把公民宪法第十九条的修宪分档规则从「提案人自选档位」升级为代码强制,并加节点守卫背书,防 setCode 削弱:

- 第一章总则(核心章)除不可修改条款外的条款 → 修改必须走特别案 Special(触发强制公投)。
- 第一章总则以外(第二章起)的一般条款 → 修改必须走重要案 Major(只许 Major,不许自愿升格 Special)。
- 不可修改条款(第 1,2,3,17,19,24,34,42 条)→ 禁改(既有,保留)。

分两步:
1. runtime 侧在 `legislation-yuan::propose_amend_law` 按新旧宪法逐条 diff 判定改动范围,强制匹配档位。
2. node 守卫扩展:创世 manifest 冻结核心条款集,节点逐块背书「核心条款改动必须经特别案 + 合格公投凭据」,`setCode` 改不动。

策略定案:GeneralOnly 只许 Major,禁止自愿升格 Special。

## 预计修改目录

- `citizenchain/runtime/public/legislation-yuan/`
  - 用途:新增 `amend_scope.rs` 分类单源;`propose_amend_law` 接入强制;`Error` 变体;创世 manifest 增核心条款集;测试。
  - 边界:只处理宪法修改档位强制,不扩展立法流程、不改投票引擎职责。
  - 类型:runtime 代码与测试。

- `citizenchain/runtime/primitives/`
  - 用途:加核心章索引常量 `CONSTITUTION_CORE_CHAPTER_INDEX`,供 runtime 与 node 共用。
  - 边界:只加常量,不改既有派生/发行/费率原语。
  - 类型:共享常量。

- `citizenchain/node/src/core/`
  - 用途:宪法守卫扩核心章背书(`ImmutableReference`/`GuardError`/`check` + 创世交叉校验 + 测试)。
  - 边界:只加守卫分支,不改导入链路业务逻辑。
  - 类型:节点代码与测试。

- `memory/04-decisions/`
  - 用途:ADR-027 补第十九条章→档位强制与守卫背书口径。
  - 边界:只更新当前目标态,不保留旧兼容叙述。
  - 类型:文档更新。

- `memory/05-modules/` 与 `memory/08-tasks/`
  - 用途:模块文档同步 + 任务卡验收记录。
  - 类型:文档。

## 验收要求

- `cargo test -p legislation-yuan`
- `cargo test -p legislation-vote`(如受影响)
- `cargo check -p legislation-yuan --no-default-features`(WASM/no_std)
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`
- `cargo fmt --check`(两侧)
- 链开发期:改创世 manifest 需重新创世;重生后重跑 CitizenApp 机构注册表生成器。

## 进度

- [x] 建卡
- [x] 第一步 runtime 强制
- [x] 第二步 node 守卫背书
- [x] 构建 + 测试
- [x] 更新文档 + 完善注释 + 清理残留

## 执行结果(2026-07-09)

设计优化:判定单源改放 `primitives::constitution`(runtime 与 node 共用),优于原计划的 legislation-yuan 私有模块。
node 侧改为从 block#0 **另派生核心章基准**(`ImmutableReference.core_articles`),**无需改创世 manifest、无需重新创世**——比原计划更省。

改动落点:
- `primitives/src/constitution.rs`(新)+ `lib.rs`:`classify`/`AmendmentScope`/`CONSTITUTION_CORE_CHAPTER_INDEX` + 6 单测。
- `legislation-yuan/src/lib.rs`:`ensure_constitution_amend_ok`(diff→档位强制)接入 `propose_amend_law` 与提交层复校验;
  新 Error `CoreClauseRequiresSpecial/GeneralClauseRequiresMajor/EmptyAmendment`;删旧 `ensure_immutable_preserved`(重构为 `ensure_immutable_articles_unchanged`)。
- `legislation-yuan/src/tests/`:两章构造 helper + 三档强制 5 用例 + `enum_discriminants` 钉 `VoteType::Special==4`。
- `node/src/core/constitution.rs`:`ImmutableReference.core_articles` + `check_core_chapter_tier` + `MLawVersionHead` 补 `content_hash/vote_type` + `GuardError::CoreClauseNotSpecial` + 4 用例。
- 文档:ADR-027 §6 清过期不可修改清单(23/33/41→24/34/42)+ 新增 §6.3;PRIMITIVES_TECHNICAL 补 constitution.rs。

策略定案:GeneralOnly 只许 Major(不许自愿升格 Special)。

验收(全过):primitives 8(constitution 6)/ legislation-yuan 28(新 5)/ node 26(新 4)/ no_std WASM 编过 / clippy 零新增告警 / fmt。

## 第三步:公投凭据背书(设计 B,2026-07-09 同批落地)

「只记录 vote_type==Special」可被撒谎的 runtime 伪造 → 再落一层**永久公投凭据**:

- `primitives::constitution::referendum_passed`(口径单源,votingengine `legislation_referendum_final_passed` 改转发)。
- `votingengine` 核心通过 proposal 人口快照提供合资格人口总数；`LegislationVoteEngine` trait 加 `referendum_result(proposal_id)` 只读查询（`()` 实现返回 None）；legislation-vote 实现读取快照分母与 `LegReferendumTally`。
- legislation-yuan 新**永久** StorageMap `ConstitutionAmendmentProof: version→(eligible,yes,no)`;`write_law_version` 对核心章改动版本取 `referendum_result` + 过口径后写入;缺/不过 → `ReferendumProofMissing`/`ReferendumNotPassed`。
- node `check_core_referendum_proof`:核心章有改动的版本逐块读凭据 + 复核口径,缺/不过 → `CoreClauseReferendumMissing/NotPassed` 拒块。storage_key 加 `constitution_amendment_proof`。

设计要点:永久存储(不受 votingengine 90 天清理影响)+ 同 legislation-yuan pallet 读 → **无转移块检测、无跨 pallet 布局漂移、无需重新创世**。

天花板(honest):完全恶意 runtime 仍可伪造自洽通过计票 → 本层是纵深防御,非密码学保证;真正门是 setCode 治理闸 + 社会层。

验收:primitives 10 / legislation-yuan 29 / legislation-vote 29 / node 28 / votingengine 回归 / no_std / clippy 零新增 / fmt 全过。链端非 breaking(纯新增 storage + trait 方法,创世未动,无需重新创世)。

## 第四步:护宪大法官第21条终审凭据背书(设计 B 同构,2026-07-09 落地)

第21条:**一切修宪**(含一般章重要案)最终须护宪大法官 4/7 终审 —— 覆盖面比公投广(公投仅核心章)。

- `primitives::constitution`:`guard_review_passed(approve)`(≥4)+ 常量 `CONSTITUTION_GUARD_APPROVAL_THRESHOLD=4`(legislation-vote 引用之,单源)。
- `LegislationVoteEngine::guard_review_result(id) -> Option<u32>`(数 `LegGuardSigns` approve=true;+`()` impl None);legislation-vote 实现。
- legislation-yuan 永久 `ConstitutionGuardVoteProof: version→approve`;`write_law_version` 对**所有** tier=宪法 Amend 写入(过口径);`GuardReviewProofMissing`/`GuardReviewNotPassed`。
- node `check_guard_review_proof`:对**每个** `v>创世` 修宪版本逐块读凭据+复核口径,缺/不过→`GuardReviewMissing/NotPassed(v)` 拒块;storage_key 加 key。

要点:护宪成员真源 = admins-change(`constitution_guard_members()` 查 NJD role=护宪大法官),**无论普选/互选/联邦特权/阈值票产生,终态都在此真源** → 本层锚定它即可,**不绑普选**(普选生命周期是上游、另议)。

验收:primitives 11 / legislation-yuan 30 / legislation-vote 29 / node 30 / votingengine 回归 / no_std / clippy 零新增 / fmt 全过。非 breaking、无需重新创世。

## 遗留 follow-up(另窗口)

护宪凭据背书的强度封顶在 **admins-change 真源**的完整性上,而该真源本身节点层无守卫。「admins-change 真源本身能否加锚」已派生**独立评估任务**(spawn_task,新窗口),不在本卡范围。
