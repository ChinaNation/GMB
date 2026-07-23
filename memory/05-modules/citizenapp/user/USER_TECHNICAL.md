# USER 模块技术文档

## 1. 模块目标

`lib/my/user/` 负责 CitizenApp 的"我的 / 用户"模块，当前覆盖以下能力：

- 用户背景图上传与更换
- 用户头像上传与更换
- 聊天账户选择（钱包名称即用户昵称，双向同步）
- 用户二维码生成与展示
- 通讯录扫码导入、私人联系人名称、端到端加密云同步和跨设备恢复
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
  - 通讯录模型、按钱包 Isar 缓存、端侧加解密、Cloudflare 同步和待同步操作
- `lib/my/user/contact_book_page.dart`
  - 通讯录页面、搜索、同步状态和单条联系人卡片

相关协作模块：

- `lib/wallet/pages/wallet_page.dart`
  - 在选择聊天账户时提供钱包选择
  - 钱包改名时同步更新用户资料中的通信钱包名称
- `lib/my/myid/`
  - 电子护照页面和链上唯一身份状态服务
  - `identity_badge_snapshot_store.dart` 只保存按钱包账户隔离的公开身份徽章展示信号
  - “我的”页面只提供入口和头像认证角标，不承载电子护照设置流程

## 3. 数据模型

### 3.1 用户资料 `UserProfileState`

字段：

- `avatarPath` — 本地头像路径
- `backgroundPath` — 本地背景图路径
- `communicationWalletIndex` — 聊天账户钱包 index
- `communicationAddress` — 聊天账户钱包地址（SS58）
- `communicationWalletName` — 聊天账户钱包名称

展示规则：

- 用户昵称 = 当前默认钱包的 `walletName`；Cloudflare `display_name` 只是供他人读取的公开镜像。
- 钱包名称和公开镜像都缺失时，由 `ProfilePresentation` 按钱包账户稳定选择本地默认昵称；禁止回退为完整或截断账户。

设计说明：

- 不再有独立的昵称业务字段，用户设置的昵称完全由钱包名称决定；本地默认昵称只是无设置值时的展示兜底
- 用户修改昵称 = 修改通信钱包名称（`WalletManager.renameWallet`）
- 用户在钱包页改通信钱包名称 = 自动同步到用户资料
- 用户设置的公开头像和背景上传 Cloudflare R2；`avatarPath/backgroundPath` 只承接旧本机图片迁移和“我的”页即时显示，迁移成功后清空
- 用户未设置或真实图片读取失败时，从 `assets/profile_defaults/` 11 张本地照片中按账户稳定选择头像和背景，两个位置避免使用同一张图

### 3.2 用户二维码 `UserContactBody`

协议号：

- `QR_V1`

字段：

- `proto` — 协议标识（固定 `QR_V1`）
- `kind` — 固定 `user_contact`
- `body.address` — 聊天账户 SS58 地址
- `body.name` — 用户昵称（= 通信钱包名称）

### 3.3 通讯录 `UserContact`

字段：

- `address` — 对方 SS58 地址（当前链 `ss58 = 2027`）
- `contactName` — 当前用户为该联系人保存的私人联系人名称，对应协议字段 `contact_name`
- `createdAt` / `updatedAt` — 毫秒时间戳

公开昵称、头像、背景、个性签名、链上身份和会员徽章不复制进联系人记录，统一按
`address` 读取 `CitizenProfile`。因此通讯录、广场、聊天和关注列表始终进入同一个
`UserProfilePage`，不存在第二套联系人公开资料。

公开资料缺失时，各入口统一使用 `ProfilePresentation` / `ProfileAvatar` 的本地稳定
默认资料；联系人私人名称只在通讯录显示，钱包账户只在明确的账户行显示，不能充当昵称。

## 4. 持久化方案

### 4.1 用户资料

存储：`SharedPreferences`，键 `user.profile.state.v2`

内容：JSON 对象，保存头像路径、背景图路径、通信钱包 index/地址/名称

### 4.2 通讯录

本机缓存复用 Isar `AppKvEntity`，按默认热钱包 `account_id` 隔离：

- `contacts:<account_id>`：解密后的本机通讯录缓存
- `contact_pending_ops:<account_id>`：用户真实产生、尚未同步成功的添加/改名/删除操作
- `contact_sync_state:<account_id>`：最近一次同步阶段、时间和错误状态；分页游标只在单次请求内使用，不持久化

