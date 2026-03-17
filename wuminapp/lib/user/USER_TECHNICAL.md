# USER 模块技术文档

## 1. 模块目标

`lib/user/` 负责 WuminApp 的"我的 / 用户"模块，当前覆盖以下能力：

- 用户背景图上传与更换
- 用户头像上传与更换
- 用户昵称展示与修改
- 通信账户选择（用于生成用户二维码）
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
- `lib/wallet/capabilities/sfid_binding_service.dart`
  - 保存投票账户绑定状态、地址、公钥，并负责向后端发起绑定请求

## 3. 数据模型

### 3.1 用户资料 `UserProfileState`

字段：

- `nickname` — 用户昵称
- `nicknameCustomized` — 是否已自定义昵称
- `avatarPath` — 本地头像路径
- `backgroundPath` — 本地背景图路径
- `communicationAddress` — 通信账户地址（钱包 SS58 地址）

设计说明：

- 默认昵称固定展示为 `轻节点`
- `nicknameCustomized=false` 表示仅展示默认昵称，不能启用二维码
- 头像和背景图只保存本机文件路径，不做跨设备同步
- `communicationAddress` 与 `nickname` 组成用户二维码内容

### 3.2 用户二维码 `UserQrPayload`

协议号：

- `WUMINAPP_CONTACT_V1`（新版）
- `WUMINAPP_USER_CARD_V1`（旧版，解析兼容）

字段（新版）：

- `proto` — 协议标识
- `address` — 通信账户 SS58 地址
- `name` — 用户昵称

设计说明：

- 生成二维码统一使用新版 `WUMINAPP_CONTACT_V1` 格式
- 解析二维码同时兼容旧版 `WUMINAPP_USER_CARD_V1`
- 二维码内容为明文 JSON
- 更改昵称或通信账户后二维码实时更新

### 3.3 通讯录 `UserContact`

字段：

- `accountPubkeyHex`
- `sourceNickname`
- `localNickname`
- `addedAtMillis`
- `updatedAtMillis`

设计说明：

- `sourceNickname` 是对方二维码里的原始昵称
- `localNickname` 是本机自定义显示昵称，只影响当前设备展示
- 同一公钥只保留一条通讯录记录
- 重复扫码会更新对方原始昵称，但不覆盖本机自定义昵称

### 3.4 投票账户绑定状态 `SfidBindState`

字段：

- `status` — 绑定状态枚举
- `walletAddress` — 投票账户钱包地址
- `walletPubkeyHex` — 投票账户公钥
- `updatedAtMillis` — 最后更新时间

状态说明：

- `unbound` 未设置
- `pending` 已提交到 SFID 系统，等待绑定结果
- `bound` SFID 确认绑定成功

## 4. 持久化方案

### 4.1 用户资料

存储位置：`SharedPreferences`

键：`user.profile.state.v2`

内容：JSON 对象，保存昵称、是否已设置昵称、头像路径、背景图路径、通信账户地址

### 4.2 通讯录

存储位置：`SharedPreferences`

键：`user.contacts.items.v1`

内容：JSON 数组，保存通讯录列表

### 4.3 投票账户绑定

存储位置：`SharedPreferences`

键：

- `sfid.bind.status`
- `sfid.bind.address`
- `sfid.bind.pubkey_hex`
- `sfid.bind.updated_at`

## 5. 页面与交互流程

### 5.1 用户主页（"我的"页面）

页面元素：

- 顶部背景图（点击可更换）
- 头像 + 昵称（纯展示，不可直接编辑）
- 二维码图标（通信账户设置后可用，点击进入放大展示页）
- 右箭头（进入用户资料编辑页）
- 通讯录入口
- 我的钱包入口
- 设置入口

### 5.2 用户资料页面 `ProfileEditPage`

页面元素（自上而下）：

1. **标题**：用户资料
2. **用户二维码**：昵称+通信账户地址生成的二维码（未设置前显示占位提示）
3. **用户头像行**：左侧标签 + 右侧小头像(44px) + 箭头，点击箭头直接选择相册图片更换头像
4. **用户昵称行**：左侧标签 + 右侧当前昵称 + 箭头，点击弹窗修改昵称（即时保存）
5. **通信账户行**：左侧标签 + 右侧钱包地址（或"未设置"）+ 箭头，点击跳转钱包选择页，选择后即时保存，二维码随之更新
6. **投票账户行**：左侧标签 + 右侧钱包地址（或"未设置"）+ 绑定状态标签 + 箭头，点击跳转钱包选择页，选择后提交 SFID 绑定

所有修改即时保存，无需手动保存按钮。

### 5.3 通信账户流程

1. 用户在用户资料页点击"通信账户"行
2. 跳转 `MyWalletPage(selectForBind: true)` 选择钱包
3. 选中钱包后，地址保存到 `UserProfileState.communicationAddress`
4. 二维码实时更新为 `{nickname, communicationAddress}`
5. 其他用户扫描该二维码可将本用户添加到通讯录

### 5.4 投票账户流程

1. 用户在用户资料页点击"投票账户"行
2. 跳转 `MyWalletPage(selectForBind: true)` 选择钱包
3. 选中钱包后，调用 `SfidBindingService.submitBinding(address, pubkeyHex)`
4. 本地状态切换为 `pending`，界面显示"绑定中"
5. SFID 系统返回绑定成功后调用 `markBound(...)` 进入 `bound`，显示"已绑定"

当前边界：

- 已完成"选钱包 -> 发送公钥 -> 本地 pending 状态切换"
- `pending -> bound` 的回执触发点已预留在 `SfidBindingService.markBound(...)`

### 5.5 二维码启用规则

二维码只有同时满足以下条件才可用：

- 用户昵称已自定义设置（`nicknameCustomized=true`）
- 通信账户已设置（`communicationAddress` 非空）

### 5.6 通讯录流程

导入：

1. 用户进入"我的通讯录"
2. 点击扫码添加
3. 打开 `QrScanPage(mode: QrScanMode.contact)`
4. 扫描用户二维码后解析昵称与地址
5. 写入本地通讯录

编辑：

1. 点击通讯录列表项
2. 弹窗修改本机显示昵称
3. 空值表示恢复为对方原始昵称

约束：

- 不允许把自己加入通讯录
- 重复扫码同一公钥时更新来源昵称和更新时间

## 6. 依赖

- `flutter/material.dart`
- `image_picker`
- `qr_flutter`
- `shared_preferences`
- `local_auth`

协作依赖：

- `WalletManager`
- `SfidBindingService`
- `ApiClient`

## 7. 已知限制与后续扩展

- 背景图和头像仅保存本地路径，应用重装或跨设备不会迁移
- 二维码当前为本地明文 JSON，不具备验签能力
- 通讯录仅本地保存，不和后端同步
- 投票账户绑定成功状态依赖 SFID 回执接入，当前仅完成 pending 前半段
- 通信账户与投票账户可以是不同钱包地址
