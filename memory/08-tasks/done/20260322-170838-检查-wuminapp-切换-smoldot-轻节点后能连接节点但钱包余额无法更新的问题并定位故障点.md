# 任务卡：检查 wuminapp 切换 smoldot 轻节点后能连接节点但钱包余额无法更新的问题并定位故障点

- 任务编号：20260322-170838
- 状态：open
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-03-22 17:08:38

## 任务需求

检查 wuminapp 切换 smoldot 轻节点后能连接节点但钱包余额无法更新的问题并定位故障点

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
- 已排查钱包余额链路：`wallet_page.dart -> ChainRpc.fetchBalance() -> SmoldotClientManager.request()`
- 已确认首个故障点在 `wuminapp/lib/rpc/smoldot_client.dart`
- 具体问题 1：发起 `state_getStorage` 前没有等待轻节点同步完成，页面首次进入时容易过早查询
- 具体问题 2：smoldot JSON-RPC 返回 `error` 时被当成 `result == null` 吞掉，导致上层把真实故障误判成账户不存在或余额 0
- 次级风险：`wuminapp/assets/chainspec.json` 当前是 `CitizenChain (Dev)`，如果打包目标不是当前开发链，会带来接入错误链或错误 bootnode 的风险
- 已修复：为 smoldot RPC 增加同步门槛、错误抛出与更长的轻节点请求超时
- 已更新：`memory/05-modules/wuminapp/rpc/RPC_TECHNICAL.md`
