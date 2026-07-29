# USER 模块技术文档

## 1. 模块目标

`lib/my/user/` 负责 CitizenApp 的"我的 / 用户"模块，当前覆盖以下能力：

- 用户背景图上传与更换
- 用户头像上传与更换
- 按 `cid_number` 读取、缓存和编辑公开昵称
- 用户二维码生成与展示
- 通讯录扫码导入、私人联系人备注、端到端加密云同步和跨设备恢复
- 电子护照入口展示

## 2. 文件结构

- `lib/my/user/user.dart`
  - 用户主页 `ProfilePage`
  - 用户资料编辑页 `ProfileEditPage`
  - 二维码页面 `UserQrPage`
- `lib/my/user/user_service.dart`
  - 用户资料模型与持久化
  - 用户二维码载荷模型
- `lib/my/user/contact_service.dart`
  - 通讯录模型、按身份账户隔离的 Isar 缓存、端侧加解密、Cloudflare 同步和待同步操作
- `lib/my/user/contact_book_page.dart`
  - 通讯录页面、搜索、同步状态和单条联系人卡片

相关协作模块：

- `lib/wallet/pages/wallet_page.dart`
  - 管理热钱包档案和当前默认账户
  - `walletName` 只作为本机钱包标签，不读写公开昵称
- `lib/my/myid/`
  - 电子护照页面和链上唯一身份状态服务
  - `identity_badge_snapshot_store.dart` 只保存按钱包账户隔离的公开身份徽章展示信号
  - “我的”页面只提供入口和头像认证角标，不承载电子护照设置流程

## 3. 数据模型

### 3.1 用户资料 `UserProfileState`

字段：

- `avatarPath` — 本地头像路径
- `backgroundPath` — 本地背景图路径

展示规则：

- 当前用户账户唯一取自 `WalletManager.getDefaultWallet()`；`account_id` 是账户真源，
  `ss58_address` 只作展示。
- 公开昵称唯一真源是按 `cid_number` 寻址的 `CitizenProfile.displayName`
  （接口字段 `display_name`）。
- `walletName` 是本机钱包标签，只在钱包列表、详情和账户选择场景展示，不能作为
  公开昵称、缓存或回退值。
- `display_name` 缺失时，由 `ProfilePresentation` 优先按 `cid_number` 稳定选择
  本地默认昵称；无 CID 的纯访客才按当前身份账户稳定兜底。禁止回退为完整或截断账户。

设计说明：

- 用户修改公开昵称只走 `PUT /v1/square/profile`，成功后写
  `CitizenProfileCache`；不得调用 `WalletManager.renameWallet`。
- 钱包改名只更新 Isar `WalletProfileEntity.walletName`；不得创建资料会话、请求
  Cloudflare 或触发广场身份刷新。
- “我的”页先用缓存或稳定默认昵称直接渲染，同一页面生命周期、同一 CID 最多
  后台刷新一次；反复进入不重复联网，也不显示等待昵称的转圈。
- 已删除旧 `NicknamePublisher` 与专用 `ensureSessionFor`。数据库打开时幂等删除
  `wallet_name_pending:*`、`wallet_name_synced_at:*`，不保留同步队列或双轨逻辑。
- 用户设置的公开头像和背景上传 Cloudflare R2；`avatarPath/backgroundPath` 只承接旧本机图片迁移和“我的”页即时显示，迁移成功后清空
- 用户未设置或真实图片读取失败时，从 `assets/profile_defaults/` 11 张本地照片中按账户稳定选择头像和背景，两个位置避免使用同一张图

### 3.2 用户二维码 `UserContactBody`

协议号：

- `QR_V1`

字段：

- `p` — 协议标识，固定 `QR_V1`
- `k` — 二维码类型，固定 `3`
- `b.cid_number` — 当前身份账户链上闭环绑定的永久 CID
- `b.ss58_address` — 当前身份账户的 prefix 2027 SS58 展示地址
- `b.display_name` — 公开昵称或按 CID 稳定生成的公开兜底；不使用钱包标签或私人备注

只有当前链上身份账户能生成固定 `k=3`。未注册账户与其它钱包账户只生成五分钟
`k=4` 临时收款码；扫码添加时必须复核二维码 CID 与 SS58 派生 AccountId 的链上
绑定一致，不能把二维码昵称写入 `contact_remark`。

