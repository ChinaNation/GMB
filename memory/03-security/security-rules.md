# GMB 安全规则

## 1. 基础红线

- CID 不保存原始实名数据
- CitizenChain 不保存普通公民的原始实名档案和非公开隐私信息
- 依法公开的机构法定代表人、机构岗位任职和竞选公开资料可以上链；不得因公开职务身份而附带公开护照号、出生日期、住址等私密档案
- 机构法定代表人任免生效后，必须将 `legal_representative: Option<{ family_name, given_name, cid_number, account }>` 作为一个原子公开机构信息上链；人的姓名不得保存拼接字段或另造前缀别名
- 创世没有真实法定代表人资料时不得填充假数据，不得把首位管理员默认为法定代表人；依赖法定代表人的业务在任命完成前必须拒绝执行
- permit 必须短期有效
- CitizenApp 私密聊天正文、会话摘要、MLS 状态和附件明文只能保存在通信参与方设备，
  禁止写入 Cloudflare D1、KV 或 Durable Object Storage。大媒体 R2 中转只允许保存端侧
  E2E 加密的短期密文，并按既定领取确认或 24 小时生命周期删除。
- CitizenApp 通讯录明文只能保存在用户设备；Cloudflare D1 只允许保存由 CID 当前链上
  绑定钱包账户用途钥在端侧生成的单联系人 AES-256-GCM 密文、以目标
  `cid_number` 计算的 HMAC `contact_id`、公开密钥上下文 `binding_revision + account_id`、
  nonce、MAC 和更新时间。派生上下文必须绑定
  `genesis_hash + cid_number + binding_revision + account_id + purpose`。同一钱包账户在
  新设备经一次明确钱包级授权后可重新派生相同用途密钥；换绑后的新账户在正式换绑
  作用域内使用自己的 child 派生新密钥。
  有当前账户签名时，同一次换绑流程必须先用当前账户解密 Chat、通讯录，再用新账户重
  加密；没有当前账户签名时，新账户可以控制 CID，但不能解密此前历史私有密文。联系人
  CID、联系人钱包账户、SS58、私人备注、公开昵称、关系明文及用途密钥禁止以明文进入 Cloudflare；
  Worker 不得持有、生成、密封、恢复或下发任何用户私有数据密钥。CID 注销必须立即硬
  删除全部通讯录密文。
- Chat 推送只能发送固定唤醒类型和发送方 `cid_number`，禁止把钱包账户当 Chat 身份，
  也禁止携带明文、密文、附件地址、会话摘要或通知预览
- 用户注销的唯一删除主键是 `cid_number`：当前绑定 `account_id` 只用于会话、finalized
  双向绑定复核和注销签名授权，不得用于缩小或推导删除范围。授权通过后必须先关闭该 CID
  的实时连接、撤销该 CID 跨历次换绑账户签发的全部短期凭证，再立即按 CID 硬删除
  Cloudflare 中的设备公钥、推送 Token、一次性 KeyPackage、防重放摘要、通讯录密文、
  社交关系、会员镜像、finalized 交易最小证明、充值订单、内容、媒体与用量数据；不得
  软删除、延期删除、按当前账户前缀猜测 R2 范围或保留恢复副本。D1 交易证明和充值订单
  必须在创建时固化 `cid_number`；链上/EVM 原始交易事实仍由公链保存。
- Cloudflare Session 明文 token 只允许返回客户端；KV 键与 D1 强一致注销索引只能保存
  `SHA-256(token)`。挑战和会话索引必须记录 `cid_number`：注销按 CID 撤销跨换绑全部
  会话与挑战，换绑吊销才允许用 CID + `previous_account_id` 缩小到账户级鉴权材料。禁止依赖
  KV 前缀枚举作为完整注销真源。
- CitizenApp 的 P-256 设备子钥按热钱包 `walletIndex` 存于 Android Keystore / iOS
  Secure Enclave：换绑到另一只钱包时必须使用当前新账户所属钱包的子钥，不得读取旧
  钱包；只有同一热钱包内换账户时才可复用同一 `walletIndex` 子钥并重签归属证明。
  整只热钱包删除、删除末账户级联钱包删除及全量清空钱包时，必须销毁全部账户 child、
  钱包 KEK 和该 `walletIndex` 的设备子钥；只有被删账户拥有本机当前激活 CID 绑定时，
  才清该 CID 当前绑定公开元数据和内存用途子钥。任一安全存储删除失败不得阻断其它项，
  全部尝试后统一报告；冷钱包和删除非末账户不得误删整钱包共享设备子钥。
