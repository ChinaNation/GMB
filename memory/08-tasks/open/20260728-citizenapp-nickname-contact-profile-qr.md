# CitizenApp 昵称、通讯录、用户主页与 QR 用户码统一改造

状态：open（功能开发完成；待 S5 正式创世资产重生后执行真实管理员正向会话验收）

## 任务需求

- 钱包账户名称回归纯本机钱包标签，与公开用户昵称彻底解耦。
- 公开昵称继续使用现有 `display_name`，按身份主键 `cid_number` 保存并用于用户主页、广场、通讯录和二维码。
- 通讯录同时表达公开昵称、私人备注、`ss58_address`、`cid_number`；`account_id` 只作为内部账户标识。
- 联系人 `cid_number` 必须进入端到端加密载荷并支持跨设备恢复。
- 用户主页头像下依次展示公开昵称、SS58 地址及复制按钮、CID、三项计数；CID 不提供复制按钮。
- QR_V1 `k=3 user_contact` 增加 `cid_number`，清理二义性的旧 `contact_name`，CitizenApp、CitizenWallet、OnChina 前后端及协议文档同批切换，不保留兼容分支。

## 已确认边界

- 公开昵称唯一真源为现有 `display_name`，不得复制为通讯录第二真源。
- 私人备注字段使用 `contact_remark`。
- QR 用户码中的公开昵称字段复用 `display_name`。
- 不修改 `citizenchain/runtime/`。
- 每一步必须先提交技术方案，用户确认后才能执行。
- 每一步执行后必须更新文档、完善中文注释与测试、清理旧字段和残留，再提交下一步方案。

## 预计修改目录

- `citizenapp/lib/8964/`：公开昵称读写、用户主页展示和用户二维码入口。
- `citizenapp/lib/my/user/`：通讯录字段、密文载荷、同步和页面展示。
- `citizenapp/lib/wallet/`：钱包名称与公开昵称旧同步关系清理。
- `citizenapp/lib/qr/`：QR_V1 用户码模型及解析。
- `citizenwallet/lib/qr/`：冷钱包用户码解析同步。
- `citizenchain/onchina/frontend/core/`：OnChina 前端用户码解析同步。
- `citizenchain/onchina/src/core/qr/`：OnChina 后端管理员登录用户码解析同步。
- `memory/01-architecture/qr/`、`memory/05-modules/`、`memory/07-ai/`：协议和模块文档回写。
- 对应测试目录：跨端协议、资料、通讯录、主页和二维码测试。

## 主要风险

- QR 用户码是 clean cutover；任一端未同步会中断 OnChina 管理员登录。
- 旧 `walletName ↔ display_name` 同步器存在多设备状态，必须清理队列和旧 KV，不得留下双真源。
- 通讯录已有 2026-07-28 CID 主键重构，改造不得恢复以账户作为云端归属主键。

## 完成标准

- 钱包改名不再改变公开昵称，公开昵称修改不再改变钱包名称。
- 通讯录离线和跨设备恢复均能显示私人备注、SS58、CID，并从资料真源补公开昵称。
- 四端只接受新的 QR_V1 用户码结构，OnChina 管理员登录真实跑通。
- 相关自动化测试、真实 App/OnChina 验收、文档、注释和残留扫描全部完成。

## 实施记录

### Step 2：公开昵称与钱包名解耦（2026-07-28）

- `display_name` 收口为公开昵称唯一真源，按 `cid_number` 读取和缓存；
  `walletName` 仅保留为本机钱包标签。
- 资料编辑页不再读取或重命名默认钱包；钱包创建、导入和改名不再访问资料服务。
- 删除 `NicknamePublisher`、钱包名同步专用 `ensureSessionFor` 及旧同步测试。
- “我的”页缓存优先显示公开昵称，同一页面生命周期、同一 CID 最多后台刷新一次；
  钱包名 revision 广播在身份账户不变时直接忽略。
