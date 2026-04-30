# 任务卡：按机构多签和个人多签重构 wuminapp 多签入口：在 lib/duoqian 下建立 shared/institution/personal 分层，删除交易页多签交易聚合入口，把我的页多签账户入口迁到交易页，并在交易页提供机构多签和个人多签两个入口；保留各多签账户详情中的发起转账提案能力。

- 任务编号：20260430-092239
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-04-30 09:22:39

## 任务需求

按机构多签和个人多签重构 wuminapp 多签入口：在 lib/duoqian 下建立 shared/institution/personal 分层，删除交易页多签交易聚合入口，把我的页多签账户入口迁到交易页，并在交易页提供机构多签和个人多签两个入口；保留各多签账户详情中的发起转账提案能力。

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
- 已在 `wuminapp/lib/duoqian/` 下建立多签新目录：
  - `shared/`：单类型列表、详情共享入口、入口类型枚举
  - `institution/`：机构多签列表与创建入口
  - `personal/`：个人多签列表与创建入口
- 已删除交易页旧的“多签交易”聚合入口页面，交易页改为“机构多签”“个人多签”两个入口。
- 已移除“我的”页面里的“多签账户”入口。
- 已保留并补齐各多签账户详情页中的“发起转账提案”入口，入口进入既有 `transfer_proposal_page.dart`。
- 已删除旧的 `wuminapp/lib/governance/duoqian_institution_list_page.dart` 与 `wuminapp/lib/trade/duoqian/duoqian_trade_page.dart`。
- 已同步更新 `memory/05-modules/wuminapp/trade/onchain/ONCHAIN_TECHNICAL.md` 与 `memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`。
- 已执行残留搜索，代码与测试中无 `DuoqianTradePage`、`duoqian_trade_page`、`DuoqianInstitutionListPage`、`duoqian_institution_list_page` 旧入口引用。
- 已执行 `dart format`、`flutter analyze`、`flutter test`，均通过。

## 完成信息

- 完成时间：2026-04-30 09:32:36
- 完成摘要：wuminapp 多签入口已按机构/个人分流到 lib/duoqian，旧聚合入口与我的页入口已移除，账户详情保留发起转账提案，文档与验证已完成
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
