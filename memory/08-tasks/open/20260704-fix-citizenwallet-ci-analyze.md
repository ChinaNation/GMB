# 修复公民钱包 CI 分析失败

## 任务需求

修复 `CitizenWallet CI` 中 `flutter analyze --no-fatal-infos` 报出的 3 个问题：

- 公民钱包测试仍引用已删除的旧 `Balances` 直签 pallet 常量。
- `ReorderableListView.builder` 曾切到新版拖拽回调参数,但本机安装脚本使用的 Flutter 3.41.0 不支持该参数。
- 测试中常量字符串使用 `final`，触发 `prefer_const_declarations`。

## 影响范围

- `citizenwallet/lib/ui/home_page.dart`
- `citizenwallet/test/signer/pallet_registry_test.dart`
- `citizenwallet/test/signer/payload_decoder_test.dart`
- `citizenwallet/scripts/citizenwallet-run.sh`
- `.github/workflows/citizenwallet-ci.yml`
- `memory/05-modules/citizenwallet/CITIZENWALLET_PQC_TECHNICAL.md`
- 本任务卡

## 实施步骤

- [x] 读取执行前必读文档和公民钱包相关文档。
- [x] 修复 `balancesPallet` 旧常量引用。
- [x] 修复拖拽排序回调的 Flutter 版本兼容问题。
- [x] 修复测试常量声明。
- [x] 清理 CI / 本地脚本中的旧 `Balances` 同步口径。
- [x] 运行公民钱包本地分析/测试验证。
- [x] 清理残留并记录结果。

## 验收标准

- `citizenwallet` 在 CI 使用的 Flutter stable 3.44.x 上 `flutter analyze --no-fatal-infos` 通过。
- 相关 signer 测试通过。
- 公民钱包代码、脚本和 CI workflow 不再残留旧 `Balances` 直签 pallet 字段。

## 验证记录

- `dart format lib/ui/home_page.dart test/signer/pallet_registry_test.dart test/signer/payload_decoder_test.dart`：通过。
- `bash -n citizenwallet/scripts/citizenwallet-run.sh`：通过。
- `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/citizenwallet-ci.yml")'`：通过。
- `flutter test test/signer`：通过。
- `git diff --check`：通过。
- 旧引用检查：公民钱包、CI workflow 和公民钱包文档中不再残留旧 `Balances` 直签同步口径。
- 2026-07-04 本机安装失败复修:`citizenwallet/lib/ui/home_page.dart` 回退为 Flutter 3.41.0 支持的 `onReorder`,并按 Flutter 原生语义恢复向下拖动时 `newIndex -= 1`。
- `flutter analyze --no-fatal-infos`：通过。
- `flutter build apk --debug`：通过,已生成 `build/app/outputs/flutter-apk/app-debug.apk`。
- `flutter install --debug -d RZCY814477Y`：通过,已安装到 `SM A156U`。
