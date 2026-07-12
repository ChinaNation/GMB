# 修复 CitizenApp 冷启动 ANR（公民没有响应）：广场后台会话越界懒注册 Turnstile

任务需求：CitizenApp 冷启动复现「公民没有响应」ANR，定位并按推荐修复。
所属模块：citizenapp（8964 广场会话 / Turnstile 页）、citizenapp/cloudflare（turnstile 页 CSP）

## 现场诊断（实锤）

- ANR 判定（logcat `ActivityManager`）：`Input dispatching timed out ... Waited 5002ms for FocusEvent`。
- 主线程栈（`/data/anr` dropbox）：`"main" tid=1 state=R utm=1038`（≈10.4s 纯用户态 CPU）。
- ANR dump 里**无 `1.ui` 线程** → 平台线程与 UI 线程合并的构建，任何主 isolate Dart 重活或平台视图开销都直接堵输入派发。
- ANR 窗口内 logcat 主导活动 = `SquareTurnstilePage` 的 `webview_flutter` 平台视图 + Cloudflare Turnstile 反爬 JS（`crcfrcn.com/api/v1/security/turnstile`）。
- 非上次修的 smoldot 轻节点问题（那些是子线程，全 async）。

## 回归根因

`square_session_provider.dart` 文档契约白纸黑字：「后台会话流程**绝不读 seed、绝不弹窗、绝不懒注册**」。
但代码里 `ensureSession` 的 `onDeviceNotRegistered` 恰在做懒注册 + 弹 Turnstile + `signWithWallet`（读 seed）。
`git log -S` 确认该越界路径由**最近一条提交 `1978db4a5 发布会员体系`** 引入 —— 正是「之前才解决、这次又出现」。
广场是默认首个 tab，冷启即 `_loadFeed()/_refreshMembership()/_refreshIdentity...` 并发调 `ensureSession`，
未注册设备每条都尝试弹 Turnstile WebView，把合并主线程顶死 → ANR。

## 输出物（按推荐修复）

1. 根因修复 `citizenapp/lib/8964/profile/services/square_session_provider.dart`：
   删除 `ensureSession` 的 `onDeviceNotRegistered` 懒注册实参（恢复文档契约）；删无用 import
   `device_subkey_registrar.dart`。未注册设备 → 会话失败按公开只读处理。子钥注册仍只在
   **钱包创建时**经 `main.dart` 的 `WalletManager.subkeyRegistrar` 完成，`DeviceSubkeyRegistrar` 类不成死码。
2. 加固 `citizenapp/lib/8964/pages/square_turnstile_page.dart`：
   `loadRequest` 从 `initState` 同步路径挪到 `addPostFrameCallback`，让路由先出首帧再拉重 WebView，
   保护钱包创建时那条合法 Turnstile 路径不卡首帧。
3. CSP `citizenapp/cloudflare/src/security/turnstile.ts`：
   `script-src` 补 `https://static.cloudflareinsights.com`，消除 beacon 被拦的 console 报错。

## 主动放弃（原推荐 #2 会话持久化）

根因是懒注册回归而非会话不持久。修复 #1 后：未注册设备直接只读（不再弹窗）；已注册设备冷启只走
P-256 静默握手（异步网络，不堵主线程）。持久化 bearer token 到磁盘无 ANR 收益却增安全面 → 不做。

## 验收标准

- `flutter analyze` 无新增告警（尤其无未用 import）。
- 相关 square/profile/membership 测试仍绿。
- 真机重装冷启动：无「公民没有响应」，logcat 无 `ANR in org.citizenapp`；未注册设备广场公开只读、不弹 Turnstile。

## 验收结果（2026-07-12 真机 Pixel 8a 已通过）

- `flutter analyze` 改动两文件：No issues found。Worker `tsc --noEmit`：无错。
- `flutter test`（profile_edit / profile_header / membership）：24 passed。
- 真机 debug 包重装（签名相符，未 uninstall、未动用户钱包数据）冷启动：
  - `mCurrentFocus=org.citizenapp/.MainActivity` —— 干净拿到老版本超时 5s 的 `FocusEvent(hasFocus=true)`。
  - dropbox 无新 `data_app_anr`（最新仍是修复前 2026-07-11 23:05）。
  - 截图：直接进广场并完整渲染（feed 空态），全程无 Turnstile、无白屏、无 ANR。
- 状态：DONE。

## 无遗留

开发期零用户、无老钱包/存量（见 feedback_in_development_zero_users）。删掉懒注册即完成，
不存在"老钱包未注册→只读"场景，不需要任何补注册入口。契约按 feedback_square_session_never_lazy_register
执行：注册只在钱包创建时静默完成。
