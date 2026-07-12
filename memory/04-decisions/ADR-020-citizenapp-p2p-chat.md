# ADR-020：CitizenApp 设备侧私密聊天

- 状态：Accepted
- 决议日期：2026-07-11
- 关联任务卡：`memory/08-tasks/open/20260711-chat-square-step1.md`
- 技术真源：`memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`

## 决议

CitizenApp Chat 使用钱包地址作为聊天账户、OpenMLS 作为端到端加密、Protobuf `GMB_CHAT_V1` 作为消息外层。

消息、会话、联系人关系、发送队列和附件只保存在用户设备。Cloudflare 只承担：

- 钱包 session 和 Chat 设备登记。
- 一次性 OpenMLS KeyPackage。
- Durable Object WebSocket 当前请求内密文转发。
- WebRTC SDP/ICE 当前请求内信令转发。
- APNs/FCM 无内容唤醒。
- 公共 STUN 候选发现；不配置云端附件中继或中继凭证。

Cloudflare 不保存消息或附件，R2 不参与 Chat。接收设备不可达时，密文留在发送设备本机队列；通用推送在系统允许的后台窗口自动建立连接并发送 `peer_ready`，发送设备在线则立即重试，离线则由反向唤醒启动其本机队列。

## 密钥边界

- 钱包主私钥只用于一次性绑定硬件 P-256 设备子钥。
- Chat 登录和设备登记使用 P-256 子钥静默签名。
- OpenMLS 设备密钥独立生成并只存在设备安全存储。
- CitizenWallet、CID 资料和 CitizenChain runtime 不进入聊天收发路径。

## 附件

附件经 WebRTC DTLS DataChannel 在设备间直连传输。直连失败时附件继续保留在发送设备等待重试；Worker、D1、KV、Durable Object Storage 和 R2 均不得接收、保存或中继附件字节与引用。

## 删除

用户删除会话时只删除当前设备本地记录。用户注销时，当前设备本地 Chat 数据清空，Worker 同步关闭连接并硬删除所有设备登记、KeyPackage 和防重放行。

## 禁止项

- 禁止区块链节点承担聊天投递或设备配对。
- 禁止服务器保存私聊或群聊内容。
- 禁止用钱包私钥直接加密消息。
- 禁止把 Chat 路由缓存做成第二套通讯录。
- 禁止为旧聊天流程保留接口、字段、表、对象或客户端分支。
