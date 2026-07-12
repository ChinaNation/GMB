# CitizenApp Chat 设备侧存储与广场会员访问第 1 步

- 状态：in_progress
- 创建：2026-07-11
- 模块：`citizenapp`
- 链上：不修改 `citizenchain/runtime/`

## 目标

- Chat 对所有钱包用户开放，会员不参与 Chat 权限。
- Cloudflare 只保存设备公钥、一次性 KeyPackage、推送 Token 和绑定防重放数据。
- 消息、会话、联系人关系和附件只保存在用户设备；Cloudflare 不持久化任何消息或附件。
- 接收设备不可达时只发送无内容唤醒通知，发送设备使用本地队列重试，不要求用户同时打开 App。
- 用户注销时立即关闭连接、撤销凭证并硬删除 Cloudflare 活动数据和本机数据。
- 广场必须使用钱包会话浏览；无订阅账户每日限量浏览且禁止发布；有效会员按四档权益发布并不限产品浏览量。

## 强制决策

- 彻底改造，不迁移旧 Chat 数据，不保留旧接口、旧字段、旧表、兼容分支或过渡流程。
- 直接重建 Chat 基线结构，清空 staging/production 旧 Chat D1 数据和 R2 `chat/` 对象。
- 删除旧聊天内容表、云端待投递/确认状态、R2 Chat 附件上传下载和对应客户端流程。
- 第 1 步不实现防机器人、盗链、异常用量、WAF、Turnstile、设备完整性证明和风控熔断；这些统一归第 2 步。
- 第 1 步只部署 staging；第 2 步完成安全防护后再统一部署 production。

## 实施范围

- `citizenapp/cloudflare/src/chat/`：瞬时 WebSocket/信令转发、无内容 APNs/FCM 唤醒和注销清理。
- `citizenapp/cloudflare/src/feeds/`：钱包会话和每日浏览额度。
- `citizenapp/cloudflare/src/posts/`、`uploads/`：发布全链路有效会员校验。
- `citizenapp/cloudflare/src/account/`：注销先删 Chat，再处理订阅和广场数据；全引用硬删除。
- `citizenapp/cloudflare/migrations/`：只维护全新部署基线；本任务不新增数据迁移文件。
- `citizenapp/lib/chat/`：本地出站队列、后台唤醒、瞬时密文和设备间附件传输。
- `citizenapp/lib/8964/`：浏览剩余额度和无会员发布禁用。
- `citizenapp/android/`、`citizenapp/ios/`：平台推送与后台能力。
- `memory/`：更新架构、安全边界、任务状态和残留说明。

## 数据基线

- `chat_devices`：账户、设备、公钥、推送服务、推送 Token、有效期。
- `chat_keypackages`：一次性 MLS KeyPackage，消费即删。
- `chat_device_binding_nonces`：nonce 哈希和有效期，过期即删。
- `square_browse_days`：账户、UTC 日期、已返回内容数、更新时间。

## 验收

- D1 不存在任何聊天内容表，R2 不存在 `chat/` 对象。
- Worker、D1、KV 和 Durable Object Storage 不保存消息、会话或附件。
- 两台真实设备完成前台投递、后台唤醒、离线后恢复重试和附件设备间传输。
- 无订阅账户达到每日额度后被服务端拒绝，且上传准备、上传完成和发布确认均不能绕过会员校验。
- 四档会员普通/竞选发布权限符合套餐真源。
- 注销后 Chat、会话、设备、浏览计数、媒体和账户引用查询均为零。
- 类型检查、单元测试、Flutter 测试、staging 真实 HTTP 和真实页面验收通过。
- 文档、中文注释和旧代码/旧文案/旧配置残留全部清理。

## 完成记录

- 已彻底删除独立 Chat 迁移文件、旧聊天内容表、云端消息状态、R2 Chat 附件接口、客户端云端附件字段和远程补拉流程；不保留迁移或兼容分支。
- staging / production D1 已清空旧 Chat 数据并按 `0001_square_core.sql` 目标基线重建；远端迁移登记已清理；两个 R2 bucket 的 `chat/` 前缀均确认无对象。
- Worker 已实现瞬时 WebSocket/信令转发、通用 APNs/FCM 唤醒、注销先删 Chat、无订阅每日 100 条浏览和发布全链路会员校验。
- App 已实现消息/附件设备本地存储、本机出站重试、推送后台短时自动收发、Token 刷新重登记和 WebRTC DataChannel 附件。
- 浏览扣量已使用 D1 原子条件更新；并发旧快照请求不能越过每日上限。
- staging Worker 当前版本 `04dce458-9050-4fbc-bb14-13abc49ce36c` 已部署并真实验收：health 200、未登录接口 401、无订阅额度 100、额度耗尽 429、发布准备 402；临时 KV/D1 验收数据已硬删除。
- Worker 类型检查通过，19 个测试文件 108 项通过；Flutter Chat 42 项和广场相关 7 项通过；Android debug APK 构建通过，iOS plist/entitlements 校验通过，官网 build 通过。全量 `flutter analyze` 仅剩本任务未触及的链上支付文件 1 条既有 info。
- 文档、安全规则、统一命名、协议、官网中英文白皮书、节点内置白皮书和历史任务卡已更新；旧 Chat 存储关键词与旧接口残留全仓搜索为零。
- Firebase 项目 `citizenapp-23542`、`org.citizenapp` Android/iOS 应用和专用 FCM 服务账号已创建；服务账号仅授予 FCM API Admin。三项 staging Worker Secret 已写入，OAuth 返回 200，FCM HTTP v1 返回预期无效 Token 错误，证明服务端鉴权链路有效。
- Firebase 客户端公开配置已作为现有代码默认值，不提交平台配置文件；不传构建参数的 Android debug APK 构建通过。本机下载的服务账号 JSON 与 `google-services.json` 已硬删除。
- 2026-07-12 目标态已删除聊天附件中继：客户端只使用公共 STUN 发现候选地址，附件仅设备直连；中继接口、密钥、数据表和客户端分支不再保留。
- production 已为同一最小权限 FCM 服务账号创建独立密钥并写入三项 Secret；Google OAuth 返回 200，FCM HTTP v1 对故意无效 Token 返回预期 `INVALID_ARGUMENT`。下载 JSON 与临时私钥均已硬删除。

## 外部阻塞

- Apple Developer 尚未创建 APNs Key，无法写入 APNs Worker Secret。
- APNs 凭证缺失使 iOS 后台推送与 Android/iOS 双真机完整闭环尚无法验收；production Android FCM、前台 Chat 与广场 API 不受 APNs 延期影响。
