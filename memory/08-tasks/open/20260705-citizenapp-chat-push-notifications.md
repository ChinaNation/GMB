# CitizenApp 聊天后台系统通知（FCM/APNs，红点/声音/静默）

> 功能 B（暂缓）。先做前台实时修复卡 `20260705-citizenapp-chat-realtime-delivery-fix`，本卡待用户具备 Firebase/APNs 资源后再排。

## 任务需求

- App 未打开/后台/被杀时，收到新消息按用户系统通知权限展示：红点（角标）/ 声音 / 不提醒。
- 推送只带信号（"有新消息" + 会话 id），**绝不带明文或密文正文**，正文由 App 前台拉取 mailbox 后本地解密（沿用现有 E2E "notice only" 设计）。

## 建议模块

- 推送基础设施：Android FCM、iOS APNs。
- 设备 token 注册：`citizenapp/cloudflare/`（新增 push token 表 + 注册/注销接口）+ `citizenapp/lib/chat/`（客户端注册）。
- Worker 推送发送：`citizenapp/cloudflare/src/chat/`（新 envelope 时向收件方离线设备发推送）。
- 客户端通知：`citizenapp/lib/chat/` + `lib/security/`（权限）+ Android/iOS 原生通道。

## 影响范围

- 新增 D1 表 `chat_push_tokens`（owner_account, device_id, platform, token, updated_at）+ 注册/注销/失效清理接口。
- Worker envelope POST：WS 推送之外，额外向收件方**离线设备**发 FCM/APNs（在线设备已由 WS 覆盖，避免重复打扰）。
- 客户端引入 `firebase_messaging` + `flutter_local_notifications`；申请通知权限（已在首启权限说明页有通知权限申请入口，见 CITIZENAPP_TECHNICAL 2.3）。
- 点击通知深链到对应会话。

## 主要风险点

- 推送 payload 泄漏明文：铁律只带信号 + 会话 id。
- 在线/离线重复通知：在线走 WS、离线走推送，需按 DO 在线设备集合去重。
- 外部依赖前置：iOS 需 Apple 开发者证书 + APNs Auth Key；Android 需 Firebase 项目 + `google-services.json`。缺任一，对应平台 B 卡停摆。
- token 失效/轮换：注册 token 需幂等更新 + 失效清理。
- 与现有首启通知权限策略（`lib/security/app_permission_bootstrap.dart`）对齐，不重复弹窗。

## 是否需要先沟通

- 是。前置资源未定：用户是否已有 Firebase 项目（Android FCM）与 Apple 开发者证书 + APNs key（iOS）。未具备则本卡挂起。

## 预计修改目录

- `citizenapp/cloudflare/`：push token 表 + 注册接口 + envelope 发推送；配置/代码。
- `citizenapp/lib/chat/`：客户端 token 注册、通知展示、点击深链；代码。
- `citizenapp/android/`、`citizenapp/ios/`：FCM/APNs 原生接入与证书配置。
- `memory/05-modules/citizenapp/chat/`：架构文档补后台推送。

## 分步骤技术方案（待前置资源确认后细化）

1. 前置：确认/创建 Firebase 项目 + APNs key。
2. Worker：`chat_push_tokens` 表 + 注册/注销接口 + envelope 离线推送发送（去重在线设备）。
3. 客户端：集成 `firebase_messaging`/`flutter_local_notifications`，注册 token，按系统权限展示红点/声音/静默。
4. 深链：点击通知进入对应会话。
5. 验收：真机前台（走 WS）/ 后台（走推送）/ 杀进程（走推送）三态；权限拒绝时静默不崩。

## 当前执行状态

- [ ] 暂缓，待前置资源确认后启动。