- 广场身份模型用 `displayName` 替代 `walletName`，发帖预览只消费公开昵称或稳定兜底。
- Isar 每次进程打开时幂等清除 `wallet_name_pending:*` 与
  `wallet_name_synced_at:*`，保留其它 `AppKvEntity`。
- 通讯录字段、用户主页布局和 QR_V1 跨端结构仍按后续已确认步骤实施，本记录不把
  整体任务提前标记完成。
- `flutter analyze --no-fatal-infos` 全量通过；`flutter test --concurrency=1`
  共 918 项通过、5 项按既有宿主能力条件跳过、0 项失败；Android debug APK
  构建成功。
- Pixel 8a（Android 16）覆盖安装后，钱包页仍显示既有本机钱包标签，
  “我的”页改为显示与钱包名不同的稳定公开昵称兜底；强制停止并重启后两者仍保持
  解耦，未修改钱包名或其它用户数据。
- 当前 Pixel 身份是未注册访客，因此真机验证的是“无 `display_name` 时按身份稳定
  兜底且不泄漏钱包名”；已注册身份的 `display_name` 更新与缓存回刷由资料 API、
  页面和专项自动化测试覆盖。
- 真机仍复现既有 `广场登录态响应不完整` 异常，堆栈位于广场首页通知会话；
  与本步昵称解耦无关，继续归广场会话任务处理。

### Step 3：通讯录 CID 真源、私人备注与密文重建（2026-07-28）

- `UserContact` 已彻底收口为 `cid_number + account_id + ss58_address +
  contact_remark + created_at + updated_at`；CID 是去重、增删改、待同步和跨端合并的
  唯一关系主键，私人备注允许空值，旧 `contact_name` JSON 直接拒绝。
- `contact_id` 改为索引钥对目标 CID 的 HMAC-SHA256；AES-256-GCM 载荷包含属主
  CID、联系人 CID、账户、SS58、私人备注和时间戳，AAD 绑定属主 CID 与不透明 ID。
  HKDF 域统一为 `citizenapp.account-data/contacts-cloud`，并用 `encryption` / `index`
  context 隔离，不新增版本化协议标识。
- 修复了新增联系人后因局部变量遮蔽而误用“对方账户”启动属主同步的既有缺陷。
- 当前用户名片码仍是本步骤输入边界：扫码先由 SS58 派生规范 `account_id`，再经链上
  双向绑定解析 CID 后入库；未绑定 CID 时拒绝。二维码钱包标签不写入私人备注；
  缺少 CID 的收款码不再兼作联系人码。QR 载荷结构本身留到下一步四端同批切换。
- 通讯录卡以公开资料昵称为主标题，私人备注、CID、SS58 分行展示；修改操作只编辑
  私人备注。公开资料缓存和用户主页直接按联系人 CID 寻址，不再为已入库联系人重复
  account→CID 链读。
- CID 换绑不删除、不重建、不全量重传联系人密文；新绑定账户使用自己的 child 直接
  派生新用途子钥，可以读取密文记录但不能解密旧账户加密的历史私有数据。账户间搬运
  和云端重建标记均已删除。
- 本地目标键为 `contact_book_by_cid:`、`contact_pending_by_cid:`、
  `contact_sync_by_cid:`；废弃账户分区只清理不读取。安全存储只保留当前绑定公开元数据，
  用途钥由当前账户在内存派生，废弃账户密钥名只删不读。
- 本步骤仍按已确认边界保存可离线读取的本地联系人副本；本地副本的字段级静止态
  加密归独立任务 `20260728-citizenapp-chat-local-data-encryption`，后续必须与聊天
  正文、搜索索引和当前绑定生命周期一起实施，不在本步骤建立第二套临时加密流程。
