# USER 模块技术文档

## 1. 模块目标

`lib/my/user/` 负责 WuminApp 的"我的 / 用户"模块，当前覆盖以下能力：

- 用户背景图上传与更换
- 用户头像上传与更换
- 通信账户选择（钱包名称即用户昵称，双向同步）
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
  - 在选择通信账户时提供钱包选择
  - 钱包改名时同步更新用户资料中的通信钱包名称
- `lib/my/myid/`
  - 电子护照页面、状态服务、后端接口封装和现场签名页
  - “我的”页面只提供入口，不承载电子护照设置流程

## 3. 数据模型

### 3.1 用户资料 `UserProfileState`

字段：

- `avatarPath` — 本地头像路径
- `backgroundPath` — 本地背景图路径
- `communicationWalletIndex` — 通信账户钱包 index
- `communicationAddress` — 通信账户钱包地址（SS58）
- `communicationWalletName` — 通信账户钱包名称

计算属性：

- `nickname` — 用户昵称 = `communicationWalletName`，未设置时返回默认值 `轻节点`

设计说明：

- 不再有独立的昵称字段，用户昵称完全由通信账户钱包名称决定
- 用户修改昵称 = 修改通信钱包名称（`WalletManager.renameWallet`）
- 用户在钱包页改通信钱包名称 = 自动同步到用户资料
- 头像和背景图只保存本机文件路径

### 3.2 用户二维码 `UserContactBody`

协议号：

- `WUMIN_QR_V1`

字段：

- `proto` — 协议标识（固定 `WUMIN_QR_V1`）
- `kind` — 固定 `user_contact`
- `body.address` — 通信账户 SS58 地址
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
电子护照页展示字段由 SFID 状态接口同步：绑定状态、身份ID号码、投票账户地址、身份ID状态。
绑定状态和身份ID状态必须分离；`identity_status == NORMAL` 显示“状态：正常”，其他值显示“状态：异常”。

## 5. 页面与交互流程

### 5.1 用户主页

页面元素：背景图、头像、昵称（通信钱包名称）、二维码图标、右箭头、钱包/通讯录/电子护照/设置入口

### 5.2 用户资料页 `ProfileEditPage`

自上而下：
1. 用户二维码（通信账户未设置时显示占位提示）
2. 用户头像行（左侧头像 + 右箭头，点击换头像）
3. 用户昵称行（左侧显示通信钱包名称 + 右箭头，点击弹窗修改，同步改钱包名）
4. 通信账户行（选择钱包后即时保存）

### 5.3 昵称双向同步

- 用户资料页改昵称 → `WalletManager.renameWallet()` + `UserProfileService.updateCommunicationWalletName()`
- 钱包详情页改钱包名 → 检查该钱包是否为通信账户 → 是则 `UserProfileService.updateCommunicationWalletName()`

### 5.4 通信账户流程

1. 用户资料页点击通信账户行 → 跳转钱包选择
2. 选中钱包后保存 `walletIndex + address + walletName`
3. 二维码实时更新

### 5.5 电子护照入口

1. “我的”页面点击电子护照入口
2. 跳转 `lib/my/myid/MyIdPage`
3. 电子护照设置、状态同步和现场签名由 `lib/my/myid/` 负责
4. 页面展示“身份ID / 投票账户 / 状态”，其中“状态”是身份ID状态，不是绑定状态徽标
5. 现场签名页 `lib/my/myid/myid_sign_page.dart` 扫描 SFID 管理端签名请求时，扫码区域必须是
   固定正方形相机框并带四角提示，不得继续使用整块矩形相机画面。

### 5.6 通讯录

- 支持扫码添加（`QrScanMode.contact`）
- 支持修改本机昵称
- 交易页通讯录（`selectForTrade=true`）：点击联系人返回 SS58 地址填入收款栏，不做 AccountId hex 转换

## 6. 依赖

- `image_picker`、`qr_flutter`、`shared_preferences`、`local_auth`
- 协作：`WalletManager`、`lib/my/myid/MyIdPage`
