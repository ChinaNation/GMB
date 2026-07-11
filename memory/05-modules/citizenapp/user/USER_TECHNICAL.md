# USER 模块技术文档

## 1. 模块目标

`lib/my/user/` 负责 CitizenApp 的"我的 / 用户"模块，当前覆盖以下能力：

- 用户背景图上传与更换
- 用户头像上传与更换
- 聊天账户选择（钱包名称即用户昵称，双向同步）
- 用户二维码生成与展示
- 通讯录扫码导入与本地昵称修改
- 电子护照入口展示

## 2. 文件结构

- `lib/my/user/user.dart`
  - 用户主页 `ProfilePage`
  - 用户资料编辑页 `ProfileEditPage`
  - 二维码页面 `UserQrPage`
  - 通讯录页面 `ContactBookPage`
- `lib/my/user/user_service.dart`
  - 用户资料模型与持久化
  - 用户二维码载荷模型
  - 通讯录模型与持久化

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

计算属性：

- `nickname` — 用户昵称 = `communicationWalletName`，未设置时返回默认值 `轻节点`

设计说明：

- 不再有独立的昵称字段，用户昵称完全由聊天账户钱包名称决定
- 用户修改昵称 = 修改通信钱包名称（`WalletManager.renameWallet`）
- 用户在钱包页改通信钱包名称 = 自动同步到用户资料
- 头像和背景图只保存本机文件路径

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
- `sourceNickname` — 对方二维码里的原始昵称
- `localNickname` — 本机自定义显示昵称
- `addedAtMillis` / `updatedAtMillis` — 时间戳

## 4. 持久化方案

### 4.1 用户资料

存储：`SharedPreferences`，键 `user.profile.state.v2`

内容：JSON 对象，保存头像路径、背景图路径、通信钱包 index/地址/名称

### 4.2 通讯录

存储：`SharedPreferences`，键 `user.contacts.items.v1`

### 4.3 电子护照

电子护照状态归属 `lib/my/myid/MyIdService`，用户模块不直接读写电子护照状态。
电子护照页展示字段直接读取链上 `CitizenIdentity::VotingIdentityByAccount`：投票账户、身份 CID 号、状态、有效期。
状态由链上 `citizen_status` 和护照有效期窗口派生，不再使用 OnChina 本地状态接口或 `myid.*` 本地档案缓存。
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
3. `MyIdService` 扫描本机钱包列表并读取链上 `VotingIdentityByAccount`
4. 页面只展示“投票账户 / 身份 CID 号 / 状态 / 有效期”
5. 页面不得提供选择钱包、更换钱包、钱包二维码或扫码签名入口

电子护照详情页属于主动链流程，会启动并等待轻节点同步；这与“我的”首页头像徽章只读快照的边界不同。

### 5.6 通讯录

- 支持扫码添加（`QrScanMode.contact`）
- 支持修改本机昵称
- 交易页通讯录（`selectForTrade=true`）：点击联系人返回 SS58 地址填入收款栏，不做 AccountId hex 转换

## 6. 依赖

- `image_picker`、`qr_flutter`、`shared_preferences`、`local_auth`
- 协作：`WalletManager`、`lib/my/myid/MyIdPage`
