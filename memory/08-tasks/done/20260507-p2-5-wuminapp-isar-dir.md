# 任务卡:P2-5 wuminapp Isar 目录小写化

## 任务需求

执行重新创世前总审计 P2-5：将 `wuminapp/lib/Isar/` 统一改为 `wuminapp/lib/isar/`，消除 Flutter/Dart 目录大小写命名不一致。

本任务只做目录命名和 import 路径同步，不修改 Isar collection、schema、数据库名称、迁移逻辑或业务行为。

## 预计修改目录

| 目录 | 用途、边界和修改类型 |
|---|---|
| `wuminapp/lib/Isar/` | 旧大写目录；通过大小写中转改名为 `wuminapp/lib/isar/`，涉及残留清理。 |
| `wuminapp/lib/isar/` | 新小写目录；继续承载 `wallet_isar.dart` 与 `wallet_isar.g.dart`，涉及目录命名统一。 |
| `wuminapp/lib/duoqian/` | 同步多签模块 import 路径；只改引用，不改业务逻辑。 |
| `wuminapp/lib/wallet/` | 同步钱包模块 import 路径；只改引用，不改业务逻辑。 |
| `wuminapp/lib/security/` | 同步锁屏服务 import 路径；只改引用，不改业务逻辑。 |
| `wuminapp/lib/onchain/` | 同步链上支付页面 import 路径；只改引用，不改业务逻辑。 |
| `wuminapp/lib/trade/` | 同步本地交易存储 import 路径；只改引用，不改业务逻辑。 |
| `wuminapp/lib/proposal/` | 同步提案缓存相关 import 路径；只改引用，不改业务逻辑。 |
| `wuminapp/lib/rpc/` | 同步链交易监听 import 路径；只改引用，不改业务逻辑。 |
| `wuminapp/test/duoqian/` | 同步多签相关测试 import 并回归；涉及测试。 |
| `wuminapp/test/wallet/` | 同步钱包相关测试 import 并回归；涉及测试。 |
| `memory/07-ai/` | 登记 `wuminapp/lib/isar/` 并禁止恢复 `wuminapp/lib/Isar/`；涉及统一命名文档。 |
| `memory/08-tasks/` | 更新本任务卡和重新创世审计记录；涉及任务文档。 |

## 执行清单

- [x] 使用 `isar_tmp` 中转完成大小写目录改名。
- [x] 将所有 `Isar/wallet_isar.dart` import 改为 `isar/wallet_isar.dart`。
- [x] 更新统一命名文件，登记新目录并禁止恢复旧目录。
- [x] 更新重新创世审计记录。
- [x] 扫描确认没有旧 `Isar/` import 残留。
- [x] 运行 wuminapp 目标 analyze 与测试。

## 验收标准

- `wuminapp/lib/Isar/` 不再存在。
- `wuminapp/lib/isar/wallet_isar.dart` 和 `wallet_isar.g.dart` 存在。
- `rg 'Isar/|package:wuminapp_mobile/Isar|\\.\\./Isar' wuminapp/lib wuminapp/test` 无命中。
- 目标 Flutter analyze 和测试通过。

## 执行结果

2026-05-07：

- 已将 `wuminapp/lib/Isar/` 通过 `isar_tmp` 中转改名为 `wuminapp/lib/isar/`。
- 已同步 `wuminapp/lib/` 与 `wuminapp/test/` 内所有 `wallet_isar.dart` import。
- 已在 `memory/07-ai/unified-naming.md` 登记 `wuminapp/lib/isar/`，并禁止恢复旧 `wuminapp/lib/Isar/`。
- 未修改 Isar collection、schema、数据库名称或迁移逻辑。

## 验证记录

- `rg 'package:wuminapp_mobile/Isar|\\.\\./Isar|\\.\\./\\.\\./Isar|/Isar/|lib/Isar|Isar/wallet_isar' wuminapp/lib wuminapp/test`：无命中。
- `flutter analyze` 目标文件：通过，`No issues found!`。
- `flutter test test/wallet/wallet_manager_test.dart test/wallet/wallet_manager_reorder_test.dart test/wallet/attestation_service_test.dart test/duoqian/duoqian_discovery_service_test.dart test/duoqian/personal_pending_create_lookup_test.dart test/duoqian/personal_proposal_history_service_test.dart`：通过。
