# 任务卡：重构 wallet 目录为一层子目录结构：ui 页面迁移到 pages，ui/cards 组件迁移到 widgets，同步引用、测试和文档

- 任务编号：20260430-110243
- 状态：open
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-04-30 11:02:43

## 任务需求

重构 wallet 目录为一层子目录结构：ui 页面迁移到 pages，ui/cards 组件迁移到 widgets，同步引用、测试和文档

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- wuminapp/WUMINAPP_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/wuminapp.md

### 默认改动范围

- `wuminapp`

### 先沟通条件

- 修改 Isar 数据结构
- 修改认证流程
- 修改关键交互路径


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/wuminapp.md

# WuMinApp 模块执行清单

- App 只是交互入口，不承担信任根职责
- Isar 结构、认证流程、关键交互变化前必须先沟通
- 关键 Flutter 交互与本地存储逻辑必须补中文注释
- 文档与残留必须一起收口


## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/wuminapp.md

# WuMinApp 完成标准

- App 仍然只是交互入口
- 关键 Flutter 交互和 Isar 逻辑已补中文注释
- 文档已同步更新
- 关键交互或数据结构变化已先沟通
- 残留已清理


## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已将 `lib/wallet/ui/wallet_page.dart`、`transaction_history_page.dart` 迁移到 `lib/wallet/pages/`
- 已将 `lib/wallet/ui/cards/*` 迁移到 `lib/wallet/widgets/`
- 已同步测试目录，将 `test/wallet/ui/cards/*` 迁移到 `test/wallet/widgets/`，将 `test/wallet/ui/wallet_list_tile_test.dart` 迁移到 `test/wallet/pages/`
- 已更新 `wallet.dart` barrel export、源码 / 测试 import 与当前技术文档
- 已清理空的旧 `ui/`、`ui/cards/`、`test/wallet/core`、`test/wallet/ui` 目录
- 验证通过：`flutter analyze`、`flutter test test/wallet`、`flutter test`
