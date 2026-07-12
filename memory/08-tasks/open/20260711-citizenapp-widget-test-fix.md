# CitizenApp widget 测试真失败修复

- 状态：done（改动留工作区待 review）
- 创建：2026-07-11
- 模块：`citizenapp`（仅测试）
- 链上：不修改 `citizenchain/`
- 来源：命名审计任务阶段 I 收尾发现；方案 B（Isar 隔离 + dart_test.yaml）跑完后暴露的 pre-existing widget 真失败

## 前提规则（已核验）

- 广场必须使用钱包会话浏览（[20260711-chat-square-step1.md](../open/20260711-chat-square-step1.md) 第 15 行）。
- 入口 `WalletGate` 包裹整个 `AppShell`：无热钱包强制建钱包，有热钱包才放行；建钱包即注册设备子钥。
- 故正常流程下广场/profile **一定有默认热钱包 + 可用会话**；测试里 `FakeSessionProvider(null)` 是不存在的状态。
- 会员精确匹配禁降档已上线（[[project_membership_visitor_two_tier_exact_match]]）→ `identityBadgeStyle` 勾色恒白，不再随会员档着色。

## 第一批：widget stale/harness（只改测试，产品零改）

- `test/ui/identity_badge_test.dart`：3 条跨档「勾随会员档」旧断言重写为「底色随身份、勾恒白」。→ +4 绿
- `test/8964/profile/profile_header_test.dart`：`FakeSessionProvider(null)` → `fakeSession()`（2 处）。→ +14 绿
- `test/8964/profile/profile_posts_tab_test.dart`：`_page` 注入有效会话。→ +6 绿
- `test/8964/profile/user_profile_page_test.dart`：`_wrap` 注入有效会话。→ +3 绿
- 定性：identity_badge=stale（exact-match 上线源码简化测试没跟上）；profile 三文件=harness 欠配会话（撞 `_tabBody` 无会话永久 spinner→`pumpAndSettle` 超时）。非回归——「广场须钱包会话」由 chat-square-step1 新规背书，WalletGate 保证真实流程必有会话。

## 第二批：personal_proposal（原判「污染」错，实为独立真 bug）

- `personal_proposal_history_service_test.dart` 单跑/全量都失败：`fetchAll`→`_refreshLocalVotingEntities` 对本机 voting 提案查链，测试环境 `fetchProposalStatus` 返回 null → 把 `action=='create'` 的 voting 提案当「幽灵」**删除**（每测试首条 create 丢失）。
- 根因：把「链不可达」误判成「链上确认不存在」。**离线会静默删本机待投票创建提案 = 数据丢失真 bug**。
- **治本改产品**（用户拍板）：新增 `ChainRpc.isFinalizedChainReachable()`（读 `System.Number` 正向探针）；`_refreshLocalVotingEntities` 与孪生 `hasUnchainedVotingCreateProposal` 都只在链确认可达时才删幽灵，离线保留本机记录。
- **测试 hermetic 化**：该测试原本非 hermetic（`ensureSynced`→`ensureStarted` 真启 smoldot，本机连着真链就红、连不上才绿=flaky）。注入 `_OfflineChainRpc` 切断真链，聚焦 Isar 持久层。→ +5 绿、耗时 7s→1s。

## 验收结果

- 逐文件全绿：identity_badge / profile_header / profile_posts_tab / user_profile_page / square_home_page / personal_proposal_history。
- `flutter analyze` 全量仅 1 条既有 info（`onchain_payment_service.dart:43`，非本任务文件），零新增。
- 全量 `flutter test`：**+518 ~1 -0，All tests passed!**（1 skip = smoldot native probe 守卫，预期）。
- 改动留工作区不提交，供 review。

## 改动文件

- 测试：identity_badge_test / profile_header_test / profile_posts_tab_test / user_profile_page_test / personal_proposal_history_service_test。
- 产品：`lib/rpc/chain_rpc.dart`（+探针）、`lib/transaction/personal-manage/personal_proposal_history_service.dart`（删幽灵门禁×2）。
