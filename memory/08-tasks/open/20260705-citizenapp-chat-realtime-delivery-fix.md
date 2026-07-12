# CitizenApp Chat 瞬时实时投递

- 状态：已由 `20260711-chat-square-step1.md` 彻底改造
- 模块：`citizenapp`

## 当前结论

- 在线接收设备通过账户级 Durable Object 休眠 WebSocket 立即接收 OpenMLS `ChatEnvelope`。
- Durable Object 只持有当前连接及序列化设备附件，不写 Storage，不保留通知索引或消息状态。
- 接收设备不可达时，Worker 返回 `queued` 并发送无内容推送；密文继续留在发送设备本地队列。
- 推送后台处理器会在系统允许的短时窗口内自动建立 WebSocket、发送 `peer_ready` 并重试本机队列；双方不需要同时打开聊天页。
- 若任一设备被系统完全停止或长期离线，消息等待持有本地队列的发送设备恢复，禁止上传云端代存。

## 禁止事项

- 禁止恢复远程补拉、轮询云端消息、通知索引或消息确认状态。
- 禁止把消息、会话或附件写入 D1、R2、KV 或 Durable Object Storage。
- 禁止恢复旧接口、旧字段、旧数据或兼容分支。

当前真源见 `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`。
