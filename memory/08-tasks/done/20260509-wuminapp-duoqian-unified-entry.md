# 2026-05-09 wuminapp 多签交易单入口恢复

## 任务需求

交易 Tab 上恢复**单入口** "多签交易"，进入后看到**统一的账户列表**(个人 + 机构 混合)，右上角 "+" 弹出 2 个选项(增加个人多签 / 增加机构多签)。`lib/governance/personal-manage` 与 `organization-manage` 后端代码完全保留分离。

由 commit `bcc8a0b7 多签模块重构`(2026-05-09 15:49)误把单入口拆成两入口,本次回滚 UI 表现。

## 影响范围

- `wuminapp/lib/governance/duoqian_account_list_page.dart`(新建,统一壳子)
- `wuminapp/lib/transaction/transaction_tab_page.dart`(2 入口改 1 入口)
- `wuminapp/lib/governance/organization-manage/duoqian_account_list_page.dart`(删除)
- `wuminapp/lib/governance/personal-manage/personal_manage_account_list_page.dart`(删除)

## 风险点

- 两个原 list page 外部引用只有 `transaction/transaction_tab_page.dart` 一处(已 grep 确认),删除安全。
- 详情页(`*AccountInfoPage`)、create page、service、discovery、Isar entity、storage codec 全部保留,后端 0 改动。
- runtime / 链端 0 改动。
- 新统一页面要并行驱动 `DuoqianDiscoveryService` 与 `PersonalManageDiscoveryService`,2 套 Isar collection 并行查询。

## 执行状态

- [x] 新建 `governance/duoqian_account_list_page.dart`(统一壳子,2 套数据源合并展示 + "+"两选项 ActionSheet)
- [x] 改 `transaction/transaction_tab_page.dart`(2 个 `_TransactionEntryRow` → 1 个 `'多签交易'`,删 `personal-manage` import)
- [x] git rm `governance/organization-manage/duoqian_account_list_page.dart`
- [x] git rm `governance/personal-manage/personal_manage_account_list_page.dart`
- [x] `flutter analyze` 通过(No issues found,1.5s)
- [x] 残留扫描全零(`PersonalManageAccountListPage` / 旧文件路径 import 全零)
