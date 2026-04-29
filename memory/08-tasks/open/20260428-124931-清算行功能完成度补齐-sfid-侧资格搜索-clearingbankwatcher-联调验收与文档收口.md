# 任务卡：清算行功能完成度补齐：SFID 侧资格搜索、ClearingBankWatcher、联调验收与文档收口

- 任务编号：20260428-124931
- 状态：open
- 所属模块：sfid
- 当前负责人：Codex
- 创建时间：2026-04-28 12:49:31

## 任务需求

清算行功能完成度补齐：SFID 侧资格搜索、ClearingBankWatcher、联调验收与文档收口

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

## 本卡目标

- 把 SFID 侧完成度从约 85% 补到可验收状态。
- 范围包括 `sfid/backend` 清算行搜索、`ClearingBankWatcher`、SFID 前端清算行资格展示、接口文档和联调记录。

## 待补齐清单

- 关闭或归档 `20260428-115654-sfid-修复-clearing-bank-watcher-启动崩溃` 任务卡，避免状态残留。
- 明确 `/api/v1/app/clearing-banks/search` 在 watcher 未首次 scan 成功时的降级策略是否符合产品预期。
- 补联调验收：链上 `ClearingBankNodes` 注册 / 更新 / 注销后，SFID 搜索结果能按预期变化。
- 为 wuminapp 所需字段补齐契约说明：清算行 sfid_id、主账户、费用账户、节点端点来源。
- 检查 `clearing-bank-eligibility.md` 是否需要追加 Step 2 watcher 章节和 Step 3 移动端消费说明。

## 验收标准

- `cargo test --manifest-path sfid/backend/Cargo.toml clearing_bank` 通过。
- `cargo check --manifest-path sfid/backend/Cargo.toml` 通过。
- watcher 启动不再出现 Tokio runtime 前 `spawn` 崩溃。
- 至少有一份联调记录证明 SFID 搜索和链上 `ClearingBankNodes` 一致。

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