- CitizenApp 的 Chat、MLS、附件和通讯录用途钥必须由独立设备数据钥硬件封装：Android
  使用无用户认证要求的 Keystore AES-256-GCM 钥，iOS 使用独立 application tag 的
  Secure Enclave P-256 ECIES 钥。AAD 必须精确包含 `wallet_index + genesis_hash +
  cid_number + binding_revision + account_id + purpose + context`。设备数据钥不得复用
  钱包严档 KEK 或 P-256 签名子钥；整钱包删除时必须连同封装硬件钥与密文 blob 清除。
- CID 的唯一控制凭证是链上当前绑定钱包账户。私有数据密钥只能由该账户的 child
  mini-secret 在 CitizenApp 本地直接派生，Worker、D1、R2、注册局和链节点都不得成为
  第二密钥持有人或恢复方。换绑 finalized 后，新账户立即接管 CID、公开数据、授权与
  付款职责，并使用自己的派生密钥处理后续私有数据。换绑前当前账户能签名时，客户端在
  同次换绑中只对 Chat 与通讯录执行端内重加密交接；不能签名时，新账户不能直接解密此前
  私有密文，流程也不得要求此前账户、设备、助记词或缓存参与。密码学上无法撤回此前账户
  已知的密钥。绑定版本必须单调推进，同版本不同创世、CID 或账户失败关闭。
- 账户 child 只允许在正式交易签名、明确钱包级鉴权、CID 注册/有效换绑交易签名、换绑
  密文交接、真实数据访问确认本地设备数据钥缺失/失效，或 Worker 在真实登录中明确返回
  `device_not_registered` 时读取。进入或切换广场、Chat、创作者、通讯录、会员/订阅页面
  不得检查、生成或阻断任何设备密钥；已有 P-256 子钥与设备用途钥必须直接静默使用。
  本地数据钥生成与 P-256 设备登记必须使用独立入口、独立全局 single-flight 和独立失败
  回滚：前者只派生并硬件封装缺少的数据钥，绝不调用设备登记或 Turnstile；后者只登记
  P-256 子钥，绝不生成、覆盖或删除本地数据钥。两者均按
  `(genesis_hash, cid_number, account_id)` 去重，相同钱包账户不得因 `binding_revision`
  变化再次读取 child。后台推送预热不得触发。CID 换绑目标 `account_id` 必须与当前不同；
  相同账户在读取私钥、构造交易或暂存交接数据之前直接拒绝。
- 换绑 finalized 后，App 只激活新绑定公开元数据并完成已明确授权的数据交接，不预生成
  本地数据钥、不登记 P-256 设备子钥；后续分别由真实数据缺钥和 Worker 未登记响应触发。
  App 随后撤销不匹配
  Session，等待此前 Chat HTTP/WebSocket/MLS 上下文完全关闭后再建立新上下文。Worker
  必须按 finalized 三元组清除旧挑战、Session、设备子钥、Chat 设备和 KeyPackage；实时
  连接关闭失败必须返回失败，禁止把旧连接仍存活的状态视为收敛完成。
- 广场 R2 删除和读取只能使用经 `account_id + post_id` 重新生成并与
  `square_uploads.object_keys_json` 逐项完全一致的规范对象清单。非法 JSON、空清单、
  多余键、错误账户路径、错误帖子路径或已发布帖子缺上传索引时必须在任何 provider、
  R2、D1 副作用前失败；禁止把损坏清单退化为空数组后继续删除索引。
- 广场帖子删除、权益到期清理和用户注销释放 `resource_totals` 时，存储总量扣减必须与
  对应 `square_media_assets`、帖子和上传索引删除放入同一个 D1 原子 batch。禁止提供
  单独释放总量的入口；跨存储清理中途失败时 D1 总量和内容索引必须同时保留供幂等重试，
  成功后不得因再次注销或重试而重复扣减全局容量。

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
- `runtime-benchmarks` 只能生成真实 benchmark 账户、签名和计时夹具，不得以 feature
  条件改变生产验签、权限、状态转换或错误结果。正式候选 WASM 构建必须显式禁用该
  feature，并由构建闸门拒绝误配。
