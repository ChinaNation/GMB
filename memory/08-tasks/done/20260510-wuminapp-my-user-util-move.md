# wuminapp my 目录归拢 user 和 util

## 需求

- 将原用户目录移动到 `wuminapp/lib/my/user/`。
- 将原工具目录移动到 `wuminapp/lib/my/util/`。
- 前端 UI 显示、页面结构、按钮文案、业务行为不变。
- 完成后更新文档、完善注释、清理残留。

## 边界

- 只处理 wuminapp 目录归属和 import 路径。
- 不改电子护照业务逻辑。
- 不改交易、钱包、治理页面 UI 和业务逻辑。
- 不处理当前工作区已有的无关 CI workflow 和交易页测试改动。

## 完成记录

- 已完成 `user` 与 `util` 目录迁移。
- 已同步更新 Dart import。
- 已更新 wuminapp user 技术文档。
- 已清理旧顶层用户/工具目录路径残留。
- 已执行 `dart analyze lib test/user/user_service_test.dart test/utils/amount_format_thousands_test.dart`。
- 已执行 `flutter test test/user/user_service_test.dart test/utils/amount_format_thousands_test.dart`。
