# 任务卡：SFID绑定签名回执与wuminapp启动ANR修复

- 任务编号：20260525-175008
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-05-25 17:50:08

## 任务需求

修复SFID绑定签名回执challenge_id不一致、简化绑定弹窗步骤文案、排查并修复wuminapp启动前等待smoldot导致的Android无响应问题。

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
- 2026-05-25：定位 `challenge not found or expired` 根因:SFID 公民绑定 sign_request 曾使用 `bind-{uuid}` 作为二维码 id,但后端 Store 保存的是原始 `uuid`。
- 2026-05-25：定位 wuminapp Android “公民没有响应”根因: `main()` 在 `runApp()` 前 await smoldot 初始化,冷启动阶段容易触发系统 ANR。
- 2026-05-25：已修复 SFID 公民绑定 sign_request id,更新绑定弹窗回执扫描标题,删除“第三步：扫描签名结果二维码”提示。
- 2026-05-25：已调整 wuminapp 启动顺序,首帧后后台初始化 smoldot,并更新 SFID 前端、wuminapp QR/RPC 文档。

## 完成信息

- 完成时间：2026-05-25 17:53:43
- 完成摘要：修复 SFID 公民绑定签名回执 challenge_id 不一致，简化绑定弹窗签名回执扫描文案，并将 wuminapp smoldot 初始化移到首帧后后台执行以避免启动 ANR。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