- 正式候选 WASM 必须从空 `target` 构建；上传前必须对随后上传的同一份压缩 WASM
  检查 `RuntimeVersion.apis` 不含 FRAME Benchmark API，并通过 NodeGuard 的公民身份
  四签名域行为探针。只验证另一份本地 runtime、Rust 单元测试或源码 feature 列表不能
  代替最终 `:code` 验收。
- 本机部署只能从根 `citizenconsole/` 控制台进入；该目录属于本机私有运维工具，整目录
  必须由 Git 忽略，不得提交或推送；`.runtime/`、日志、编译产物和私密文件同样不得脱离
  该边界。服务只监听 `127.0.0.1`，使用随机 HttpOnly 会话 Cookie、严格 Origin 校验、
  单任务互斥和日志脱敏。
- CitizenConsole 总入口和所有本机 Secret 读取、写入、删除、使用必须经 Apple 签名的
  `com.gmb.citizenconsole.security` 原生应用调用 Touch ID；只允许
  `deviceOwnerAuthenticationWithBiometrics`，禁止设备密码回退。Secret 必须写入
  Data Protection Keychain，并使用 `kSecUseDataProtectionKeychain`、
  `biometryCurrentSet`、`WhenUnlockedThisDeviceOnly` 和独立 Keychain access group；
  签名、Provisioning Profile 或 entitlements 任一缺失时必须失败关闭。
- CitizenConsole 生产原生应用只能使用 `Developer ID Application`、Hardened Runtime
  和 Apple notarization；`get-task-allow` 必须不存在或为 `false`。原生程序内部和
  启动脚本都必须复验 Team ID、Bundle ID、Developer ID 证书 OID、签名资源完整性、
  嵌套 Node 签名、公证状态和调试授权，任何一项不符都拒绝启动；Apple Development、
  ad-hoc、未公证或启动时现场自动重签的构建不得管理生产 Secret。
- 原生程序必须作为 CitizenConsole 根进程，通过匿名 `AF_UNIX socketpair` 启动并连接
  已密封的 Node 子进程；禁止恢复可从终端任意调用的 Secret `get/put/delete` CLI、
  公共 Unix Socket、任意 Keychain 名称或任意 Touch ID 提示文案。Node、网页、动作脚本、
  充值代码和 Node 运行时必须封入同一个签名资源边界，代码被修改后必须无法启动。
- 本机部署 Secret 只允许保存在上述受生物识别保护的 macOS Keychain，远端流水线
  Secret 只允许保存在 GitHub Secrets；禁止明文 Secret 文件、浏览器回传、前端存储、
  日志输出、普通 `security` 命令读取、整服务枚举或 Wrangler OAuth 回退。
- `CitizenConsole · 充值发币` 是唯一允许一次 Touch ID 后在页面连接生命周期内持续持有
  内存 Secret 的模块；不设置时间超时，点击“锁定”、离开页面、连接断开或进程退出必须
  清除内存 Secret。发币私钥必须由原生根进程持有，只能经匿名管道交给已密封的一次性
  发币工作进程，普通 UI Node 不得读取、返回或持有该私钥。其他敏感动作仍逐次 Touch ID，
  不得复用充值发币解锁状态。
- CitizenConsole 所有修改类 HTTP 请求必须同时校验精确 `Origin` 和 Fetch Metadata；
  缺失来源或跨站请求一律拒绝。所有响应必须禁止缓存并设置 CSP、`frame-ancestors`、
  `nosniff`、Referrer Policy、Permissions Policy、COOP 与 CORP；部署子进程只能继承
  明确环境白名单和本动作需要的 Secret，不得展开继承控制台完整环境。
- Cloudflare 本机管理权限必须拆分为 `CF_DEPLOY_TOKEN`、`CF_DATA_TOKEN`、
  `CF_ZT_TOKEN`：分别限定部署、数据和 Zero Trust/DNS 资源；三者只保存在
  CitizenConsole 生物识别 Keychain。Worker 运行时 `CF_API_TOKEN` 保持独立，
  不得借用管理令牌。`CF_ZT_TOKEN` 必须包含 Access 应用/策略、Service Token、
  Tunnel 与限定 DNS 的管理权限，禁止使用 Global API Key。
