# 2026-05-11 投票引擎边界彻底修复

## 任务目标

- 将内部投票的阈值、快照、计票、通过/否决判定统一收回 `votingengine/internal-vote`。
- 删除业务模块可显式传入“本次投票通过阈值”的旧接口，不做兼容。
- 注册/注销个人多签和机构多签统一走特别内部投票，由投票引擎按管理员快照生成全员通过条件。
- 个人多签和机构多签的动态阈值由投票引擎保存、校验和更新；`admins-change` 只维护管理员名单。
- 所有内部投票提案创建后，发起人自动记一票赞成，避免第二笔投票交易。
- 更新投票引擎文档、完善中文注释、清理旧接口和旧阈值残留。

## 修改范围

- `citizenchain/runtime/votingengine/src`
- `citizenchain/runtime/votingengine/internal-vote/src`
- `citizenchain/runtime/votingengine/joint-vote/src`
- `citizenchain/runtime/votingengine/citizen-vote/src`
- `citizenchain/runtime/src`
- `citizenchain/runtime/governance`
- `citizenchain/runtime/transaction`
- `citizenchain/runtime/issuance`
- `memory/05-modules/citizenchain/runtime/votingengine`

## 验收标准

- 全仓库不再存在业务模块调用显式投票阈值旧接口。
- 动态阈值只按 `threshold * 2 > admin_count && threshold <= admin_count` 校验。
- 注册/注销生命周期提案由投票引擎写全员阈值快照。
- 一般内部投票读取固定阈值或投票引擎动态阈值。
- 管理员变更提案只由 `admins-change` 路径创建，投票引擎校验新动态阈值。
- 发起提案后发起人的赞成票自动存在。
- `joint-vote::jointreferendum` 保持在 `joint-vote`，不迁移到 `citizen-vote`。

## 完成记录

- `InternalVoteEngine` 已改为语义化接口：一般内部投票、生命周期内部投票、注册主体创建投票、管理员变更内部投票。
- `internal-vote` 已新增 pending/active 动态阈值存储，并在注册、注销、管理员变更执行成功或终态时统一激活、更新或清理。
- `admins-change` 已删除阈值保存和派生职责，只保存管理员集合和生命周期；管理员变更提案把新动态阈值交给投票引擎处理。
- `personal-manage`、`organization-manage`、`duoqian-transfer`、`grandpakey-change`、`resolution-destro`、`resolution-issuance`、`runtime-upgrade` 已同步新接口和发起人自动赞成规则。
- 文档已同步更新到 `memory/05-modules/citizenchain/runtime/votingengine`、`governance/*`、`CROSS_MODULE_INTEGRATION.md`、`wuminapp/governance/GOVERNANCE_TECHNICAL.md`。

## 验证结果

- `cargo fmt`
- `cargo check -p citizenchain`
- `cargo test -p internal-vote -p admins-change -p personal-manage -p organization-manage -p duoqian-transfer -p grandpakey-change -p resolution-destro -p resolution-issuance -p runtime-upgrade -p joint-vote`
- `cargo test -p duoqian-transfer`
- 残留扫描：旧显式阈值接口、`InternalThresholdProvider`、`RuntimeInternalThresholdProvider`、`admins-change::derived_threshold`、`Subjects.threshold` 目标范围内无残留。
- `git diff --check`
