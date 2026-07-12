# CitizenApp Cloudflare 广场与聊天边缘能力

- 状态：已由 `20260711-chat-square-step1.md` 彻底改造
- 模块：`citizenapp`

## 当前结论

- Cloudflare Worker 负责钱包会话、广场 API、会员权益强制校验、媒体授权、设备登记、一次性 KeyPackage 和通用推送唤醒。
- 广场图片使用 Cloudflare Images，视频使用 Cloudflare Stream；帖子正文、索引、会员状态和浏览计数保存在 D1。
- 私密聊天消息、会话、联系人关系和附件只保存在通信双方设备，不进入 D1、R2、KV 或 Durable Object Storage。
- 在线消息使用 Durable Object 的休眠 WebSocket 瞬时转发；附件使用 WebRTC DataChannel 设备间传输；接收设备暂不可达时由发送设备本地队列继续重试。
- APNs/FCM 推送仅携带固定唤醒类型和发送方钱包账户，不携带消息正文、密文、附件地址、会话摘要或通知预览。
- 用户注销时先关闭实时连接并硬删除设备登记、推送 Token、KeyPackage、防重放摘要和浏览计数，再清理广场及订阅数据。

## 数据边界

- D1 Chat 表只保留 `chat_devices`、`chat_keypackages` 和 `chat_device_binding_nonces`。
- R2 不保存任何聊天对象。
- Durable Object 不使用持久化存储保存聊天数据。
- 客户端 Isar 和应用私有文件目录是消息及附件的唯一存储位置。

## 禁止事项

- 禁止恢复云端消息队列、消息确认状态、聊天附件对象存储或旧接口。
- 禁止为旧表、旧字段、旧协议或旧数据增加迁移、兼容、双写或回退分支。
- 禁止把区块链节点、runtime 或链上存储用于私密聊天传输。
- 禁止在推送通知中加入可还原聊天内容的字段。

## 后续真源

- 架构：`memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`
- 协议：`memory/07-ai/unified-protocols.md`
- 当前执行任务：`memory/08-tasks/open/20260711-chat-square-step1.md`
