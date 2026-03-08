# USER 模块技术文档（当前实现态）

## 1. 模块定位

`lib/user/` 负责“我的”模块相关能力：

- 头像/昵称展示
- 头像/昵称编辑
- 用户资料本地持久化
- 用户二维码展示
- 观察账户管理（列表、添加、删除、重命名、余额刷新）

当前实现按两份文件收口：

- `lib/user/user.dart`
- `lib/user/observe_accounts.dart`

## 2. 代码结构

### 2.1 `user.dart`

- `UserProfileState`
  - 字段：`nickname`、`avatarPath`
  - 语义：本地用户资料快照

- `UserProfileService`
  - `getState()`：从 `SharedPreferences` 读取用户资料
  - `saveState(UserProfileState)`：写入用户资料并返回最新状态

- `ProfilePage`
  - “我的”主页卡片（头像、昵称、绑定状态、编辑入口、二维码入口、观察账户入口）
- `ProfileEditPage`
  - 用户资料编辑页（头像选择、昵称输入、保存校验）
- `UserQrPage`
  - 用户二维码页（二维码展示 + 内容复制）

### 2.2 `observe_accounts.dart`

- `ObservedAccount`
  - 观察账户模型（`id/orgName/publicKey/address/balance/source`）
- `ObservedAccountService`
  - 观察账户增删改查
  - 余额刷新与失败回退
  - 账户输入归一化（公钥/SS58）
- `ObserveAccountsPage`
  - 观察账户列表页（刷新、添加、删除、进入详情）
- `ObserveAccountDetailPage`
  - 观察账户详情页（改名并保存）

## 3. 关键流程

### 3.1 用户资料首次加载

1. `ProfilePage.initState()` 调用 `_loadState()`
2. 读取 SFID 绑定状态（`SfidBindingService`）
3. 读取用户资料（`UserProfileService`）
4. 合并更新页面状态

### 3.2 用户资料编辑

1. 点击编辑箭头进入 `ProfileEditPage`
2. 可从相册选择头像（`image_picker`）
3. 可输入昵称（空昵称会阻止保存）
4. 返回 `UserProfileState` 给 `ProfilePage`
5. `ProfilePage` 调用 `UserProfileService.saveState()` 持久化

### 3.3 用户二维码

1. 点击“我的”卡片右上角二维码图标
2. `ProfilePage._openUserQr()` 组装二维码 payload
3. 跳转 `UserQrPage` 用 `QrImageView` 渲染二维码
4. 支持一键复制二维码原始内容

### 3.4 观察账户

1. 在“我的”页点击“观察账户”进入 `ObserveAccountsPage`
2. 添加时支持公钥或 SS58 地址输入，服务层完成归一化
3. 列表支持下拉刷新余额
4. 列表项支持左滑删除
5. 点击列表项进入详情页，可修改观察账户名称

## 4. 存储设计

### 4.1 用户资料（SharedPreferences）

- `user.profile.nickname`
- `user.profile.avatar_path`

默认值：

- 昵称默认 `公民用户`
- 头像路径默认 `null`

### 4.2 观察账户（Isar）

- `ObservedAccountEntity`：持久化观察账户清单
- `AdminRoleCacheEntity`：管理员目录缓存（用于推断组织名称）
- `AppKvEntity(wallet.admin_catalog.updated_at)`：目录缓存时间戳

## 5. 二维码载荷格式（当前）

协议标识：`WUMINAPP_USER_V1`

字段：

- `type`
- `nickname`
- `avatar_path`
- `sfid_bind_status`
- `wallet_address`

说明：当前为本地展示与分享口径，尚未引入后端签名或验签流程。

## 6. 后端依赖接口

- `GET /api/v1/wallet/balance`：观察账户余额查询
- `GET /api/v1/admins/catalog`：管理员目录查询（推断观察账户默认名称）

## 7. 依赖清单

- UI/状态：`flutter/material.dart`
- 头像选择：`image_picker`
- 二维码渲染：`qr_flutter`
- 偏好存储：`shared_preferences`
- 本地数据库：`isar`
- 地址编解码：`polkadart_keyring`
- API 调用：`ApiClient`
- 身份绑定状态：`SfidBindingService`

## 8. 已知限制

- 头像路径是本地文件路径，跨设备不可迁移。
- 用户二维码当前是明文 JSON 负载，不具备防篡改能力。
- 观察账户清单目前仅保存在本地 Isar，不与后端同步。
