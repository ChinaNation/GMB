# CitizenApp 聊天通用唤醒推送

- 状态：代码已并入 `20260711-chat-square-step1.md`，外部凭证待配置
- 模块：`citizenapp`

## 最终方案

- Android 使用 FCM，iOS 使用 APNs。
- 推送载荷固定为 `kind=chat_wake` 与 `sender_account`，禁止增加消息正文、密文、附件、会话摘要和预览。
- 推送只唤醒接收设备；消息仍由发送设备经瞬时连接投递，不从 Cloudflare 拉取聊天内容。
- App 获取平台 Token 后，通过钱包会话和设备绑定签名登记到 Worker；Token 刷新时覆盖当前设备记录。
- 用户注销时立即硬删除推送 Token 和设备登记。

## 外部配置

- Firebase 项目需提供 Android/iOS 应用参数和 FCM 服务账号凭证。
- Apple Developer 需提供 APNs Key、Key ID、Team ID 和 Topic。
- 密钥只写入 Cloudflare Worker Secrets，不进入仓库。

当前实现与验收真源见 `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`。