- 新增一次性破坏性迁移
  `citizenapp/cloudflare/migrations/0002_reset_contacts_for_cid_payload.sql`，SQL
  内存库实测执行后旧记录数为 0。本步骤没有执行本地/生产 D1 部署，生产执行仍需
  单独审核和授权。
- 验证检查点：`flutter analyze` 通过；CitizenApp 全量 923 项通过、5 项按既有宿主
  条件跳过；联系人/扫码/UI/钱包密钥定向 44 项通过；Worker 31 个测试文件 192 项
  通过且 `tsc --noEmit` 通过。最终补充边界后 `user_service_test.dart` 13 项继续通过。
- Android debug APK 构建并覆盖安装到 Pixel 8a 成功；真机进入“我的通讯录”无崩溃，
  当前设备是未注册访客，页面按安全门显示“注册后使用通讯录”，因此未修改链上身份、
  未添加真实联系人，也未绕过门控。完整联系人卡字段由真实页面 Widget 测试覆盖。
- 全量检查完成后，工作区另一条订阅 CID 主键改造线程开始改写 Subscription RPC，
  当前仓库短暂出现与本步骤无关的参数中间态错误；本步骤未覆盖或修补该线程文件。
  该并发状态下再次独立复核：通讯录与钱包密钥生命周期 29 项通过、联系人 Worker
  5 项通过、迁移 SQL 真实执行通过；全量 Analyze、Worker 全量与 `tsc` 仍由订阅
  参数中间态阻断。

### Step 4：用户主页身份信息布局收口（2026-07-28）

- 用户主页头像下收口为公开昵称、SS58 地址及复制按钮、独立 CID 行、关注/关注者/
  帖子三项计数；公开昵称继续只消费 `display_name` 或按 CID 稳定生成的本地兜底，
  不读取钱包名。
- 主页不再展示或复制原始 `account_id`。SS58 只由资料中的当前绑定规范 AccountId
  调用统一 `ss58FromAccountIdText()` 即时派生；复制完整 SS58 并提示
  “SS58 地址已复制”。
- CID 固定使用页面路由的 `cid_number` 身份真源，与 SS58 分行展示，不提供复制按钮；
  资料响应尚未加载或 AccountId 非法时显示“SS58：暂不可用”并隐藏复制入口。
- 展开头部高度调整为 372，为昵称、SS58、CID、签名、计数和可选订阅按钮预留空间；
  320px 宽度 Widget 验证无布局溢出。
- 主页专项 Widget 测试 18 项通过，覆盖公开昵称兜底、完整 SS58 派生与真实剪贴板
  目标、CID 无复制入口、非法 AccountId 从严降级、窄屏布局及原有主页交互。
- `flutter analyze` 全量通过；资料模块 60 项通过；CitizenApp 全量 925 项通过、
  5 项按既有宿主能力条件跳过、0 项失败；Android debug APK 构建成功。
- APK 已覆盖安装到 Pixel 8a。真机“我的”页显示现有公开昵称兜底，点击个人主页入口
  后按当前未注册访客状态明确提示“请先完成身份注册再查看个人资料”，未伪造 CID、
  未修改身份或绕过门控；因此本机无法展示已注册资料卡，字段顺序、复制目标和窄屏
  布局由真实 Widget 页面测试完成验收。

### Step 5：QR_V1 用户身份码四端 clean cutover（2026-07-28）

- `k=3 user_contact` 固定码已统一为严格三字段
  `cid_number + ss58_address + display_name`；CID 为无首尾空格的 1–32 UTF-8
  字节，SS58 必须使用本链 prefix 2027，公开昵称为无首尾空格的 1–40 个 Unicode
  字符。顶层和 body 均严格拒绝缺失、未知字段及旧 `contact_name`。
- CitizenApp 只有链上 CID↔AccountId 闭环命中的当前身份账户能生成 `k=3`；
  未注册账户和其它钱包子账户统一生成五分钟 `k=4 user_transfer` 临时收款码，
  链读失败从严拒绝。身份码昵称只来自公开资料缓存或按 CID 稳定生成的兜底，不读取
  钱包标签。
