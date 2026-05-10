# wuminapp 电子护照模块迁移

## 需求

- 在 `wuminapp/lib/my/myid/` 下承接电子护照功能。
- 用户可见名称统一改为“电子护照”。
- 在“我的”页面的“通讯录”和“设置”之间新增“电子护照”入口。
- 完成后更新文档、完善注释、清理旧残留。

## 边界

- 只处理 wuminapp 端目录归属、页面入口、命名和文档。
- 后端接口路径暂时保持现有 `/api/v1/app/vote-account/*`。
- 不改 SFID 后端、链上 runtime、node 前后端。

## 验收

- `lib/my/user` 不再承载电子护照设置流程。
- `lib/wallet/capabilities` 不再承载电子护照注册/状态接口。
- `flutter analyze` 通过。
- 旧入口文案和旧服务引用清理完成。

## 完成记录

- 已新增 `wuminapp/lib/my/myid/` 电子护照模块。
- 已在“我的”页面新增“电子护照”入口。
- 已删除 `lib/my/user` 和 `lib/wallet/capabilities` 中的旧归属实现。
- 已更新 wuminapp user/wallet 技术文档。
- 已使用 Dart SDK 执行 `dart analyze lib`，结果通过。
