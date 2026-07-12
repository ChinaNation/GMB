# GMB 安全规则

## 1. 基础红线

- CID 不保存原始实名数据
- CitizenChain 不保存真实身份
- permit 必须短期有效
- CitizenApp 私密聊天消息、会话、联系人关系和附件只能保存在通信双方设备，禁止写入 Cloudflare D1、R2、KV 或 Durable Object Storage
- Chat 推送只能发送固定唤醒类型和发送方钱包账户，禁止携带明文、密文、附件地址、会话摘要或通知预览
- 用户注销时必须先关闭实时连接、撤销短期凭证，再立即硬删除 Cloudflare 中的设备公钥、推送 Token、一次性 KeyPackage、防重放摘要和其他账户引用；不得软删除、延期删除或保留恢复副本

## 2. AI 开发安全规则

- 不允许 AI 在未确认需求、未检查仓库代码/文档/任务卡或真实运行输出时，自行猜测关键业务逻辑、现有实现、运行状态、扣费、分账、权限、存储和部署结果
- 对不了解或未复查的代码实现，AI 必须先全仓搜索、读取相关代码和文档，必要时执行只读检查，再回复用户；无法确认时只能明确说明“尚未检查/无法确认”
- 修改信任边界前必须先沟通
- 修改数据库模型前必须先确认影响范围
- 修改链上资格和权限规则前必须先确认
- 修改二维码结构和 permit 结构前必须同步更新文档与测试
- 修改 `citizenchain/runtime` 中会影响 `citizenapp` 在线端或 `citizenwallet` 公民钱包二维码签名/验签兼容性的内容前，必须先同步更新双端代码、文档与测试；未完成双端更新前，不允许继续修改 runtime
- 上述兼容性触发项至少包括：`spec_version` / `transaction_version`、`construct_runtime!` 中的 pallet index、相关 call index、签名载荷编码依赖、冷钱包 `pallet_registry` / `payload_decoder` 所依赖的运行时索引与版本
- 不允许删除、迁出或重命名 AI 编程系统核心基础设施

## 3. 代码与文档规则

- 更新代码后必须同步更新文档
- 更新代码后必须清理残留
- 关键逻辑必须补充中文注释
- 不允许保留临时调试逻辑进入正式分支
- `memory/` 相关核心目录与入口文件只能原位修改，不能在 PR 中移除

## 4. 发布前规则

- 测试通过后才能发布
- 文档未更新视为未完成
- 主要 review 问题未处理不能发布
- 目标结构和真实运行态验收未完成时不能发布

## 5. CitizenApp API 与媒体安全

- CitizenApp production API 唯一入口为 `https://www.crcfrcn.com/api/*`；staging 唯一入口为受 Cloudflare Access 保护的 `https://www.crcfrcn.com/api-staging/*`，禁止恢复 `workers.dev`、Preview URL 或独立 API 子域名。
- 官网浏览器请求只允许精确 Origin `https://www.crcfrcn.com`；原生 App 无 Origin 时必须使用钱包 Session、P-256 设备逐请求签名、时间窗和一次性 nonce，不能仅凭 User-Agent、IP 或客户端声明授权。
- 首次设备绑定、设备换钥和风险升级必须通过 Turnstile；Stripe 与 Stream webhook 分别使用提供商签名，不叠加设备签名。
- Worker 必须在解析 JSON 前限制请求体，并按 IP 哈希、钱包账户、接口类别分层限流；staging 还必须由 Cloudflare Access 限定维护账户。
- Cloudflare WAF 规则 `citizenapp-api-edge-limit` 对 production/staging API 按 IP 执行 60 次/10 秒的边缘阻断，阻断持续 10 秒；Stripe 与 Stream 签名 webhook 必须排除，避免提供商回调被普通客户端限流误伤。
- Cloudflare Images 必须启用签名交付，Cloudflare Stream 必须启用 signed URL；D1、R2 manifest 和 Feed 禁止保存长期公开媒体 URL。
- 媒体上传必须在服务端同时校验单帖权益、月度图片/视频额度、活动上传数和全局媒体成本熔断；Chat 不进入媒体用量预算，也不得把消息或附件保存到 Cloudflare。
- `citizenapp/cloudflare/src/limits/catalog.ts` 是 Cloudflare 资源硬上限唯一真源；环境变量只能收紧，不能放宽。所有外部路由必须在 D1 前完成路由白名单和 `Content-Length` 检查，并在读取阶段继续按实际字节截断。
- 头像、背景、manifest 和广场图片必须经 Worker 校验实际字节、MIME、图片文件头、尺寸与 sha256 后才能写 R2/Images；禁止向客户端签发 R2 PUT 或 Images 直传地址。视频必须统一使用绑定精确 `Upload-Length` 和最长时长的 Stream TUS，并在 webhook 按实际时长、分辨率复核。
- Chat 附件只允许 WebRTC 设备直连；仅使用 STUN 发现候选，禁止配置、签发或保存附件中继凭证。直连失败时附件继续保留发送设备本机，不得回退 Cloudflare 中继或存储。
