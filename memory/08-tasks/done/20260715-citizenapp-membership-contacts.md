# CitizenApp 会员入口与加密通讯录同步

任务需求：
- “身份｜会员”进入时不再校验钱包是否存在于链上或余额是否达到存在性存款。
- 通讯录按默认热钱包隔离，并以端到端密文保存到 Cloudflare，换设备导入同一钱包后可恢复。
- 重新实现通讯录页面与单条联系人 UI，联系人点击进入现有统一用户主页。
- 广场、聊天、通讯录继续共用同一个用户主页，不保留联系人详情副本。
- 本任务是两步改造的第 1 步；电子护照严格留到第 2 步，本任务禁止修改。

安全边界：
- Cloudflare Session 只证明钱包控制权和已登记设备子钥，不再以链上账户存在或余额作为登录门禁。
- 会员购买、换档、发布、投票、竞选等真实业务动作继续按各自规则校验链上身份和资格。
- Cloudflare 只保存通讯录密文、随机数、认证码和不透明联系人 ID，不得保存联系人账户或私人联系人名称明文。
- 通讯录加密密钥由热钱包 seed 域隔离派生，仅保存在设备安全存储中，不上传 Cloudflare。
- 用户注销必须硬删除 Cloudflare 通讯录密文；本机删除钱包必须清理本地通讯录密钥和缓存。

预计修改目录：
- `citizenapp/cloudflare/src/auth/`：移除 Session 的链上账户/余额门禁，保留设备子钥认证；涉及 Worker 代码、中文注释和残留清理。
- `citizenapp/cloudflare/src/chain/`：删除仅供旧 Session 门禁使用的链钱包检查；只做代码残留清理。
- `citizenapp/cloudflare/src/contacts/`：新增端到端加密通讯录 CRUD；Worker 不解密联系人内容。
- `citizenapp/cloudflare/src/account/`：账户注销时硬删除通讯录密文；涉及代码和注释。
- `citizenapp/cloudflare/src/limits/`、`citizenapp/cloudflare/src/routes.ts`：登记通讯录路由、请求限制和分派；涉及接口代码。
- `citizenapp/cloudflare/migrations/`：新增 `square_contacts` 密文表；涉及 SQL。
- `citizenapp/cloudflare/test/`：更新 Session 测试并新增通讯录权限、密文和注销测试。
- `citizenapp/lib/my/membership/`：链身份暂不可读时降级展示会员页面；不重做现有会员卡 UI。
- `citizenapp/lib/my/user/`：实现通讯录 Isar 缓存、加密同步、搜索、联系人卡片和统一主页跳转；清理旧联系人详情与 SharedPreferences 双轨。
- `citizenapp/lib/8964/`：复用现有 Session、用户资料、头像和用户主页；不建立第二套公开资料真源。
- `citizenapp/lib/wallet/core/`：在钱包边界内派生和保存通讯录专用密钥；seed 不出钱包核心。
- `citizenapp/lib/qr/`、`citizenapp/lib/chat/`、`citizenapp/lib/transaction/`：接入新通讯录和统一用户主页导航；不修改二维码协议、聊天密文或交易语义。
- `citizenapp/test/`：补充加密、同步、会员降级、通讯录 UI 和导航测试。
- `memory/`：更新统一协议、命名、安全边界和 CitizenApp 技术文档；清理旧本地通讯录及链上登录门禁说明。

验收标准：
- 没有链上账户、余额不足或链 RPC 不可用时，设备子钥认证成功仍可创建 Cloudflare Session。
- 会员页面在链身份暂不可读时仍展示 D1 会员和套餐，并明确提示身份暂未刷新；需要身份的动作保持禁用。
- D1 中不存在联系人账户、联系人名称或联系人关系明文。
- 同一钱包在另一设备导入后可以解密并恢复通讯录；不同钱包不能解密。
- 通讯录普通模式点击联系人进入现有 `UserProfilePage`，转账选择模式仍返回联系人账户。
- 旧 `_ContactDetailPage`、旧通讯录 SharedPreferences 单真源及旧链钱包门禁全部清理。
- 完成测试、真实本地 Worker/D1/App 验收、文档更新、中文注释和残留扫描。

当前进度：
- [x] 用户确认第 1 步完整技术方案和新增文件。
- [x] 实现会员 Session 门禁修复和降级展示。
- [x] 实现 Cloudflare 加密通讯录接口与注销清理。
- [x] 实现 CitizenApp 通讯录密钥、缓存、同步和 UI。
- [x] 完成自动化测试和真实运行态验收。
- [x] 更新文档、完善注释、清理残留并归档任务卡。

执行记录：
- 2026-07-15：任务拆为两步；本卡只执行会员入口和通讯录，电子护照保持已确认第 1 版设计且本步不修改。
- 2026-07-15：Cloudflare Session 删除链上账户/余额门禁；会员查询保留链身份读取和 `identity_error` 降级，订阅/续订继续 fail-closed，已有自动续费订阅仍可取消。
- 2026-07-15：新增 `square_contacts`、密文 CRUD、P-256 请求证明、账户注销硬删除和独立限流。D1 只保存 owner、HMAC `contact_id`、AES-GCM 密文、nonce、MAC 和更新时间。
- 2026-07-15：通讯录改为按默认热钱包隔离的 Isar 本地优先模型；密钥在 `WalletManager` 内以 HKDF-SHA256 派生并保存到系统安全存储；历史 SharedPreferences 数据一次性迁移后删除旧键。
- 2026-07-15：通讯录页面完成搜索、同步状态、公开资料组合卡、改名/删除；普通点击进入统一 `UserProfilePage`，交易选择模式仍返回 SS58 地址；聊天头像也复用统一用户主页入口。
- 2026-07-15：验证结果：Cloudflare TypeScript 检查通过，22 个测试文件/169 项通过；CitizenApp 全量 554 项通过、5 项既有跳过；`flutter analyze --no-fatal-infos` 通过，仅 2 条任务外 info；Android Debug APK 构建并安装 Pixel 8a，会员页、通讯录本地降级和联系人统一主页跳转真实通过。
- 2026-07-15：本地 Wrangler + D1 真实 HTTP 验收通过：P-256 签名后的联系人 PUT/GET/DELETE 均为 200，读取记录不返回 owner，删除后为 0。staging/production 未获本任务部署授权，未触碰远端；真机通讯录因此按设计显示本地缓存与“同步失败”状态，待后续部署当前 Worker/D1 变更后启用跨设备云同步。
- 2026-07-15：电子护照目录和 `citizenchain/runtime/` 均无本任务 diff；旧链登录门禁、联系人详情副本和旧字段代码残留扫描通过。

- 状态：done

## 完成信息

- 完成时间：2026-07-15 10:34:37
- 完成摘要：完成会员入口解耦链账户门禁、端到端加密通讯录跨设备同步、本地优先 UI 与统一用户主页接入，并通过全量测试、本地 Worker/D1 和 Pixel 8a 真机验收；远端未部署。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
