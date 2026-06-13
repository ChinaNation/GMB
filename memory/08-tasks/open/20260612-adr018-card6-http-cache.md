# 任务卡:卡⑥ HTTP/SFID 后端 Isar+TTL 缓存 + 公权机构目录 catalog

属 ADR-018 §四E + §九(2026-06-13)。

## E 类:HTTP/SFID 后端缓存(37 处无缓存)
- [ ] `wallet/capabilities/api_client.dart`:health 5min / admin catalog 1d / 机构注册证 7d
- [ ] `my/myid/myid_api.dart:50` 电子护照状态:Isar 缓存 + 15min 刷新
- [ ] `rpc/sfid_public.dart:51` 清算行搜索:已部分缓存,补 TTL
- [ ] `update/app_update_service.dart:92` GitHub release:低频,加短缓存

## §九:公权机构目录 catalog(下一步建界面时落地)
公权机构界面"还没做,下一步再做";本项随界面开发落地,先约定契约。
- [ ] 公权机构目录 = SFID 后端 catalog 接口(分页 + 搜索)+ Isar/TTL(catalog 1d)。**轻节点不扫链**。
- [ ] 点进详情:用已知 sfid_number **本地派生**主/费地址(`governance/shared/account_derivation.dart`)+ 精确整键读余额/状态;自定义账户清单由 catalog 带出,**不碰 `SfidRegisteredAddress` 长前缀**。
- [ ] catalog 接口若后端未就绪:先与 SFID 后端约定 OpenAPI 契约(分页游标 + 省/类型筛选 + 机构基础字段),客户端先行接 mock。

## 验收
- [ ] flutter analyze 0 + flutter test 全过
- [ ] E 类:各接口命中缓存,logcat 验证 HTTP 调用数下降
- [ ] 旧代码/文档/注释清理无残留

## 边界
- 公权机构详情的余额/状态仍走链上精确整键(经卡⑤ 缓存);catalog 只负责"目录发现",不替代链上实时态。
