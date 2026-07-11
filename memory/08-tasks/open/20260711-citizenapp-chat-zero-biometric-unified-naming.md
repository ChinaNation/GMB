# 20260711 CitizenApp 聊天零生物识别与统一命名

## 状态

- 状态：open
- 当前阶段：第 3～6 步本地实现、自动化、真机与残留清理完成；`0010` 已由会员任务统一应用到 staging / production，Worker 发布随会员短名切换一并收口
- 产品边界：中文名“公民”，英文名 `CitizenApp`，模块 id/目录 `citizenapp`
- runtime 边界：签名域 `0x1A` 的权威常量仍位于 `citizenchain/runtime/primitives`。统一命名需要在数值和签名消息字节不变的前提下，将旧 Chat 钱包绑定名称改为 Chat 设备绑定；执行前必须按仓库规则单独取得 runtime 二次确认。除该权威命名、注释和金标名称外，不修改 runtime 业务逻辑。
- 远端边界：用户已允许本任务的 migration / Worker 发布并入会员任务；仍禁止 GitHub push 和远端 CI

## 任务需求

1. 修复首次进入聊天 Tab 连续弹出两次生物识别的问题。
2. 重新冻结授权边界：进入聊天、收消息、发消息、附件、轮询和实时连接不得读取钱包 seed，不得触发生物识别。
3. CitizenApp 聊天模块统一使用 `Chat` / `Mls` / `Account` 目标命名，彻底删除历史代码、目录、字段、注释、文档和测试残留。
4. 初始化必须按账户和设备 single-flight；Tab 初始化、App resume、轮询和 WebSocket 不得重复创建邮箱会话、设备登记或 KeyPackage 发布。
5. 完成自动化、真实本地 Worker 和 Pixel 真机验收；App 已解锁后进入聊天，CitizenApp 生物识别请求增量必须为零。

## 已确认根因

### 设备证据

- Pixel 8a / Android 16 私密空间真实记录了两个独立的 CitizenApp 强生物识别请求：request `169` 与 `170`。
- 两次请求均为纯强生物识别 Keystore 操作，`CredentialRequested=false`，不是允许设备密码的 App 锁验证。
- 第一次成功后约 2 秒又创建第二个请求，符合页面 resume 重入后重复读取严档 seed 的时序。

### 代码调用链

1. `ChatTab.initState()` 注册生命周期监听，并立即 `_reload(syncFirst: true)`。
2. `_reload()` 调用 `ChatRuntime.syncPending()`。
3. `syncPending()` 进入 `_ensureOwnerContext(prepareMailbox: true)`，随后执行邮箱 session、设备登记和 KeyPackage 发布。
4. 设备绑定缓存不存在时，`_ensureDeviceRegistered()` 调用 `_signWalletPayload()`。
5. `_signWalletPayload()` 调用 `WalletManager.signWithWallet()`；后者读取硬件严档 seed，触发第一次强生物识别。
6. 系统生物识别窗口使 App 经历非 resumed → resumed；`_LifecycleObserver` 在 resumed 时再次 `_reload(syncFirst: true)`。
7. 第一次网络登记尚未完成、缓存尚未写入，第二条初始化仍判断为未绑定，再次调用 `signWithWallet()`，触发第二次生物识别。

### 状态机缺陷

- `_reloadGeneration` 只防止旧请求结果覆盖新 UI，不取消已经开始的同步、设备登记或硬件解密。
- `_ensureOwnerContext()`、`_ensureMailboxReady()`、`_ensureDeviceRegistered()` 没有共享的 in-flight future。
- `_pauseSync()` 只停止轮询和实时连接，不会终止正在进行的 `_reload()`。
- 因此当前并不只存在“固定两次”问题；网络更慢或生命周期再次抖动时，结构上可能产生更多重复初始化。

## 唯一授权边界

| 操作 | 唯一签名密钥 | 生物识别 |
|---|---|---|
| 进入聊天、拉取密文、ack、WebSocket | P-256 设备子钥/session | 禁止 |
| 聊天 MLS 设备登记 | P-256 设备子钥 | 禁止 |
| 发布/刷新 MLS KeyPackage | MLS 设备密钥 + P-256 session | 禁止 |
| 发送消息、附件加解密 | MLS 设备密钥 | 禁止 |
| 转账、投票、切换默认身份等动钱动权 | sr25519 钱包主私钥 | 每次强生物识别 |

