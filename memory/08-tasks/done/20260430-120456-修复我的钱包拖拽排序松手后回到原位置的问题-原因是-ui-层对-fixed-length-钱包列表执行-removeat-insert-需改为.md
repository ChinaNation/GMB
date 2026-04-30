# 任务卡：修复我的钱包拖拽排序松手后回到原位置的问题，原因是 UI 层对 fixed-length 钱包列表执行 removeAt/insert，需改为可变列表并验证排序落盘

- 任务编号：20260430-120456
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-04-30 12:04:56

## 任务需求

修复我的钱包拖拽排序松手后回到原位置的问题，原因是 UI 层对 fixed-length 钱包列表执行 removeAt/insert，需改为可变列表并验证排序落盘

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
- 已定位原因：`WalletManager.getWallets()` 返回 `toList(growable: false)` 形成 fixed-length 列表，`MyWalletPage._onReorder()` 直接对 `_wallets` 执行 `removeAt/insert` 会触发 `Unsupported operation`，导致拖拽预览能移动但松手后状态没有真正保存。
- 已修复 `wuminapp/lib/wallet/pages/wallet_page.dart`：新增 `reorderWalletProfiles()`，拖拽时先复制成可变列表，再按 Flutter `onReorder` 规则修正下标并写入 UI 状态。
- 已保持原落盘路径：`_onReorder()` 继续调用 `WalletManager.reorderWallets()` 写入 Isar `sortOrder`，没有修改钱包数据结构、底部导航文案或其他功能入口。
- 已补充 `wuminapp/test/wallet/pages/wallet_list_tile_test.dart`：覆盖 fixed-length 钱包列表排序，确认不改写原列表。
- 已更新 `memory/05-modules/wuminapp/wallet/WALLET_TECHNICAL.md`：记录钱包卡片拖拽排序流程、fixed-length 列表约束、`sortOrder` 字段和测试覆盖。

## 验证记录

- `dart format wuminapp/lib/wallet/pages/wallet_page.dart wuminapp/test/wallet/pages/wallet_list_tile_test.dart`
- `flutter test test/wallet/pages/wallet_list_tile_test.dart test/wallet/wallet_manager_reorder_test.dart`
- `flutter analyze`
- `flutter test test/wallet`
- `flutter test`

## 残留检查

- 本次只触碰钱包拖拽排序修复、对应测试、钱包技术文档和任务卡。
- 未新增 `trade`、`test` 功能目录，未修改底部第 3 个按钮“交易”的文案。