- CitizenApp 扫码添加联系人会把 SS58 解为 AccountId，再按链上绑定解析 CID，
  与二维码声明 CID 精确比较；不一致直接拒绝。二维码 `display_name` 不写入私人
  `contact_remark`。
- CitizenWallet 没有 CID 链上真源，因此只解析 `k=3`，账户详情收款彻底切换为
  五分钟 `k=4`，不再把离线钱包标签伪装成公开身份。
- OnChina 前端严格解析三字段并修复通用扫码弹窗误读不存在的 `body.address`
  缺陷；管理员登录页只展示 `display_name`。后端从 SS58 派生 AccountId，并要求
  二维码 `cid_number + account_id` 同时命中链上同一条 Active 管理员人员记录后
  才签发定向登录挑战；公开昵称不参与授权。
- 已同步 QR fixture、协议规范、扫码识别、统一命名、CitizenApp/CitizenWallet
  职责、OnChina 和模块技术文档；未修改 `citizenchain/runtime/`，未新增手写文件。
- 经用户对确切生成物路径单独确认后，OnChina 生产构建已删除旧
  `dist/assets/index-PCQS-5HR.js`，生成并引用
  `dist/assets/index-IHEWbXwx.js`；既有 CSS hash 未变化。真实本地生产预览由浏览器
  加载新 JS，管理员登录页与“公开昵称仅用于本页显示”文案均唯一出现，控制台 0
  错误，验收后已关闭预览服务。
- 全量回归结果：CitizenApp `flutter analyze` 通过，`flutter test
  --concurrency=1` 为 931 项通过、5 项按既有宿主能力条件跳过、0 项失败；
  CitizenWallet `flutter analyze` 通过、245 项测试全部通过；OnChina 前端
  `tsc -b` 和 Vite 生产构建通过，后端 Rust 155 项测试全部通过。
- CitizenApp Android debug APK 已覆盖安装到 Pixel 8a（Android 16），真机实际走通
  “我的 → 钱包 → 账户详情 → 账户二维码”，当前未注册访客账户显示“临时收款码，
  5 分钟内有效”，未再生成 `k=3` 身份码。真机仍记录到既有
  `广场登录态响应不完整` 异步异常，但 QR 页面未崩溃且堆栈仍属于广场会话模块，
  不在本步骤跨模块修补。
- CitizenWallet Android debug APK 已覆盖安装并冷启动到空钱包页，进程无 Flutter
  崩溃；设备没有现成冷钱包，未擅自创建或导入助记词，因此账户详情的真实页面只由
  245 项自动化测试（含 4 项账户详情专项）验收。当时发现旧 Isar 3.1
  `libisar.so` 未满足 16 KB ELF 对齐；该独立发布兼容问题已在任务
  `20260729-citizenwallet-android-16kb-isar` 中完成依赖和产物修复，真实 16 KB
  内核运行验收仍由该独立任务跟踪，不与本次 QR 协议改造混改。
- Step 5 的代码、注释、协议文档、生成物、全量自动化和当前可用真机页面验证已完成。
  由于本机没有运行 OnChina 后端、PostgreSQL 与可供登录的 Active 管理员身份，
  本步骤尚未完成“真实管理员用户码 → 钱包签名 → OnChina 会话”跨端验收，任务保持
  `open`；下一步必须在经确认的真实联调环境中完成该链路后，才能满足整体完成标准。

### Step 6：OnChina 管理员扫码登录真实服务联调（2026-07-29）

- Pixel 8a 运行态复核确认 CitizenApp 当前选中身份是“访客/匿名”；其现有账户
  `account_id` 为
  `0xc09ea9c7be5fbf694e815d7a70e2c88d83748c80c20757fcbb683d5f03363168`，
  对应 SS58 为
  `w5FwMkSmTSBVJWNoGuPe1cMsXjXyLxe3ekzSmH49hujitsk7Q`。CitizenWallet 真机为空钱包，
  因此本机不存在“已注册 Active 管理员 + 对应离线私钥”的真实正向联调凭据；本步骤
  未创建、导入或伪造助记词。