聊天模块不得再引用以下能力：

- `WalletManager.signWithWallet()`
- `WalletManager.verifyWalletAccess()`
- `SecureSeedStore.readSeed()`
- 冷钱包二维码聊天设备绑定签名
- 任何 sr25519 钱包主密钥聊天绑定流程

## 新设备绑定协议

1. CitizenApp 先使用现有硬件 P-256 设备子钥静默完成 Worker challenge/session。
2. 聊天设备绑定 payload 只包含当前 session owner、MLS device id、MLS device public key、expires at 和 nonce。
3. payload 使用 `OP_SIGN_CHAT_DEVICE_BIND=0x1A` 域，由同一 P-256 设备子钥签名；只改权威名称和签名算法调用方，不改变 op tag 数值及 `signing_message` 原语。
4. Worker 的 `owner_account` 只允许从已验证 session 派生，不再接受客户端提交的 owner 真源。
5. Worker 从 `square_device_subkeys` 读取该 owner 已登记的 P-256 公钥并验证设备绑定签名。
6. 签名验证成功后写入 `chat_devices`；旧 sr25519 钱包绑定签名不得继续被接受。
7. 旧 `chat_devices` 与 `chat_keypackages` 在新协议部署时清理并重新登记；不保留双验签或兼容分支。
8. `chat_envelopes` 属于用户密文投递数据，不因认证方式换代擅自删除；新协议不增加旧格式解析分支。

## single-flight 与页面活动规则

### ChatRuntime

- 唯一入口：`ensureReady(ownerAccount)`。
- single-flight key：`ownerAccount + deviceId + devicePublicKey`。
- 相同 key 的并发调用必须返回同一个 in-flight future。
- 成功后复用仍有效的 session、device binding、KeyPackage 和 transport context。
- 失败时只清除命中的 future/context，下一次允许重试；旧失败不得清除新账户上下文。
- 默认账户切换、退出登录、设备密钥轮换或本机聊天数据清理时，精确失效对应 key。

### ChatTab

- 顶层 `IndexedStack` 保活不等于 Tab 活跃。
- AppShell 必须向 ChatTab 提供唯一活动状态；只有聊天 Tab 当前可见且 App resumed 时才能启动同步。
- `initState`、Tab 进入、App resume、轮询和 WebSocket 通知全部调用同一个 coordinator，不得分别实现初始化。
- App pause 只停止轮询/WebSocket；恢复时复用 single-flight/context，禁止创建第二套登记流程。

## 统一、精简命名

### 产品与模块

| 语境 | 唯一名称 |
|---|---|
| 中文产品名 | 公民 |
| 英文产品名 | `CitizenApp` |
| 模块 id/目录/package | `citizenapp` |
| 用户界面功能 | 聊天 |
| Dart/TypeScript 业务前缀 | `Chat` |
| 密码学专有名 | `Mls` |
| 线协议名 | `GMB_CHAT_V1` |

### 类型和字段

| 语义 | 唯一目标名 |
|---|---|
| Tab / 运行态 / 消息流 / 存储 | `ChatTab` / `ChatRuntime` / `ChatFlow` / `ChatStore` |
| Cloudflare transport / transport 接口 | `ChatCloudTransport` / `ChatTransport` |
| 外层信封 / 设备 / KeyPackage | `ChatEnvelope` / `ChatDevice` / `MlsKeyPackage` |
| 账户角色 | `ownerAccount` / `senderAccount` / `recipientAccount` / `requesterAccount` / `peerAccount` |

角色字段跨语言唯一映射：

- Dart/Rust：`ownerAccount / senderAccount / recipientAccount / requesterAccount / peerAccount`
- JSON/SQL/Proto：`owner_account / sender_account / recipient_account / requester_account / peer_account`

不得保留旧 getter、旧 JSON key、旧 Proto 字段别名、旧 typedef、deprecated wrapper、双目录或 import 转发壳。

