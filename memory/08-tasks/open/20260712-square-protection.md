# CitizenApp 广场安全与盈利保护

- 状态：completed
- 创建：2026-07-11
- 模块：`citizenapp`、`citizenweb`
- 链上：不修改 `citizenchain/runtime/`

## 目标

- 正式 API 统一使用现有 `www.crcfrcn.com/api/*`，staging 使用同域受 Access 保护的 `www.crcfrcn.com/api-staging/*`。
- 关闭 production、staging 的 `workers.dev` 与 Preview URL，删除客户端和文档中的旧地址。
- 使用 WAF、Turnstile、设备逐请求签名、防重放和分层限流阻断机器人与异常请求。
- Cloudflare Images、Stream 改为私有交付，Feed 不保存或返回长期公开媒体地址。
- 修复 Stream 上传预授权按会员最大时长占用容量的问题，限制活动上传并清理过期资产。
- 按会员收入、媒体存储和播放用量实施盈利保护；Chat 对所有钱包用户开放且不保存消息或附件。
- APNs 暂不配置，不阻塞广场、会员、Android FCM、TURN 和前台 Chat。

## 强制决策

- 彻底改造，不保留旧 API 地址、旧公开媒体字段、旧 SQL 链、兼容分支或过渡流程。
- `www.crcfrcn.com` 是唯一对外域名，不新增 CitizenApp API 子域名。
- 现有 `0003` 至 `0009` 合并进唯一 `0001_square_core.sql`，远端 D1 清空后按目标基线重建，不迁移旧数据。
- 官网同源调用 `/api`；原生 App 使用钱包 Session、P-256 设备签名和 nonce 防重放。
- Turnstile 只用于首次设备绑定、设备换钥和风险升级，不干扰正常浏览。
- 会员浏览动态和文章条数不限制；视频播放使用异常用量阈值保护计费资源。

## 实施范围

- `citizenapp/cloudflare/src/security/`：Turnstile、请求证明、防重放、限流和用量熔断。
- `citizenapp/cloudflare/src/uploads/`：按申报时长授权、活动上传限制和过期资产清理。
- `citizenapp/cloudflare/src/media/`：Images、Stream 私有签名交付和公开 URL 清理。
- `citizenapp/cloudflare/src/auth/`、`chat/`、`feeds/`、`shared/`：统一安全入口和接口限制。
- `citizenapp/cloudflare/migrations/`：唯一目标数据库基线。
- `citizenapp/lib/8964/`、`lib/chat/`：设备请求签名、风险验证和短期媒体地址刷新。
- `citizenweb/src/`：同源 `/api` 调用和风险验证。
- `memory/`：架构、安全、协议、命名、部署和验收记录。

## 验收

- `www.crcfrcn.com/api/*` 是唯一正式 API；旧 `workers.dev`、Preview URL 和未授权 staging 路径不可访问。
- 重放、过期签名、错误设备、异常频率、超限上传和非法来源请求均在服务端拒绝。
- Images、Stream 原始 ID 和过期签名地址不能直接播放或下载。
- 上传授权只预占申报视频时长；过期上传和孤立媒体自动硬删除。
- 四档会员发布权益、媒体用量和盈利熔断符合服务端真源。
- Chat 仍对所有钱包用户开放，Cloudflare 不保存消息、会话和附件。
- Worker、Flutter、官网测试以及 staging、production 真实 HTTP/页面验收通过。
- 文档、中文注释、旧 URL、旧字段、旧 SQL、旧配置和临时测试数据全部清理。

## 执行记录

- production/staging API 已统一到 `www.crcfrcn.com/api*`，两个 Worker 均关闭 `workers.dev` 与 Preview URL；staging 由 Cloudflare Access 仅允许维护账户访问。
- D1 staging/production 已清空旧表和迁移历史，并按唯一 `0001_square_core.sql` 目标基线重建；不迁移、不兼容旧数据。
- Turnstile、P-256 设备逐请求签名、nonce 防重放、请求体限制、IP/钱包分层限流和注销安全数据硬删除已接入 Worker 与 App。
- Cloudflare WAF `citizenapp-api-edge-limit` 已启用：production/staging API 60 次/10 秒/IP，超限阻断 10 秒，Stripe/Stream webhook 排除。
- Images/Stream 已改为短期私有签名交付；上传按申报视频时长授权，并强制月度媒体额度、活动上传数和收入成本熔断。
- Stripe 与 Stream webhook 已分别切换到 production `/api` 正式路径；Stream 轮换后的签名 Secret 已同步 staging/production，临时配置令牌已删除。
- 官网 `citizenweb` 已构建并发布到现有 Cloudflare Pages 项目，正式 `/membership` 返回 200；APNs 按已确认范围暂不配置。
- production 已补齐独立 FCM 与 TURN Secret：FCM OAuth 200、无效 Token 返回预期 400，TURN 凭证返回 201 和两组 ICE servers；本机服务账号 JSON 已硬删除。
- 最终活动版本：staging `38f8ee7c-9eec-4ee8-8eca-92fb4cf1302e`，production `4db85515-9f58-4063-8829-4a0cd4e49bac`。
- 最终自动验收：Worker typecheck、19 个测试文件 113 项、production dry-run；Flutter 定向 analyze 与 Chat/签名/Turnstile 44 项；CitizenWeb production build 全部通过。
- 最终线上验收：production health 200、未登录 401、伪造 Origin 403、精确 Origin 200、超大请求 413、未签名 webhook 400；staging Access 302；两个旧 Worker 地址 404；官网 `/membership` 200。
- staging/production D1 各保留唯一目标基线，帖子、会员、上传测试数据均为 0；旧 API 地址、独立迁移文件名、死额度字段和临时密钥文件全仓/本机扫描为零。
