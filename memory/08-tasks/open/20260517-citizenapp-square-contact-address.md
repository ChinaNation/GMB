# 任务卡：修复公民投票页改广场并删除背景引言；修复链上交易从通讯录选择联系人时把 SS58 地址误按 AccountId hex 解析导致联系人账户地址无效的问题

- 任务编号：20260517-142413
- 状态：open
- 所属模块：citizenapp
- 当前负责人：Codex
- 创建时间：2026-05-17 14:24:13

## 任务需求

修复公民投票页改广场并删除背景引言；修复链上交易从通讯录选择联系人时把 SS58 地址误按 AccountId hex 解析导致联系人账户地址无效的问题

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- citizenapp/CITIZENAPP_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenapp.md

### 默认改动范围

- `citizenapp`

### 先沟通条件

- 修改 Isar 数据结构
- 修改认证流程
- 修改关键交互路径


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenapp.md

# CitizenApp 模块执行清单

- App 只是交互入口，不承担信任根职责
- Isar 结构、认证流程、关键交互变化前必须先沟通
- 关键 Flutter 交互与本地存储逻辑必须补中文注释
- 文档与残留必须一起收口


## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/citizenapp.md

# CitizenApp 完成标准

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
- 已将公民二级 Tab 文案从“投票”改为“广场”，并删除投票页背景引言水印文件与引用。
- 已将通讯录联系人地址统一为 SS58 `address`，链上交易页和联系人详情页不再做 AccountId hex 转换。
- 已同步 citizenapp 架构、user、QR、governance、onchain transaction 文档，并清理 open 任务卡中的错误地址边界记录。
- 验证：`dart analyze lib test` 通过；`flutter test test/user/user_service_test.dart` 通过；`git diff --check` 通过。