### 3.3 通讯录 `UserContact`

字段：

- `cidNumber` — 对方永久身份主键 `cid_number`，关系去重与增删改唯一使用该字段
- `accountId` — 对方规范 `account_id`，严格为小写 `0x` 加 64 位十六进制
- `ss58Address` — 由同一账户派生的 SS58 展示地址（当前链 `ss58 = 2027`）
- `contactRemark` — 当前用户为该联系人保存的私人备注，对应存储字段 `contact_remark`，允许空值
- `createdAt` / `updatedAt` — 毫秒时间戳

公开昵称、头像、背景、个性签名、链上身份和会员徽章不复制进联系人记录，统一按
`cid_number` 读取 `CitizenProfile`。因此通讯录、广场、聊天和关注列表始终进入同一个
`UserProfilePage`，不存在第二套联系人公开资料。

公开资料缺失时，各入口统一使用 `ProfilePresentation` / `ProfileAvatar` 的本地稳定
默认资料；联系人私人备注只在通讯录显示，钱包账户只在明确的账户行显示，不能充当昵称。

## 4. 持久化方案

### 4.1 用户资料

存储：`SharedPreferences`，键 `user.profile.state.v2`

内容：JSON 对象，只保存 `avatar_path`、`background_path`。账户、SS58 和昵称不在
`UserProfileState` 重复持久化；公开昵称从 `CitizenProfileCache` / 资料接口读取，
钱包标签只从钱包档案读取。

### 4.2 通讯录

本机缓存复用 Isar `AppKvEntity`，按 CID 当前绑定的身份账户 `account_id` 隔离：

- `contact_book_by_account:<account_id>`：解密后的本机通讯录缓存
- `contact_pending_by_account:<account_id>`：按联系人 CID 记录的添加/改备注/删除操作
- `contact_sync_by_account:<account_id>`：最近一次同步阶段、时间和错误状态
- `contact_cloud_reset_by_account:<account_id>`：身份换绑后旧云端密文尚未全量删除的持久标记

Cloudflare D1 `square_contacts` 只保存端侧 AES-256-GCM 密文、HMAC `contact_id`、
nonce、MAC 和更新时间；Worker 不接收联系人 CID、账户、SS58 或私人备注明文。
`contact_id` 固定为索引钥对目标 CID 的 HMAC-SHA256。AES-GCM 载荷包含属主 CID、
联系人四字段与时间戳，AAD 包含属主 CID；通讯录密钥由身份账户 child 经
HKDF-SHA256 的 `citizenapp.contacts/encryption` 与 `citizenapp.contacts/index`
域隔离派生并保存在设备安全存储。

旧 `contacts:` / `contact_pending_ops:` / `contact_sync_state:` 在数据库打开时直接
删除；旧 `wallet_contacts_key_v1_` 密钥只删不读；旧 `contact_name` JSON 和
`citizenapp.contacts.v1` 密文不迁移、不兼容。D1 旧密文由一次性
`0002_reset_contacts_for_cid_payload.sql` 清空，生产执行必须另行人工审核。

### 4.3 电子护照

电子护照状态归属 `lib/my/myid/MyIdService`，用户模块不直接读写电子护照状态。
电子护照页始终展示“匿名访客”“公民 · 投票身份”“公民 · 竞选身份”三张卡。当前身份卡排在首位并唯一标记“当前身份”；只有当前投票或竞选身份卡展示真实值，非当前公民卡只展示字段名称，不展示占位值、示例值或当前用户数据。匿名访客卡固定显示“没有公民身份信息”。
投票身份先由默认钱包反查永久 CID，校验 `CidRegistry` Active 与 CID↔钱包双向绑定，再读取 `CitizenIdentity::VotingIdentityByCid`：投票账户、公民身份 CID 号、居住选区、身份状态、投票身份有效期。竞选身份在此基础上读取 `CandidateIdentityByCid`，增加公民姓、名、性别、出生日期和出生地。
状态由链上 `citizen_status` 和护照有效期窗口派生，不再使用 OnChina 本地状态接口或 `myid.*` 本地档案缓存。
链读取或解析失败时三卡仍保留，但全部不标记当前身份、不展示真实值，并明确显示读取失败；不得把未知链状态静默降级成匿名访客。
用户主页头像右下角认证图标使用 `IdentityBadgeSnapshotStore` 中当前默认钱包的 `visitor/voting/candidate` 公开展示快照；该快照不保存护照详情，不作为授权或身份真源，也不能替代电子护照页真实链查询。
“我的”页面初次进入只读快照，不启动 smoldot；轻节点已被其他主动链流程启动并进入 operational 后，通过可取消监听为当前钱包刷新一次快照，不轮询。

