# 任务卡：wuminapp-duoqian-create-close-only-from-account

- 任务编号：20260430-100427
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-04-30 10:04:27

## 任务需求

收口 wuminapp 多签创建和关闭入口：创建/关闭多签只能从个人多签或机构多签账户体系进入，删除 governance 提案类型页中的多签创建/关闭入口；个人多签和机构多签关闭页面按类型拆分，不再共用一个关闭页面；更新 import、文档、残留并验证。

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
- 已删除 `governance/proposal_types_page.dart` 中注册多签机构专属的“创建多签/关闭多签”发起入口。
- 已将关闭页面按类型拆分：
  - `wuminapp/lib/duoqian/institution/institution_duoqian_close_page.dart`
  - `wuminapp/lib/duoqian/personal/personal_duoqian_close_page.dart`
- 已更新 `duoqian_account_info_page.dart`，机构多签详情进入机构关闭页，个人多签详情进入个人关闭页。
- 已将 `DuoqianManageService` 的创建/关闭提交入口拆成机构/个人语义方法：`submitProposeCreateInstitution`、`submitProposeCreatePersonal`、`submitProposeCloseInstitution`、`submitProposeClosePersonal`。
- 已将机构创建页的签名展示文案从泛化“创建多签”收口为“创建机构多签”。
- 已更新治理技术文档与仍 open 的相关任务卡路径，清理旧 `duoqian_close_proposal_page.dart` 等路径残留。
- 已执行残留搜索：`proposal_types_page.dart` 中不再存在 `OrgType.duoqian` 多签创建/关闭入口；旧关闭页类名和旧路径无命中。
- 已执行 `dart format`、`flutter analyze`、`flutter test`，均通过。

## 完成信息

- 完成时间：2026-04-30 10:11:05
- 完成摘要：多签创建/关闭入口已收口到个人/机构多签账户体系，治理提案类型页入口已删除，机构/个人关闭页已拆分，文档、残留搜索和 Flutter 验证已完成
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
