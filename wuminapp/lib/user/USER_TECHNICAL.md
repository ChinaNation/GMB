# USER 模块技术文档

## 1. 模块目标

`lib/user/` 负责 WuminApp 的"我的 / 用户"模块，当前覆盖以下能力：

- 用户背景图上传与更换
- 用户头像上传与更换
- 通信账户选择（钱包名称即用户昵称，双向同步）
- 投票账户选择与 SFID 绑定
- 用户二维码生成与展示
- 通讯录扫码导入与本地昵称修改

## 2. 文件结构

- `lib/user/user.dart`
  - 用户主页 `ProfilePage`
  - 用户资料编辑页 `ProfileEditPage`
  - 二维码页面 `UserQrPage`
  - 通讯录页面 `ContactBookPage`
- `lib/user/user_service.dart`
  - 用户资料模型与持久化
  - 用户二维码载荷模型
  - 通讯录模型与持久化

相关协作模块：

- `lib/wallet/ui/wallet_page.dart`
  - 在选择通信账户/投票账户时提供钱包选择
  - 钱包改名时同步更新用户资料中的通信钱包名称
- `lib/wallet/capabilities/sfid_binding_service.dart`
  - 保存投票账户绑定状态、地址、公钥，并负责向后端发起绑定请求

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

### 3.2 用户二维码 `UserQrPayload`

协议号：

- `WUMINAPP_CONTACT_V1`（新版）
- `WUMINAPP_USER_CARD_V1`（旧版，解析兼容）

字段（新版）：

- `proto` — 协议标识
- `address` — 通信账户 SS58 地址
- `name` — 用户昵称（= 通信钱包名称）

### 3.3 通讯录 `UserContact`

字段：

- `accountPubkeyHex` — 对方地址（SS58）
- `sourceNickname` — 对方二维码里的原始昵称
- `localNickname` — 本机自定义显示昵称
- `addedAtMillis` / `updatedAtMillis` — 时间戳

### 3.4 投票账户绑定状态 `SfidBindState`

状态：`unbound` → `pending` → `bound`

## 4. 持久化方案

### 4.1 用户资料

存储：`SharedPreferences`，键 `user.profile.state.v2`

内容：JSON 对象，保存头像路径、背景图路径、通信钱包 index/地址/名称

### 4.2 通讯录

存储：`SharedPreferences`，键 `user.contacts.items.v1`

### 4.3 投票账户绑定

存储：`SharedPreferences`，键 `sfid.bind.*`

## 5. 页面与交互流程

### 5.1 用户主页

页面元素：背景图、头像、昵称（通信钱包名称）、二维码图标、右箭头、通讯录/钱包/设置入口

### 5.2 用户资料页 `ProfileEditPage`

自上而下：
1. 用户二维码（通信账户未设置时显示占位提示）
2. 用户头像行（左侧头像 + 右箭头，点击换头像）
3. 用户昵称行（左侧显示通信钱包名称 + 右箭头，点击弹窗修改，同步改钱包名）
4. 通信账户行（选择钱包后即时保存）
5. 投票账户行（选择钱包后提交 SFID 绑定）

### 5.3 昵称双向同步

- 用户资料页改昵称 → `WalletManager.renameWallet()` + `UserProfileService.updateCommunicationWalletName()`
- 钱包详情页改钱包名 → 检查该钱包是否为通信账户 → 是则 `UserProfileService.updateCommunicationWalletName()`

### 5.4 通信账户流程

1. 用户资料页点击通信账户行 → 跳转钱包选择
2. 选中钱包后保存 `walletIndex + address + walletName`
3. 二维码实时更新

### 5.5 投票账户流程

1. 选择钱包 → 提交 SFID 绑定
2. 状态：未设置 → 绑定中 → 已绑定

### 5.6 通讯录

- 支持扫码添加（`QrScanMode.contact`）
- 支持修改本机昵称
- 交易页通讯录（`selectForTrade=true`）：点击联系人返回地址填入收款栏

## 6. 依赖

- `image_picker`、`qr_flutter`、`shared_preferences`、`local_auth`
- 协作：`WalletManager`、`SfidBindingService`
