# CitizenApp 立法表决页接线（修产品假入口）

任务需求：`LegislationVotePage`（454 行成品）从未接线，而提案入口 `ProposalKind.legislation` 为 `enabled: true` 且文案承诺「本端查看 + 投票」，形成正在生效的假入口。按甲案从立法链路接入表决页。
所属模块：citizenapp（Mobile）— 纯前端接线，不改链端、不新增链读原语。

## 问题定性（非死代码清理）

| 位置 | 现状 |
|---|---|
| `lib/citizen/proposal/proposal_registry.dart:221` | `ProposalKind.legislation` = `enabled: true`，`voteEngine: 'LegislationVote'` |
| 同文件注释 | 「禁用能力不展示，避免产生假入口」— 自陈的设计契约 |
| `lib/citizen/proposal/proposal_entry_page.dart:334` | 卡片文案「立法 / 修法 / 废法在电脑节点端发起，**本端查看 + 投票**」，但 `onTap` 只跳 `LegislationIntroPage`（静态介绍页，无任何出口） |
| `lib/votingengine/legislation-vote/legislation_vote_page.dart` | 454 行完整实现（真 service + QR 冷签），全仓 0 引用 |

用户已定：**走甲案（接线）**，不删页面、不降级 `enabled`。

## 可行性已核实（无需新增链读能力）

链端 `legislation-vote` 走共享核心登记，而非私有索引：

- `runtime/votingengine/legislation-vote/src/lib.rs:417,469` → `votingengine::Pallet::register_proposal_data(...)`
- `runtime/votingengine/legislation-vote/src/weights.rs:48` → `VotingEngine::ActiveProposalsBySubject` (r:1 w:1)

因此现有 `ProposalQueryService.fetchActiveProposalIds(institution)`（读 `VotingEngine::ActiveProposalsBySubject[subjectKey]`）**已覆盖立法提案**，只需按 module_tag / kind 过滤后路由。

`LegislationVoteQueryService` 自身只提供按 `proposalId` 的单条查询（无列表 API），列表能力由上述共享核心入口提供——这是既有模式，`proposal_entry_page.dart:386` 已在用。

## 接线目标签名

```dart
LegislationVotePage({
  required int proposalId,
  required List<WalletProfile> adminWallets,   // 由上层注入的可签名管理员钱包
  LegislationVoteService? voteService,
  LegislationVoteQueryService? queryService,
})
```

## 待办

- 确定列表落点：立法 Tab（`lib/citizen/legislation/legislation_tab.dart`）新增「待表决」区，或立法机构详情内提案区。
- 用 `fetchActiveProposalIds` + `fetchProposalMeta` 过滤出 LegislationVote 提案，渲染列表 → 点击进 `LegislationVotePage`。
- `adminWallets` 复用现有管理员钱包注入链路（对齐 `proposal_entry_page` / `institution_manage_detail_page` 既有做法）。
- 复核 `LegislationIntroPage` 文案与新入口是否重复，避免两个入口语义打架。

## 边界

- 不改链端、不改 `LegislationVotePage` 本体业务逻辑（除非接线暴露真实 bug）。
- 不动 `legislation_vote_query_service.dart` 的解码逻辑（已被 `test/legislation/legislation_codec_test.dart` 覆盖）。
- 不扩到「发起立法」——发起仍在节点端，本端只查看 + 投票。

## 验收

- `flutter analyze lib` 零问题、`flutter test --concurrency=1` 基线不回归。
- 实机验证：立法提案列表可见 → 进表决页 → QR 冷签闭环走通。
- 假入口消除：`ProposalKind.legislation` 的用户承诺与实际可达路径一致。