- CitizenConsole 是生产 Secret 和部署配置的唯一控制面。Cloudflare 只保留一个
  production Worker 及其 production D1、KV、R2、Queue、Route 和 Secret；禁止创建或
  恢复 staging/test Worker、远端测试数据资源、测试路由或测试 Access 应用。GitHub
  Actions 只保留当前正式 workflow 实际引用的 Secret。
- 测试部署和 CI 无需密码；production、Release 和服务器部署在启动目标命令前必须逐次通过 Touch ID 生物识别，不允许设备密码降级。
- CitizenChain 的44个权威节点必须使用逐节点隔离的 Keychain 项保存服务器 IP、节点身份私钥和 GRANDPA 验证私钥；这些共识身份私钥永远不得共享。需要部署控制台管理的服务器统一使用 `deploy` SSH 身份，私钥只允许写入已配置节点的 Keychain 项和 GitHub Secret，不得留在 `.ssh`、仓库、明文清单或 workflow 普通输入中；本机只允许保留非机密的 `deploy.pub`。明确不使用该身份的节点不得强行写入。
- 节点密钥只允许覆盖写入，不允许网页读取旧值；写入前必须验证私钥推导的 PeerId/GRANDPA 公钥与权威节点公开目录一致。修改节点 IP、覆盖任何节点密钥和部署节点均必须先完成 Touch ID。

## 5. CitizenApp API 与媒体安全

- CitizenApp 唯一 API 入口为 `https://www.crcfrcn.com/api/*`；禁止恢复 staging/test
  API、`workers.dev`、Preview URL 或独立 API 子域名。
- 官网浏览器请求只允许精确 Origin `https://www.crcfrcn.com`；原生 App 无 Origin 时必须使用钱包 Session、P-256 设备逐请求签名、时间窗和一次性 nonce，不能仅凭 User-Agent、IP 或客户端声明授权。
- Cloudflare 钱包 Session 只验证已登记 P-256 设备子钥及其钱包归属，不得以 `System.Account` 是否存在、钱包余额或存在性存款作为登录门禁；需要链上身份、余额或业务资格的动作必须在各自业务入口独立校验。
- 登录挑战必须用 D1 条件更新原子 claim：账户、挑战编号、未消费状态和有效期同时命中
  才能签发 Session；并发重放只允许一个成功。Session/KV 写入失败时挑战仍保持已消费，
  并清除可能写入的孤立 Session，客户端只能重新申请挑战。
- 设备子密钥登记的 `issued_at` 必须是五分钟窗口内的安全整数，并用 D1 条件 UPSERT
  保证同一 `account_id` 严格单调递增；相同或更小 `issued_at` 一律拒绝，禁止设备换钥回滚。
- 首次设备绑定、设备换钥和风险升级必须通过 Turnstile；Stream webhook 使用提供商签名，
  不叠加设备签名。
- Worker 必须在解析 JSON 前限制请求体，并按 IP 哈希、钱包账户、接口类别分层限流。
- Cloudflare WAF 规则 `citizenapp-api-edge-limit` 对 production API 按 IP 执行
  60 次/10 秒的边缘阻断，阻断持续 10 秒；Stream 签名 webhook 必须排除，避免提供商
  回调被普通客户端限流误伤。
- Cloudflare Images 必须启用签名交付，Cloudflare Stream 必须启用 signed URL；D1、R2 manifest 和 Feed 禁止保存长期公开媒体 URL。
- 媒体上传必须在服务端同时校验单帖权益、月度图片/视频额度、活动上传数和全局媒体成本熔断；Chat 不进入媒体用量预算，也不得把消息或附件保存到 Cloudflare。
- `citizenapp/cloudflare/src/limits/catalog.ts` 是 Cloudflare 资源硬上限唯一真源；环境变量只能收紧，不能放宽。所有外部路由必须在 D1 前完成路由白名单和 `Content-Length` 检查，并在读取阶段继续按实际字节截断。
- 头像、背景、manifest 和广场图片必须经 Worker 校验实际字节、MIME、图片文件头、尺寸与 sha256 后才能写 R2/Images；禁止向客户端签发 R2 PUT 或 Images 直传地址。视频必须统一使用绑定精确 `Upload-Length` 和最长时长的 Stream TUS，并在 webhook 按实际时长、分辨率复核。
- Chat 附件只允许 WebRTC 设备直连；仅使用 STUN 发现候选，禁止配置、签发或保存附件中继凭证。直连失败时附件继续保留发送设备本机，不得回退 Cloudflare 中继或存储。