- 先只读启动既有 `gmb.dev` 开发链时，链头健康但节点守卫记录
  `FoundationGenesisIdentityChanged([83, 70, 71, 89])`；既有链上治理身份与当前
  源码配置不一致，当前 FSC 管理员也被 OnChina 从严拒绝。该旧链数据目录未删除、
  未迁移、未写入，服务随后正常停止。
- 为隔离旧链状态，在仓库外临时目录启动当前已编译二进制的全新
  `citizenchain-fresh` 链、独立 PostgreSQL 和 OnChina；本次实际创世哈希为
  `0x2c9138f8b807a279204e5331d9a55e5b57dcbbe97fc057fda929a3c014abcbc0`。
  OnChina 从当前链投影 49,593 个公权机构、99,232 个机构协议账户，抽样审计通过。
  其中机构协议账户精确由 49,593 个主账户、49,593 个费用账户、43 个省储行质押
  账户和 3 个独立基金账户组成；不得再把该数量写成管理员账户或管理员记录。
- 当前创世管理员名册按跨机构记录计为 1,025 条：国家储委会 19 条、43 个省储委会
  共 387 条、43 个省储行共 387 条、国家司法院 15 条、联邦注册局 215 条、联邦
  安全局 1 条、公民链基金会 1 条。联邦安全局和基金会的两条记录属于同一名程伟、
  同一 AccountId，因此唯一管理员 AccountId 为 1,024 个。本步骤没有创建这些管理员
  记录，只读取既有创世状态。
- 真实 HTTPS 使用既有 `OnChina Organization Root CA`，未通过 `-k` 绕过证书。
  macOS Chrome 正常加载 `https://onchina.local:8964/`，页面标题为“链上中国平台”，
  “管理员扫码登录”和 QR_V1 用户码说明均唯一出现，未显示非 HTTPS 警告；页面源
  `./assets/index-IHEWbXwx.js` 正常加载，OnChina 页面来源的控制台告警和错误均为 0。
  Codex 应用内浏览器因使用独立证书信任库返回 `ERR_CERT_AUTHORITY_INVALID`，不代表
  系统 Chrome 或 OnChina 证书部署失败。
- 以当前链公开可读的 FSC Active 管理员闭环
  `CN220-CTZN2-198805200-2026` /
  `0x0cb1d05c0c9c7f05679b60d6f24c7e5719a3985264e41c5e899d4822dca4b06b`
  发起严格三字段 `k=3 user_contact` 请求，真实接口返回 HTTP 200，并生成定向
  `k=1` 登录挑战；挑战 `b.u` 精确绑定该管理员账户。这里只使用公开链上身份校验，
  未持有或使用对应私钥。
- 真实 HTTPS 负向验收结果：`k=4 user_transfer` 和含旧 `contact_name` 的用户码
  均返回 HTTP 400 `ONCHINA_LOGIN_USER_CONTACT_INVALID`；访客账户和 CID/AccountId
  错配均返回 HTTP 403 `ONCHINA_LOGIN_ADMIN_NOT_ONCHAIN`；错误 session 返回
  HTTP 403 `ONCHINA_LOGIN_SESSION_MISMATCH`；错误账户返回 HTTP 403
  `ONCHINA_LOGIN_SIGNER_MISMATCH`；正确目标账户配伪签名返回 HTTP 422
  `ONCHINA_LOGIN_SIGNATURE_VERIFY_FAILED`。