Cloudflare D1 `square_contacts` 只保存端侧 AES-256-GCM 密文、HMAC `contact_id`、
nonce、MAC 和更新时间；Worker 不接收联系人账户或联系人名称明文。通讯录密钥由
热钱包 seed 经 HKDF-SHA256 域隔离派生并保存在设备安全存储，同一助记词换设备导入
后可派生相同密钥解密恢复。

当前尚未正式创世，旧通讯录业务缓存直接删除重建；运行期不读取旧键、不执行迁移，
也不保留双轨缓存。

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

页面元素：背景图、头像、昵称（通信钱包名称）、二维码图标、右箭头、钱包/通讯录/电子护照/设置入口

### 5.2 用户资料页 `ProfileEditPage`

自上而下：
1. 用户二维码（聊天账户未设置时显示占位提示）
2. 用户头像行（左侧头像 + 右箭头，点击换头像）
3. 用户昵称行（左侧显示通信钱包名称 + 右箭头，点击弹窗修改，同步改钱包名）
4. 聊天账户行（选择钱包后即时保存）

### 5.3 昵称双向同步

- 用户资料页改昵称 → `WalletManager.renameWallet()` + `UserProfileService.updateCommunicationWalletName()`
- 钱包详情页改钱包名 → 检查该钱包是否为聊天账户 → 是则 `UserProfileService.updateCommunicationWalletName()`

### 5.4 聊天账户流程

1. 用户资料页点击聊天账户行 → 跳转钱包选择
2. 选中钱包后保存 `walletIndex + address + walletName`
3. 二维码实时更新

### 5.5 电子护照入口

1. “我的”页面点击电子护照入口
2. 跳转 `lib/my/myid/MyIdPage`
3. `MyIdService` 只取当前默认热钱包，并读取 finalized 永久 CID 身份闭环与 `VotingIdentityByCid` / `CandidateIdentityByCid`
4. 访客身份按“匿名访客 → 投票身份 → 竞选身份”排序；投票身份和竞选身份分别把对应当前卡移到首位
5. 当前公民卡展示真实字段值，非当前公民卡只展示字段名称；竞选身份的非当前投票卡不得重复展示投票身份值
6. 页面不得提供选择钱包、更换钱包、钱包二维码或扫码签名入口

电子护照详情页属于主动链流程，会启动并等待轻节点同步；这与“我的”首页头像徽章只读快照的边界不同。

### 5.6 通讯录

- 通讯录所属用户唯一来源是 `WalletManager.getDefaultWallet()`；页面和服务均不接受交易付款钱包或调用方账户覆盖。`UserContactService.getContacts()/sync()`只读写默认热钱包对应的 Isar 缓存与 Cloudflare 密文。
- 支持扫码添加（`QrScanMode.contact`）
- 支持修改私人联系人名称、删除、搜索和下拉同步
- 页面先显示按钱包隔离的 Isar 缓存，再后台刷新 Cloudflare 密文和公开用户资料
- 单条联系人显示圆角方形头像、身份徽章、私人联系人名称、公开昵称、短地址和个性签名
- 单条联系人三点菜单固定为“转账、私信、修改名称、删除联系人”；删除项使用危险红色。改名表单自行管理输入生命周期，取消或保存后不得留下已销毁输入控制器。
- “转账”只把联系人 SS58 账户预填为链上支付收款地址，不填写金额、不签名、不提交；“私信”复用 `openDirectChat()`进入统一一对一聊天。
- 普通模式点击联系人进入唯一 `UserProfilePage`；不保留联系人详情副本
- 交易页通讯录（`selectForTrade=true`）与“我的”入口使用同一页面、同一默认用户和同一联系人数据；该模式只改变点击后的返回动作：返回 SS58 地址填入收款栏，不做 AccountId hex 转换。
- 交易页右侧选择的钱包只决定付款和签名账户，不得改变左侧通讯录所属用户；切换付款钱包不切换通讯录，切换默认用户才切换通讯录。

## 6. 依赖

- `image_picker`、`qr_flutter`、`shared_preferences`、`isar_community`、`cryptography`、`flutter_secure_storage`
- 协作：`WalletManager`、`lib/my/myid/MyIdPage`
