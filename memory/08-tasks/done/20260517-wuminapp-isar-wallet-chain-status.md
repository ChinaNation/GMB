# wuminapp Isar 钱包写库与链状态提示修复

## 任务需求

- 修复 `IsarError: MdbxError (11): Try again` 与导入热钱包时可能出现的 Isar 嵌套事务问题。
- 修复钱包页把本地 Isar 写库错误误显示成“区块链连不上”的问题。
- 修正交易页顶部链状态提示语义，避免把轻节点状态快照误等同为所有业务链读已成功。
- 修复后构建 APK 并安装到当前连接的 `armeabi-v7a` 手机。

## 修改边界

- `wuminapp/lib/isar/wallet_isar.dart`：增加统一写事务串行入口，并确保钱包 settings 行处于目标状态。
- `wuminapp/lib/wallet/core/wallet_manager.dart`：消除 settings 获取中的嵌套事务，钱包写入改走统一写队列。
- `wuminapp/lib/wallet/pages/wallet_page.dart`：区分本地数据库错误与链读取错误。
- `wuminapp/lib/ui/widgets/chain_progress_banner.dart`：调整顶部提示文案为轻节点状态。
- `memory/01-architecture/wuminapp/` 与 `memory/05-modules/wuminapp/`：同步技术文档。

## 验收标准

- 创建/导入热钱包不再触发嵌套 `writeTxn`。
- 同时发生余额刷新、对账、多签扫描、钱包导入时，写库入口通过统一队列串行执行。
- 本地 Isar 错误不再被显示成区块链连接错误。
- APK 能安装到当前 `armeabi-v7a` 手机，启动后无 `IsarError/MdbxError`。

## 执行记录

- 已在 `WalletIsar` 增加业务写事务队列，所有业务写入统一通过 `WalletIsar.instance.writeTxn()` 串行执行。
- 已修复 `WalletManager._getSettings()` 在钱包创建/导入写事务中再次开启写事务的问题，事务内改用 `_getSettingsInTxn()`。
- 已把本地交易、pending 投票、多签本地状态、管理员目录缓存、证明态等业务写入改走统一写队列。
- 已把钱包页本地数据库错误提示与轻节点/链上读取错误提示分开。
- 已把 `ChainProgressBanner` 文案调整为“轻节点状态”，避免把轻节点 peer ready 误解为业务页面全部就绪。
- 已为 Isar/MDBX busy 瞬时错误增加短退避重试，并修复冷启动后台对账 Future 未完全吞住异常的问题。
- 已同步更新 wuminapp 架构文档、钱包模块文档与 RPC 模块文档。

## 验证记录

- `flutter analyze lib test`：通过，0 issue。
- `flutter build apk --debug --target-platform android-arm,android-arm64`：通过，产物 `build/app/outputs/flutter-apk/app-debug.apk`。
- `adb install -r build/app/outputs/flutter-apk/app-debug.apk`：已覆盖安装到 `moto g play - 2023`（`primaryCpuAbi=armeabi-v7a`）。
- 冷启动后 `adb logcat` 检查 `IsarError|MdbxError|active transaction|Try again|对账触发失败|Unhandled Exception|UnsatisfiedLinkError|libsmoldot.so`：无命中。
- 冷启动后日志确认轻节点恢复同步缓存、启动并收到 `grandpa-neighbor-packet-received` 与 `pong`。
