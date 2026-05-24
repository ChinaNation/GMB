# 任务卡：保存 wuminapp P2P IM 技术选型方案：平台原生近场模块采用 wuminapp/android/im 与 wuminapp/ios/im 两个浅层目录，im 目录只承载 IM 近场通信功能；同步记录远程通信全节点、Android BLE+Wi-Fi Direct、iOS Multipeer Connectivity、统一消息层边界。

- 任务编号：20260523-121946
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-05-23 12:19:46

## 任务需求

保存 wuminapp P2P IM 技术选型方案：平台原生近场模块采用 wuminapp/android/im 与 wuminapp/ios/im 两个浅层目录，im 目录只承载 IM 近场通信功能；同步记录远程通信全节点、Android BLE+Wi-Fi Direct、iOS Multipeer Connectivity、统一消息层边界。

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
- 已新增 `memory/05-modules/wuminapp/im/IM_TECHNICAL.md`，保存 wuminapp P2P IM 技术选型方案。
- 已在 `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md` 中记录 IM 预定技术路线和浅层目录约定。
- 已明确 Android 原生近场模块规划为 `wuminapp/android/im/`，iOS 原生近场模块规划为 `wuminapp/ios/im/`，两个目录只承载 IM 近场通信功能。
- 已明确当前只是保存方案，没有创建 `wuminapp/android/im/`、`wuminapp/ios/im/` 或业务代码。
- 已执行 `git diff --check`，并检查当前未创建实际 Android/iOS IM 代码目录。

## 完成信息

- 完成时间：2026-05-23 12:22:25
- 完成摘要：已保存 wuminapp P2P IM 技术选型方案：Android/iOS 原生近场模块采用 android/im 与 ios/im 浅层目录；远程通信全节点、Android BLE+Wi-Fi Direct、iOS Multipeer Connectivity、统一消息层和禁止局域网/公共全节点中继边界已写入文档。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
