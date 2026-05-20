# 任务卡：wuminapp CI 链状态轮询测试修复

- 任务编号：20260519-wuminapp-ci-chain-progress-test
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-05-19

## 任务需求

修复 GitHub `WuMinApp CI` 中 `test/widget_test.dart: app bootstraps` 因 `pumpAndSettle timed out` 失败的问题。用户确认采用“测试模式下面直接禁止链状态轮询”的方案。

## 执行范围

- `wuminapp/lib/ui/widgets/chain_progress_banner.dart`：测试环境下不启动链状态读取和定时轮询。
- `wuminapp/test/widget_test.dart`：保留现有 App 启动测试，验证组件修复后不会卡住。
- `memory/08-tasks/`：同步任务记录。

## 约束

- 不改变真机、debug、release 环境的链状态展示和轮询行为。
- 不绕开 App 启动测试本身，只禁用测试环境中无业务价值的链状态后台轮询。
- 保留中文注释，解释测试环境禁用边界。

## 实施记录

- `ChainProgressBanner` 在 Flutter widget test 环境保留组件结构和静态提示条，但跳过轻节点状态读取、进度回调和轮询定时器。
- `VoteView` 在 Flutter widget test 环境跳过隐藏广场页的提案链上首屏刷新和新区块订阅，避免 `IndexedStack` 隐藏页阻塞 App 启动测试。
- 保持真机、debug、release 环境的轻节点状态读取、提案刷新和链订阅行为不变。

## 验证记录

- `flutter test test/ui/transaction_tab_page_test.dart`
- `flutter test test/widget_test.dart`
- `flutter test --concurrency=1`
- `dart analyze lib test`
