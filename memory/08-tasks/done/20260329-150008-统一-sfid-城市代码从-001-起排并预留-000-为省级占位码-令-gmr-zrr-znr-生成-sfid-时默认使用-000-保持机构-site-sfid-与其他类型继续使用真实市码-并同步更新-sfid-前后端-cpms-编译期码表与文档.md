# 任务卡：统一 sfid 城市代码从 001 起排并预留 000 为省级占位码，令 GMR/ZRR/ZNR 生成 sfid 时默认使用 000，保持机构 site_sfid 与其他类型继续使用真实市码，并同步更新 sfid 前后端、cpms 编译期码表与文档

- 任务编号：20260329-150008
- 状态：done
- 所属模块：sfid/backend
- 当前负责人：Codex
- 创建时间：2026-03-29 15:00:08

## 任务需求

统一 sfid 城市代码从 001 起排并预留 000 为省级占位码，令 GMR/ZRR/ZNR 生成 sfid 时默认使用 000，保持机构 site_sfid 与其他类型继续使用真实市码，并同步更新 sfid 前后端、cpms 编译期码表与文档

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- sfid/README.md
- sfid/SFID_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/sfid-backend.md

### 默认改动范围

- `sfid/backend`
- 必要时联动 `sfid/deploy`

### 先沟通条件

- 修改 permit 模型
- 修改账户绑定规则
- 修改数据库结构


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/sfid.md

# SFID 模块执行清单

- 不保存原始实名
- permit、绑定、数据库结构变化前必须先沟通
- 关键接口和数据模型必须补中文注释
- 文档与残留必须一起收口


## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/sfid.md

# SFID 完成标准

- 仍然满足 SFID 不保存原始实名
- 关键接口、数据模型与边界判断已补中文注释
- 文档已同步更新
- permit、绑定、数据库结构变化已先沟通
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

## 完成信息

- 完成时间：2026-03-29 15:06:38
- 完成摘要：已统一 43 个省份 city_codes 为 000 省级占位 + 001 起连续真实市码；GMR/ZRR/ZNR 生成 sfid 时固定使用省码+000；SFID 前端工具已按 A3 默认锁定省级占位项；CPMS 编译期市码校验已排除 000；相关文档已更新，cargo test 与前端构建通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
