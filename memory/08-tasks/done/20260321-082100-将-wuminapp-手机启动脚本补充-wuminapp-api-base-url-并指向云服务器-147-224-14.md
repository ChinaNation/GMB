# 任务卡：将 wuminapp 手机启动脚本补充 WUMINAPP_API_BASE_URL，并指向云服务器 147.224.14.117 的 sfid 接口，避免真机回落到 127.0.0.1:8787 导致 runtime 升级提案提交失败

- 任务编号：20260321-082100
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-03-21 08:21:00

## 任务需求

将 wuminapp 手机启动脚本补充 WUMINAPP_API_BASE_URL，并指向云服务器 147.224.14.117 的 sfid 接口，避免真机回落到 127.0.0.1:8787 导致 runtime 升级提案提交失败

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
- 已更新 `wuminapp/scripts/app-run.sh`，默认传入 `WUMINAPP_API_BASE_URL=http://147.224.14.117:8899`。
- 已更新 `wuminapp/scripts/app-clean-run.sh`，默认传入 `WUMINAPP_API_BASE_URL=http://147.224.14.117:8899`。
- 已更新 `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md`，明确 `WUMINAPP_API_BASE_URL` 指向 `sfid` 的 HTTP API，且真机调试不得使用 `127.0.0.1`。

## 完成信息

- 完成时间：2026-03-21 08:21:36
- 完成摘要：已将 wuminapp 手机启动脚本默认补充 WUMINAPP_API_BASE_URL=http://147.224.14.117:8899，并同步更新技术文档，避免真机回落到 127.0.0.1:8787。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
