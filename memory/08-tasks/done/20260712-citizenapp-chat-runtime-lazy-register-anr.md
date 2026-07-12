# 任务卡：chat_runtime 后台会话删懒注册（恢复不读 seed/不弹 Turnstile 契约，防 ANR）

状态：已完成（2026-07-12）

任务需求：
`citizenapp/lib/chat/chat_runtime.dart` 的 `_ensureServiceReady()` 调用
`_squareApiClient.ensureSession` 时仍传 `onDeviceNotRegistered` 回调做懒注册
（`DeviceSubkeyRegistrar.register` → 弹 `SquareTurnstilePage` + `_walletManager.signWithWallet` 读 seed）。
与正上方注释「后台会话握手绝不读 seed / 不弹窗 / 不懒注册」直接矛盾，属提交
`1978db4a5 发布会员体系` 引入的同款回归（square 侧已修，见
`memory/08-tasks/done/20260712-citizenapp-square-turnstile-startup-anr.md`）。合并主线程弹 Turnstile
WebView 平台视图会顶死主线程 → ANR。

所属模块：citizenapp / chat

改动内容：
- 删除 `_ensureServiceReady` 里 `ensureSession` 的 `onDeviceNotRegistered` 实参（含内联
  `DeviceSubkeyRegistrar.register` + `signWithWallet` 读 seed 分支）。
- 删除因此变孤儿的 `import '../8964/services/device_subkey_registrar.dart';`。
- 补充中文注释：未注册设备直接会话失败按不可用降级，绝不在合并主线程弹 Turnstile。
- 保留 `import '../wallet/core/device_subkey.dart';`（`DeviceSubkey`/`bytesToHex` 仍用）与
  `_deviceSubkey`（signRawHex）/`_walletManager`（getDefaultWallet）字段（其它路径仍引用）。

契约后置行为：
`ensureSession` 在 `onDeviceNotRegistered == null` 时对 `device_not_registered` 直接 rethrow
（`square_api_client.dart:351-353`），沿 `await` 链上抛，聊天初始化按不可用降级；子钥注册只保留在钱包创建
入口（`main.dart` 的 `WalletManager.subkeyRegistrar`）。`DeviceSubkeyRegistrar` 现仅 `main.dart` 引用。

验收结果：
- `dart analyze lib/chat/chat_runtime.dart`：No issues found（unused_import 已消）。
- `dart format --set-exit-if-changed`：0 changed。
- `flutter test test/chat/ --concurrency=1`：+38 ~4（4 skip 为无 native 库 smoldot 用例，符合既有约定）全过。
- 无测试断言旧懒注册路径（`chat_tab_test.dart` 用 `_FakeRuntime extends ChatRuntime`，不触及）。
- 与 square 侧修法一致，无残留。
