# 修复 citizenapp 与 citizenwallet 协议升级边界

## 任务目标

只修复 citizenapp 与 citizenwallet 公民钱包中 runtime-upgrade 相关残留。

## 修改边界

- citizenapp 只改 `lib/governance/runtime-upgrade` 和必要治理入口文案。
- citizenwallet 公民钱包只改 signer 文案、注释和对应测试。
- 不修改 `citizenapp/lib/votingengine`。
- 不新增 votingengine 目录。
- 不修改 node。
- 不修改 runtime。
- 不修改 CID。

## 修复原则

- citizenapp 不发起协议升级提案。
- citizenapp 不选择 WASM、不填写升级理由、不获取人口快照、不提交 `propose_runtime_upgrade`。
- citizenapp 只展示协议升级说明、提案详情和保留现有投票入口。
- 用户可见文案统一为“协议升级”。
- citizenwallet 公民钱包保留大 WASM 哈希直签例外，不恢复 runtime-upgrade SCALE decoder。

## 验证要求

- 扫描确认 runtime-upgrade 不再含发起提案、人口快照残留。
- 运行 citizenapp 相关测试。
- 运行 citizenwallet signer 相关测试。

## 完成记录

- 已把 citizenapp 的状态升级入口和相关列表展示文案改为协议升级。
- 已删除 citizenapp 端协议升级提案发起逻辑、WASM 选择逻辑、人口快照调用和 `propose_runtime_upgrade` call 编码测试。
- 已保留协议升级详情展示和现有投票入口，未修改 `citizenapp/lib/votingengine`。
- 已把 citizenwallet 公民钱包协议升级相关显示文案和注释更新为“协议升级”。
- 已补充模块技术文档中的 citizenapp / citizenwallet 边界。
- 2026-05-10 追加修复：citizenapp 协议升级摘要模型与解码已删除业务 `status` 字段，真实状态只读取 `VotingEngine::Proposals.status`。

## 验证记录

- `flutter analyze lib/governance/runtime-upgrade lib/governance/governance_proposals_page.dart lib/citizen/vote/vote_view.dart lib/governance/shared/proposal/proposal_models.dart lib/governance/shared/proposal/proposal_cache.dart lib/governance/organization-manage/institution_detail_page.dart`：通过。
- `flutter test test/governance/runtime-upgrade/runtime_upgrade_service_test.dart`：通过。
- `flutter analyze lib/signer test/signer/payload_decoder_test.dart`：通过。
- `flutter test test/signer/payload_decoder_test.dart`：通过。
- 残留扫描：未发现 `状态升级`、`fetchPopulationSnapshot`、`submitProposeRuntimeUpgrade`、`buildProposeRuntimeUpgradeCallForTest` 等发起提案残留。