## 数据处理边界

- 用户已确认严格删除历史聊天命名和旧设备绑定。
- 当前 Pixel 测试空间的本机聊天测试数据在真机验收前清理，不实现旧 Isar 模型迁移或双读。
- Proto 重命名保持同一业务字段的稳定 field number，不建立旧 message/type 兼容入口。
- Cloudflare 通过新 migration 清理旧 `chat_devices/chat_keypackages`，重新以 P-256 绑定登记。
- 远端 migration 和 Worker 发布必须另行获得明确部署许可；未获许可时只做本地 D1/Worker 运行态验收。

## 分步骤实施

### 第 1 步：冻结方案与任务卡

- [x] 写入真实设备与源码根因。
- [x] 冻结聊天零生物识别授权矩阵。
- [x] 冻结 P-256 聊天设备绑定协议。
- [x] 冻结 single-flight、Tab active 和生命周期规则。
- [x] 冻结 CitizenApp/Chat 类型、字段、目录和协议命名。
- [x] 明确严格数据清理、仅两个已二次确认 primitives 文件允许命名修改，以及远端部署权限边界。

### 第 2 步：Worker P-256 聊天设备绑定

- [x] 已取得两个 primitives 路径的 runtime 二次确认；`0x1A` 已改名为 `OP_SIGN_CHAT_DEVICE_BIND`，数值和金标 message bytes 不变，并删除不再允许的 QR 动作常量。
- [x] 聊天设备登记的账户真源已改为 session owner；客户端提交 `owner_account` 明确拒绝。
- [x] Worker 已使用 `square_device_subkeys.p256_pubkey` 验证 Chat device binding domain 签名。
- [x] Worker 已删除 sr25519 Chat 设备绑定验签及旧签名常量。
- [x] 已新增 `0010_chat_device_binding.sql`，清理旧设备绑定和旧 KeyPackage，保留用户密文 envelope，并新增一次性 nonce 重放闸门。
- [x] 已覆盖合法签名、伪造 owner、错误签名、过期、重放和有效旧 sr25519 签名拒绝测试。
- [x] 已完成本地 Worker/D1 验收，未部署远端。

#### 第 2 步执行记录（2026-07-11）

- `npm run typecheck`：通过。
- Worker 全量 Vitest：18 个测试文件、111 个测试全部通过。
- `cargo test -p primitives --test sign_golden`：1 个金标测试通过；`0x1A` 的 `message_hex=ecfabbf1...a1167f` 未变化。
- 本地 D1 migration：`0008`、`0009`、`0010` 全部成功；`chat_devices=0`、`chat_keypackages=0`，`chat_envelopes` 表保留，nonce 表与过期索引存在。
- 本地真实 Worker：合法 P-256 登记返回 200；相同 nonce 重放返回 409；请求体提交 `owner_account` 返回 400。
- 验收使用的本地假 session、P-256 公钥、设备和 nonce 已清理；未执行 staging/production migration、Worker deploy、Git push 或远端 CI。
- 远端切换顺序固定为：先应用 `0010_chat_device_binding.sql`，再发布新 Worker，随后发布完成第 3 步的 CitizenApp；任何远端动作仍需用户单独授权。

### 第 3 步：CitizenApp Chat 原子改造

- [x] `lib/chat`、`test/chat`、`chat/proto` 原子迁移完成，旧目录和转发壳已删除。
- [x] 所有业务类型和账户字段按本卡目标表统一改名。
- [x] `ChatRuntime.ensureReady(ownerAccount)` 实现按账户/设备 single-flight。
- [x] Chat device binding 改用 P-256 设备子钥，彻底删除钱包 seed 签名调用。
- [x] AppShell 向 ChatTab 提供唯一 active 状态；隐藏 Tab 不初始化、不轮询、不连接实时通道。
- [x] init/enter/resume/poll/realtime 共用同一 coordinator。

### 第 4 步：Proto、Rust FFI、Isar 与签名域收口

