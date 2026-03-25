# 任务卡：将 runtime 影响 wuminapp 在线端与 wumin 冷钱包二维码签名/验签兼容性的变更纳入 AI 编程系统强制规则，要求先同步更新双端后才允许修改 runtime

- 任务编号：20260325-125830
- 状态：open
- 所属模块：ai-system
- 当前负责人：Codex
- 创建时间：2026-03-25 12:58:30

## 任务需求

将 runtime 影响 wuminapp 在线端与 wumin 冷钱包二维码签名/验签兼容性的变更纳入 AI 编程系统强制规则，要求先同步更新双端后才允许修改 runtime

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/agent-playbooks.md
- memory/07-ai/module-checklists/citizenchain.md

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
- 已将规则写入 `memory/03-security/security-rules.md`
- 已将规则写入 `memory/07-ai/agent-rules.md`
- 已将规则写入 `memory/07-ai/chat-protocol.md`
- 已将规则写入 `memory/07-ai/agent-playbooks.md`
- 已将规则写入 `memory/07-ai/module-checklists/citizenchain.md`
- 已明确触发项：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` / `payload_decoder`
