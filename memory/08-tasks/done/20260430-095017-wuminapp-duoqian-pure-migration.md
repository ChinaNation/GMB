# 任务卡：wuminapp-duoqian-pure-migration

- 任务编号：20260430-095017
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-04-30 09:50:18

## 任务需求

只迁移 wuminapp 纯多签功能到 lib/duoqian：迁移多签账户管理、创建、关闭、详情、二维码和管理提案详情；不迁移 QR 协议、Isar schema、钱包流水、治理聚合、机构通用、内部投票通用与转账提案通用文件；更新 import、文档并清理残留。

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
- 已只迁移纯多签功能到 `wuminapp/lib/duoqian/`：
  - `shared/`：多签账户列表、账户详情、管理模型、管理服务、关闭提案、二维码、管理提案详情
  - `institution/`：机构多签列表与机构创建表单
  - `personal/`：个人多签列表与个人创建表单
- 已保留未迁移的通用文件：QR 协议、Isar schema、钱包流水、治理聚合、机构通用、内部投票通用与转账提案通用文件仍在原目录。
- 已更新治理聚合页、机构详情页、提案缓存、提案类型页、转账提案服务的 import。
- 已确认 `wuminapp/lib/governance` 下不再存在纯多签源文件。
- 已同步更新 `memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md` 和 QR 协议任务卡里的多签二维码路径。
- 已执行残留搜索，代码与文档中无旧 `governance/duoqian_*` 纯多签路径引用。
- 已执行 `dart format`、`flutter analyze`、`flutter test`，均通过。

## 完成信息

- 完成时间：2026-04-30 09:56:55
- 完成摘要：纯多签功能已迁移到 lib/duoqian，通用 QR/Isar/钱包/治理聚合/转账提案能力保持原目录，import、文档、残留搜索和 Flutter 验证已完成
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