- 伪签名后以正确 session 轮询仍返回 `PENDING`，证明验签失败不会消费挑战；90 秒
  到期后轮询返回 HTTP 200 / `EXPIRED`，再次提交返回 HTTP 410
  `ONCHINA_LOGIN_CHALLENGE_EXPIRED`。数据库实查该挑战 `consumed=false`，
  `admin_qr_login_results=0`、`admin_sessions=0`，没有因负向请求产生登录会话。
- 因缺少与 Active 管理员匹配的真实冷钱包私钥，本步骤不能真实完成
  `k=1 → CitizenWallet 签名 k=2 → OnChina SUCCESS/节点绑定/工作台 → 成功后重放拒绝`
  的正向闭环；这是当前唯一未完成验收项，任务继续保持 `open`，不得把负向接口结果
  冒充完整业务跑通。
- 本步骤只执行真实运行态、HTTP、数据库和页面验收，没有修改业务代码、协议、测试
  或中文注释，也没有触碰 `citizenchain/runtime/`。独立 PostgreSQL、全新链、浏览器
  标签页、设备 XML 和 App 进程均已清理；仓库外临时环境已移入 macOS 废纸篓，可恢复。

### Step 7.1：创世管理员 citizen-identity 闭环（2026-07-29）

- 继续正向联调前核实到：现有创世只在管理员和机构记录中引用程伟 CID/AccountId，
  `citizen-identity` 原先没有 GenesisConfig，导致 CitizenApp 无法从链上读取固定
  CID↔AccountId 闭环，真实页面只能生成 `k=4` 临时收款码，不能生成管理员
  `k=3 user_contact`。
- 经用户 runtime 二次确认后，为 `citizen-identity` 增加
  `initial_cid_bindings`。程伟永久 CID
  `CN220-CTZN2-198805200-2026` 保持不变，不按当前授权账户重新生成；账户保持
  `0x0cb1d05c0c9c7f05679b60d6f24c7e5719a3985264e41c5e899d4822dca4b06b`，
  登记来源使用既有联邦注册局 `ZS001-FRG07-249474503-2026`。
- 创世原子写入 Active `CidRegistry`、`AccountIdByCid` 与 `CidByAccountId`，只建立
  匿名身份闭环，不伪造投票或竞选身份。重复 CID、重复 AccountId、空 CID 或空登记
  来源会在创世阶段直接拒绝。
- `citizen-identity` 全量 62 项和 runtime 全量 52 项通过；
  `WASM_BUILD_FROM_SOURCE=1 cargo build -p node` 成功。
- 当前源码 fresh 链真实创世哈希为
  `0xb8a32949a2d0527e3522d9db315049de7b6ecbd3adee33979b7d5df795d79d6a`；
  RPC 逐字节读取确认登记来源、Active 状态、账户正向索引、CID 反向索引和
  `blake2_256(account_id)` commitment 全部匹配。
- 本步骤消除了 CitizenApp 生成真实管理员 `k=3` 用户码的链上前置缺口，但没有伪造
  私钥。CitizenWallet 当前仍为空钱包；按用户 2026-07-29 的执行顺序，必须先完成
  S5 正式创世资产重生这一非测试工作，再由用户在真机上手动导入对应助记词，完成
  `k=3 → k=1 → k=2 → SUCCESS/节点绑定/工作台 → 重放拒绝`。
- fresh 链已停止、RPC 已关闭，临时链目录已移入 macOS 废纸篓；本任务继续保持
  `open`，直到真实管理员正向会话闭环通过。

## 当前开发、发布与测试边界（2026-07-29）

- 本卡公开昵称、通讯录、用户主页、QR_V1 四端切换和创世管理员身份闭环的功能开发
  已全部完成，当前没有遗留业务代码开发项。
- S5 正式 chainspec、CitizenApp 轻链资产、公权机构快照和 Cloudflare 创世哈希重生
  属发布资产生成，不得用本地 fresh 链验收冒充正式冻结。
- 真实管理员正向会话闭环属于后置测试；在 S5 完成前不启动该测试。
