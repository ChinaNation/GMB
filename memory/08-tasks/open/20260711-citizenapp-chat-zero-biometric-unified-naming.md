# CitizenApp Chat 零生物识别与统一命名

- 状态：已完成并由 `20260711-chat-square-step1.md` 继续收口
- 模块：`citizenapp`

## 当前结论

- Chat 使用钱包会话和设备绑定完成授权，普通聊天操作不得弹出生物识别确认。
- 设备绑定字段、签名载荷和服务端验签全仓同名，不保留别名或旧格式解析。
- 设备绑定一次性 nonce 只保存哈希和有效期；消费后或过期后硬删除。
- 一次性 KeyPackage 消费即硬删除。
- Cloudflare 不保存消息、会话、联系人关系或附件；发送失败只保留在发送设备本地队列。
- 注销立即关闭连接、撤销短期凭证并硬删除设备登记、推送 Token、KeyPackage 和 nonce 摘要。

## 验收边界

- 不新增数据库迁移链，当前基线直接描述目标结构。
- 不保留旧 Chat 数据、旧接口、旧字段或兼容分支。
- 不修改 `citizenchain/runtime/`。

当前协议真源见 `memory/07-ai/unified-protocols.md`，当前架构真源见 `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`。