- [x] Proto、生成 Dart、Rust native、FFI 导出统一为 `GMB_CHAT_V1` / Chat / Mls 命名。
- [x] Isar collection、字段和生成物统一 Chat / Account 命名。
- [x] 删除 CitizenWallet、QR 和 Signer 的旧聊天钱包绑定动作、payload 和测试。
- [x] 不保留 import 转发、deprecated 类型、旧字段读写或协议双轨。

### 第 5 步：自动化、文档与残留清理

- [x] Widget：隐藏 Tab、首次进入、resume burst 和快速切 Tab 只初始化一次。
- [x] Runtime：sync/realtime/send 并发复用同一 context；失败精确释放并可重试。
- [x] Wallet：聊天全流程 `readSeed/signWithWallet/verifyWalletAccess` 调用次数为 0。
- [x] Worker：P-256 绑定安全用例全部通过。
- [x] Proto/Rust/Isar/附件/删除/账户切换回归全部通过。
- [x] 全仓精确残留检查对旧目录、旧类型、旧字段、旧协议和旧文案返回 0；依赖 lockfile 完整性 hash 的随机子串不属于业务命名。
- [x] 7 个历史 ADR / 任务卡已迁移到确认的 Chat 目标路径，正文、任务编号和索引同步完成。
- [x] 更新 Chat、Wallet、CitizenApp 架构、统一协议和任务文档。

#### 第 3～5 步执行记录（2026-07-11）

- Chat / Isar / 页面相关测试：40 项通过；其中 runtime 并发三次 `ensureReady` 只执行一次登录签名、一次设备绑定、一次设备登记和一次 KeyPackage 发布，随后发送继续复用同一 context。
- native OpenMLS：重建 macOS arm64 `libsmoldot.dylib` 后 4 个 FFI / 持久化会话 / mailbox 闭环用例真实执行通过；Rust crate 4 项测试通过。
- Worker：TypeScript 检查通过，18 个测试文件共 112 项通过；本地 D1 migration 已全部应用。
- CitizenWallet：静态检查零问题，全量 142 项测试通过；聊天设备绑定 QR 动作、decoder、标签和测试已删除。
- runtime 金标：`cargo test -p primitives --test sign_golden` 通过，`0x1A` 的 message bytes 未改变。
- CitizenApp 静态检查只有两个与本任务无关的既有 info；Chat 代码无 warning/error。
- CitizenApp 全仓测试除 4 个既有个人多签历史测试外共 510 项通过；该测试在 host smoldot 可加载时会真实连链，把本地模拟中的 active 提案判为链上不存在并删除，单文件复跑同样失败，调用链不经过 Chat。本任务不越界修改个人多签模块。
- 精确内容扫描已排除依赖完整性哈希、图片 base64 和普通英文单词的误报；正式代码、正文、目录和文件名的旧聊天命名均为零。

### 第 6 步：真实运行态验收

- [x] 本地 Worker + 本地 D1 migration 真实启动和 API 验收。
- [x] 重建 ARM64-only APK，验证 ABI、16 KiB 对齐和签名。
- [x] Pixel 私密空间覆盖安装并启动，目标 Chat Isar schema 打开后会话列表为空。
- [x] App 已完成入口解锁后进入聊天，记录进入前后 BiometricService request id；增量为 0。
- [ ] 设备登记一次、KeyPackage 发布一次、WebSocket/轮询/收发/附件正常。
- [x] 反复切 Tab 和前后台切换不弹生物识别、不出现 ANR/crash；锁屏恢复不额外触发，避免把系统解锁认证混入 App 增量统计。
- [x] `0010_chat_device_binding.sql` 已应用到 staging / production；Worker 发布与发布后复验转由会员任务统一执行。

#### 第 6 步当前记录（2026-07-11）