## 5. 页面与交互流程

### 5.1 用户主页

页面元素：背景图、头像、公开昵称、二维码图标、右箭头、钱包/身份入口；
“个人服务”固定按“创作者 → 通讯录 → 会员｜订阅”排列，随后是设置入口。

### 5.2 用户资料页 `ProfileEditPage`

自上而下：
1. 用户二维码（当前默认账户不存在时显示占位提示）
2. 用户头像行（左侧头像 + 右箭头，点击换头像）
3. 公开昵称行（显示 `display_name`，点击编辑后只更新公开资料）

### 5.3 公开昵称与钱包名边界

- 用户资料页改公开昵称 → `CitizenProfileApi.updateProfile(displayName)` → 更新资料缓存。
- 钱包详情页改名 → `WalletManager.renameWallet()`，只更新本机钱包标签。
- 切换身份账户后按新 CID 读取公开资料；钱包改名广播因账户不变而被“我的”和广场
  页面直接忽略。

### 5.4 电子护照入口

1. “我的”页面点击电子护照入口
2. 跳转 `lib/my/myid/MyIdPage`
3. `MyIdService` 只取当前默认热钱包，并读取 finalized 永久 CID 身份闭环与 `VotingIdentityByCid` / `CandidateIdentityByCid`
4. 访客身份按“匿名访客 → 投票身份 → 竞选身份”排序；投票身份和竞选身份分别把对应当前卡移到首位
5. 当前公民卡展示真实字段值，非当前公民卡只展示字段名称；竞选身份的非当前投票卡不得重复展示投票身份值
6. 页面不得提供选择钱包、更换钱包、钱包二维码或扫码签名入口

电子护照详情页属于主动链流程，会启动并等待轻节点同步；这与“我的”首页头像徽章只读快照的边界不同。

### 5.5 通讯录

- 通讯录所属身份账户唯一来源是 `IdentityAccountCache`；页面和服务均不接受交易付款钱包或调用方账户覆盖。`UserContactService.getContacts()/sync()`只读写当前 CID 身份账户对应的 Isar 缓存与当前 Session CID 的 Cloudflare 密文。
- 扫码添加（`QrScanMode.contact`）只接受用户码：先按 SS58 派生 `account_id`，再经链上双向绑定解析 CID；对方未绑定 CID 时拒绝。收款码不再兼作联系人码。
- 支持修改可留空的私人备注、按 CID 删除、搜索和下拉同步
- 页面先显示按身份账户隔离的 Isar 缓存，再后台刷新 Cloudflare 密文和按 CID 寻址的公开资料
- 单条联系人以公开昵称为主标题，并分别显示私人备注、CID、SS58、头像、身份徽章和个性签名
- 单条联系人三点菜单固定为“转账、私信、修改备注、删除联系人”；删除项使用危险红色。备注表单自行管理输入生命周期，取消或保存后不得留下已销毁输入控制器。
- “转账”只把联系人 SS58 账户预填为链上支付收款地址，不填写金额、不签名、不提交；“私信”复用 `openDirectChat()`进入统一一对一聊天。
- 普通模式点击联系人进入唯一 `UserProfilePage`；不保留联系人详情副本
- 交易页通讯录与“我的”入口使用同一页面、同一身份和同一联系人数据；选人模式只改变点击后的返回动作：返回 SS58 地址填入收款栏，不做 AccountId hex 转换。
- 交易页选择的钱包只决定付款和签名账户，不得改变通讯录属主 CID；切换付款钱包不切换通讯录。

## 6. 依赖

- `image_picker`、`qr_flutter`、`shared_preferences`、`isar_community`、`cryptography`、`flutter_secure_storage`
- 协作：`WalletManager`、`lib/my/myid/MyIdPage`
