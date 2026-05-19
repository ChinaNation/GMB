任务需求：
- 为 wuminapp 治理机构提案列表和公民-广场提案列表增加本地持久化展示缓存。
- 页面先读本地持久化数据直接显示，再按 TTL、下拉刷新、返回刷新、新区块节流检查等条件低频读取区块链。
- 区分本地持久化展示数据与链上最终真相，减少进入页面时对区块链节点的依赖和读取压力。
- 完成后更新文档、补充必要中文注释并清理残留。

所属模块：
- wuminapp / governance / proposal
- wuminapp / citizen / vote
- wuminapp / transaction / duoqian-transfer

必须遵守：
- 本地持久化只作为提案列表展示读库，不作为投票、执行、提交前的最终真相。
- 涉及投票资格、是否已投票、提案状态提交前校验时，仍必须以链上 runtime storage 为准。
- 不新增 Isar collection schema；复用现有 `AppKvEntity`，避免生成文件和迁移扩大化。
- 不突破 governance 与 transaction 模块边界。

输出物：
- 提案本地持久化读库。
- 治理机构详情页提案列表本地优先显示与低频链上同步。
- 公民-广场提案列表本地优先显示与低频链上同步。
- 文档更新、残留检查和验证记录。

验收标准：
- App 重启后，已同步过的提案列表可以从本地持久化缓存先显示。
- 治理机构详情页不再每次进入都必须链上读取提案索引。
- 公民-广场不再每次进入都必须全量读取三类 org 提案索引。
- 下拉刷新和提案详情返回仍会强制刷新链上数据。
- 新区块监听对链上索引检查做节流，避免每块都全量读取。
- `dart analyze lib test` 和相关治理测试通过。

## 执行记录

- 状态：done
- 本地读库：新增 `ProposalLocalStore`，复用 Isar `AppKvEntity` 持久化 `LocalProposalSummary`、全局治理提案索引和单机构提案索引，不新增 Isar collection schema。
- 治理机构详情页：提案列表先读本机机构索引和摘要；本地为空、索引过期、下拉刷新、详情返回时才读取链上并回写本地。
- 公民-广场：首屏先读本机全局索引和摘要；链上三类 org 索引读取改为 TTL / 下拉刷新 / 详情返回 / 新区块 60 秒节流触发。
- 边界：本地持久化读库只用于列表展示；提案详情点击、投票状态、投票资格和提交前校验仍读取链上真值。
- 文档：已同步 `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md` 与 `memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`。
- 测试：新增 `test/governance/proposal_local_store_test.dart`；`dart analyze lib test`、`flutter test test/governance/proposal_local_store_test.dart`、`flutter test test/governance/governance_list_page_test.dart test/governance/admins-change/institution_admin_service_test.dart test/governance/admins-change/admins_change_codec_test.dart`、`git diff --check` 已通过。