- Android native 已按 `aarch64-linux-android` 重建；APK 内唯一 ABI 为 `arm64-v8a`，`libsmoldot.so` 为 64 位 ARM ELF。
- debug APK 构建通过，16 KiB zipalign 校验通过，APK v2 签名校验通过。
- APK 已覆盖安装到 Pixel 8a 私密空间并真实启动；Chat 页面显示“暂无会话”，没有停留在初始化或最终性验证文案。
- 私密空间解锁后基线为 fingerprint user 10 request `171`；首次进入聊天、连续多次切出/切回、Home 后恢复，最终 request 仍为 `171`，CitizenApp 生物识别请求增量为 0。
- 同一轮 logcat 未出现 `org.citizenapp` 的 FATAL EXCEPTION、ANR 或 App BiometricPrompt。
- staging / production 远端 D1 均已应用 `0010_chat_device_binding.sql`；只读复核确认 `chat_devices=0`、`chat_keypackages=0`、`chat_device_binding_nonces` 表存在，`chat_envelopes` 表保留。
- Worker 正式发布必须与会员短名 Secret 切换原子完成，避免发布后 Stripe、链 RPC 或 R2 因新 Secret 缺值而不可用；该阻塞和发布后 Chat 复验统一记录在会员任务第 7 步。

## 预计修改目录

- `citizenapp/lib/`：AppShell 活动 Tab 真源、Chat 业务代码、Isar、Signer/QR 旧绑定删除；涉及代码、生成物和残留清理。
- `citizenapp/test/`：Chat、Wallet、Isar、生命周期和零 seed 读取回归；涉及测试和旧目录清理。
- `citizenapp/chat/proto/`：Chat Proto 唯一源；涉及协议代码与生成物同步。
- `citizenapp/rust/src/`：Chat/Mls native 与 FFI 统一命名；涉及代码和旧导出清理。
- `citizenapp/cloudflare/src/chat/`：P-256 设备绑定、session owner 真源；涉及 Worker 代码。
- `citizenapp/cloudflare/migrations/`：旧设备绑定/KeyPackage 清理；涉及新增 migration，不删除 envelope。
- `citizenapp/cloudflare/test/`：P-256 绑定安全与旧签名拒绝测试；涉及测试。
- `citizenchain/runtime/primitives/src/`：经单独二次确认后统一 `0x1A` 权威常量和注释命名；只涉及命名，不改 runtime 业务逻辑。
- `citizenchain/runtime/primitives/tests/fixtures/`：经单独二次确认后同步金标向量名称，message bytes 必须保持不变；涉及测试数据命名。
- `memory/01-architecture/citizenapp/`：CitizenApp 聊天授权与模块命名；涉及文档。
- `memory/05-modules/citizenapp/chat/`：Chat 技术文档唯一目录；涉及文档并删除旧 `im/`。
- `memory/07-ai/`：统一字段、协议和签名域；涉及文档。
- `memory/08-tasks/open/`：本任务执行记录和真实验收结果；涉及文档。

## 主要风险

1. Worker 与 App 设备绑定协议必须协同切换；远端先后顺序错误会导致聊天设备无法登记。
2. Isar/Proto/Rust/FFI 全量改名范围大，任何旧生成物或字符串残留都会形成双轨。
3. 清理旧设备绑定后，所有测试设备必须重新登记并重新发布 KeyPackage。
4. 生命周期 single-flight 若只在页面层实现，发送/附件/实时入口仍可能绕过；唯一锁必须下沉 ChatRuntime。
5. 不能以“不再弹两次”作为验收；目标是进入聊天新增生物识别请求为零。

## 完成标准

- 公民聊天全流程不读取钱包 seed，不触发生物识别。
- ChatRuntime 同账户/设备并发初始化只有一个 in-flight future、一次设备登记和一次 KeyPackage 发布。
- 隐藏聊天 Tab 不初始化；进入、resume、轮询、实时通知共用唯一 coordinator。
- Worker 只接受 P-256 设备子钥绑定，账户只来自 session；旧 sr25519 Chat 钱包绑定签名被拒绝。
- CitizenApp 正式代码、测试、生成物、协议和文档只使用 CitizenApp/Chat/Mls/Account 目标命名，旧命名精确残留为零。
- 本地 Worker、全量自动化、ARM64 APK 和 Pixel 真机运行态验收全部通过。
- `citizenchain/runtime/` 仅允许在单独二次确认后修改上述两个 primitives 路径的权威名称；不得产生其他 runtime diff。未在无许可情况下触碰 GitHub 远端或 Cloudflare 远端。
