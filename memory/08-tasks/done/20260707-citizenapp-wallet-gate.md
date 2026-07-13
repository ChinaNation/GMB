# 公民App 无热钱包强制创建门禁（启动页已存在，不在本卡范围）

> **状态：已完成（2026-07-07）**。实现见下方「完工记录」。

## 任务需求

1. **账户判断门禁**：启动画面之后检查本机钱包——公民 App 用户的唯一账户是钱包账户，必须至少有 1 个**热钱包**（发消息/发动态/发交易都依赖它签名）；没有热钱包的用户不进广场，先进创建钱包页。
2. **强制创建钱包页**：该页只能创建 12 或 24 个助记词的**热钱包**（字面执行：不提供导入助记词、不提供冷钱包入口），创建成功后才进入主界面（广场 Tab）。

## 现状（已实证）

- **启动页已存在**（原生层）：Android `drawable/launch_background.xml` = 白底 + 居中 `@mipmap/launch_image`；iOS `LaunchScreen.storyboard`。本卡不改启动页。
- 入口链：`main()` → `CitizenApp(home: _AppLockGate)`（citizenapp/lib/main.dart:87）→ PIN 锁/设备锁验证（都未设置则直通）→ `AppPermissionGate(AppShell)`（main.dart:225）→ `_currentIndex = 0`，第一个 Tab 即 `SquareTabPage` 广场（main.dart:355）。**除锁屏验证外，打开即进广场，无钱包门禁。**
- `WalletManager.getDefaultWallet()`（lib/wallet/core/wallet_manager.dart:213）已定义"默认身份 = 最靠前热钱包，无热钱包返回 null，由上层给出创建热钱包引导"——门禁语义已预留，入口层从未接。
- `CreateWalletPage`（lib/wallet/pages/wallet_page.dart:1230）已存在：12/24 SegmentedButton 选词数 → `createWallet(wordCount)` → 防截屏助记词备份弹窗；当前创建成功后 `pop(true)` 回钱包页。
- 应用锁现状：PIN 锁 = 自管 6 位 PIN（SHA-256+salt 存 SecureStorage，5 次错锁 24h、3 次锁定清库，lib/security/app_lock_service.dart）；设备锁 = `local_auth` 生物识别/系统凭证。签名路径每次走 `_authenticateIfSupported`（人脸 > 指纹 > 设备密码）。**保持现状，不改为 passkey**（passkey 是面向服务端登录的原语，不适用于本地解锁；结论见聊天记录 2026-07-07）。

## 建议模块

- Mobile Agent / `citizenapp`：
  - `lib/main.dart`：在 `_AppLockGate` 验证通过之后、进 `AppShell` 之前插入热钱包门禁（复用 `getDefaultWallet()` 判空）。
  - `lib/wallet/`：`CreateWalletPage` 抽出/增加 onboarding 强制模式（无返回、创建成功进 AppShell 而非 pop）。

## 影响范围

- 启动顺序：原生启动页 → PIN/设备锁验证（如已设置）→ 热钱包判断 → 无热钱包 → 强制创建页；有热钱包 → AppShell（广场 Tab 顺序不变）。
- 门禁口径 = **热钱包数量 ≥ 1**：仅有冷钱包的用户同样被拦（与"唯一账户必须是热钱包"口径一致，`getDefaultWallet` 本就只认热钱包）。
- 门禁只在冷启动判定；App 使用中删光热钱包不做即时踢回（字面不扩）。
- smoldot 轻节点后台初始化时序不变（main.dart:65 postFrameCallback），门禁不等链。

## 主要风险点

- `createWallet` 前置要求设备已开启系统锁屏（`_ensureDeviceSecure`，wallet_manager.dart:714），未开锁屏的新设备会在强制创建页失败——页面需给出页面级明确指引（当前只有 SnackBar 一闪而过）。
- 强制创建页不提供导入入口：重装 App 的老用户在门禁处只能新建，导入仍在进入主界面后的钱包页——字面需求如此，已记录为默认口径。**（此口径已于 2026-07-12 反转：首启门禁页补齐"导入已有钱包"入口,复用 ImportWalletPage,二元 fail-closed;见 20260712-citizenapp-onboarding-import-wallet。）**
- 助记词备份弹窗防截屏逻辑（ScreenshotGuard）在 onboarding 模式下必须原样保留。

## 是否需要先沟通

- 否。边界来自用户原话，两处边缘默认口径（仅冷钱包用户同样被拦；强制页只创建不导入）已记录，如需调整在执行前提出即可。

## 输出物

- `lib/main.dart` 门禁接入
- `CreateWalletPage` onboarding 强制模式
- 回写本卡完工状态

## 完工记录（2026-07-07）

- 新增 `lib/wallet/wallet_gate.dart`：三态门禁（checking/needsWallet/ready）+ 第四种错误态（本地库读取失败既不误判「无钱包」也不放行，停错误态给「重试」）；`defaultWalletLoader` 可注入供测试。
- 新增 `lib/wallet/pages/create_wallet_onboarding_page.dart`：强制创建页（PopScope 禁返回、12/24 卡片选择默认 12、三条安全说明、未开锁屏页面级警示卡 + 重新检测 + resume 自动复检、创建按钮 fail-closed 禁用）；`deviceSecureProbe` 可注入。
- 新增 `lib/wallet/pages/create_wallet_flow.dart`：共享创建流程 `runCreateWalletFlow`（创建→基线余额→防截屏备份弹窗）+ 三个钱包错误文案助手转公开（原 wallet_page.dart 私有函数上移）。
- `lib/main.dart`：`AppPermissionGate(child: WalletGate(child: AppShell()))` 单点接入。
- `lib/wallet/pages/wallet_page.dart`：`CreateWalletPage._create` 改调共享流程，删除本地私有错误助手（改用公开版）。
- 测试：`test/wallet/wallet_gate_test.dart` 7 用例（门禁三态+错误重试+创建页锁屏禁用/复检/词数切换）；`wallet_manager_test.dart` 补 `getDefaultWallet` 忽略冷钱包用例；`widget_test.dart` 冒烟播种热钱包 + 新增「无钱包冷启动进创建页」用例（不建 AppShell 不触发 smoldot，纯 Dart CI 照跑）。
- 验证：`flutter analyze` 干净（仅 1 条既有无关 info）；全量 `flutter test --concurrency=1` 416 过 5 skip（均为既有环境 skip）。
- 备注：UI 稿中「去系统设置」按钮未实现——仓库无 app_settings/android_intent 类依赖，不为此新增依赖，警示卡以文字指引 + 重新检测代替。
