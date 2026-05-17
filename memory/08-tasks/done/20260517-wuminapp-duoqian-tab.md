# wuminapp 多签 Tab 入口调整

## 任务需求

- 底部 `消息` Tab 改名为 `多签`。
- 点击 `多签` Tab 后直接显示现有统一多签账户列表。
- 多签页面顶部标题显示 `多签`，右上角保留 `+`。
- 点击 `+` 继续使用现有 `新增个人多签 / 新增机构多签` 入口。
- 删除原消息页左上角通讯录按钮、中间搜索框和消息占位内容。
- 从交易 Tab 中移除 `多签交易` 入口。

## 修改边界

- `wuminapp/lib/main.dart`：底部导航和 Tab 页面映射。
- `wuminapp/lib/governance/duoqian_account_list_page.dart`：统一多签列表标题。
- `wuminapp/lib/transaction/transaction_tab_page.dart`：移除交易页多签入口。
- `wuminapp/test/`：更新入口相关测试。
- `memory/01-architecture/wuminapp/` 与 `memory/05-modules/wuminapp/`：同步当前入口口径。

## 验收标准

- 底部导航显示 `公民 / 多签 / 交易 / 我的`。
- 多签 Tab 直接展示统一多签账户列表，标题为 `多签`。
- 多签列表右上角 `+` 仍能进入新增个人/机构多签菜单。
- 交易页不再显示 `多签交易` 入口。
- 原消息页通讯录按钮、搜索框和占位文案无残留。
- 文档、测试、残留扫描完成。

## 执行结果

- [x] `wuminapp/lib/main.dart`：底部第 2 个 Tab 从 `消息` 改为 `多签`，删除原消息页入口、通讯录按钮、搜索框和占位内容。
- [x] `wuminapp/lib/main.dart`：多签列表改为第一次点击 `多签` Tab 时再构建，避免应用启动时提前触发多签账户发现。
- [x] `wuminapp/lib/governance/duoqian_account_list_page.dart`：统一多签账户列表标题改为 `多签`，保留右上角 `+` 菜单。
- [x] `wuminapp/lib/transaction/transaction_tab_page.dart`：交易页移除 `多签交易` 入口，只保留扫码支付业务入口。
- [x] `wuminapp/test/`：更新交易页和应用启动测试，覆盖 `多签` Tab 名称和交易页入口删除。
- [x] `memory/01-architecture/wuminapp/`、`memory/05-modules/wuminapp/`：同步多签入口归属和交易页边界。

## 验证记录

- [x] `dart format wuminapp/lib/main.dart wuminapp/lib/governance/duoqian_account_list_page.dart wuminapp/lib/transaction/transaction_tab_page.dart wuminapp/test/ui/transaction_tab_page_test.dart wuminapp/test/widget_test.dart`
- [x] `flutter analyze lib test`
- [x] `flutter test test/widget_test.dart`
- [x] `flutter test --concurrency=1`
- [x] `git diff --check`
- [x] 残留扫描：旧 `MessagePage`、消息 Tab 标签、交易页 `多签交易` 入口、旧多签列表路径无有效残留。
