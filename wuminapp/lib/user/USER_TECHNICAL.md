# USER 模块技术文档

## 1. 模块目标

`lib/user/` 负责 WuminApp 的“我的 / 用户”模块，当前覆盖以下能力：

- 用户背景图上传与更换
- 用户头像上传与更换
- 用户昵称展示与修改
- 用户账号绑定状态展示
- 用户二维码生成与放大展示
- 通讯录扫码导入与本地昵称修改

本次实现同时移除了旧的“观察账户”能力，用户模块不再承载该入口和相关交互。

## 2. 文件结构

- `lib/user/user.dart`
  - 用户主页 `ProfilePage`
  - 二维码页面 `UserQrPage`
  - 通讯录页面 `ContactBookPage`
  - 通讯录扫码页 `UserContactScannerPage`
- `lib/user/user_service.dart`
  - 用户资料模型与持久化
  - 用户二维码载荷模型
  - 通讯录模型与持久化

相关协作模块：

- `lib/wallet/ui/wallet_page.dart`
  - 在“绑定身份 / 重新绑定身份”时提供钱包公钥选择
- `lib/wallet/capabilities/sfid_binding_service.dart`
  - 保存绑定状态、地址、公钥，并负责向后端发起绑定请求

## 3. 数据模型

### 3.1 用户资料 `UserProfileState`

字段：

- `nickname`
- `nicknameCustomized`
- `avatarPath`
- `backgroundPath`

设计说明：

- 默认昵称固定展示为 `轻节点`
- `nicknameCustomized=false` 表示仅展示默认昵称，不能启用二维码
- 头像和背景图只保存本机文件路径，不做跨设备同步

### 3.2 用户二维码 `UserQrPayload`

协议号：

- `WUMINAPP_CONTACT_V1`（新版）
- `WUMINAPP_USER_CARD_V1`（旧版，解析兼容）

字段（新版）：

- `proto` — 协议标识
- `address` — SS58 地址
- `name` — 用户昵称

设计说明：

- 生成二维码统一使用新版 `WUMINAPP_CONTACT_V1` 格式
- 解析二维码同时兼容旧版 `WUMINAPP_USER_CARD_V1`（`account_pubkey` → `address`，`nickname` → `name`）
- 新版使用 SS58 地址替代裸公钥，更安全且用户可读
- 二维码内容为明文 JSON，当前阶段不做签名、防篡改和时效控制
- 二维码模型定义已迁移到 `lib/qr/contact/contact_qr_models.dart`

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

### 3.4 绑定状态 `SfidBindState`

字段：

- `status`
- `walletAddress`
- `walletPubkeyHex`
- `updatedAtMillis`

状态说明：

- `unbound` 未绑定
- `pending` 已把公钥提交给 SFID，等待系统回执
- `bound` SFID 确认绑定成功

## 4. 持久化方案

### 4.1 用户资料

存储位置：`SharedPreferences`

键：

- `user.profile.state.v2`

内容：

- JSON 对象，保存昵称、是否已设置昵称、头像路径、背景图路径

### 4.2 通讯录

存储位置：`SharedPreferences`

键：

- `user.contacts.items.v1`

内容：

- JSON 数组，保存通讯录列表

### 4.3 身份绑定

存储位置：`SharedPreferences`

键：

- `sfid.bind.status`
- `sfid.bind.address`
- `sfid.bind.pubkey_hex`
- `sfid.bind.updated_at`

## 5. 页面与交互流程

### 5.1 用户主页

页面元素：

- 顶部背景图
- 头像
- 昵称
- 账号绑定区
- 用户二维码卡片
- 通讯录入口
- 我的钱包入口
- 设置入口

交互：

1. 点击背景图，调用 `image_picker` 从设备相册选择并替换背景图
2. 点击头像，调用 `image_picker` 从设备相册选择并替换头像
3. 点击昵称编辑按钮，弹窗修改昵称
4. 点击“绑定身份”，跳转钱包页选择一个公钥
5. 绑定成功前，二维码保持禁用
6. 点击已启用的二维码卡片，进入放大展示页

### 5.2 身份绑定流程

当前实现流程：

1. 用户在“我的”页点击 `绑定身份`
2. 跳转 `MyWalletPage(selectForBind: true)`
3. 用户选择一个钱包后返回 `WalletProfile`
4. 前端调用 `SfidBindingService.submitBinding(address, pubkeyHex)`
5. 服务层调用 `ApiClient.requestChainBindByPubkey(pubkeyHex)`
6. 本地状态切换为 `pending`
7. 等待后续 SFID 系统回执后调用 `markBound(...)` 进入 `bound`

当前边界：

- 已完成“选钱包 -> 发送公钥 -> 本地 pending 状态切换”
- `pending -> bound` 的回执触发点已经预留在 `SfidBindingService.markBound(...)`
- 区块链和 SFID 的最终交互协议、回调入口、重试策略、失败态展示，后续再与业务一起细化

### 5.3 二维码启用规则

二维码只有同时满足以下条件才可操作：

- 用户昵称已明确设置，即 `nicknameCustomized=true`
- 身份绑定状态为 `bound`
- 已保存绑定公钥 `walletPubkeyHex`

未满足条件时：

- 用户主页显示禁用态二维码卡片
- 点击无效
- 提示“完成昵称设置并绑定身份成功后自动启用”

满足条件时：

- 根据当前昵称和当前绑定公钥实时生成二维码
- 点击进入 `UserQrPage` 放大展示
- 修改昵称或重新绑定身份后，二维码随最新状态即时更新

### 5.4 通讯录流程

导入：

1. 用户进入“我的通讯录”
2. 点击 `扫码添加`
3. 打开 `UserContactScannerPage`
4. 扫描到 `WUMINAPP_CONTACT_V1`（或旧版 `WUMINAPP_USER_CARD_V1`）二维码
5. 解析出昵称与公钥
6. 写入本地通讯录

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
- `mobile_scanner`
- `qr_flutter`
- `shared_preferences`

协作依赖：

- `WalletManager`
- `SfidBindingService`
- `ApiClient`

## 7. 已知限制与后续扩展

- 背景图和头像仅保存本地路径，应用重装或跨设备不会迁移
- 二维码当前为本地明文 JSON，不具备验签能力
- 通讯录仅本地保存，不和后端同步
- 绑定成功状态目前依赖未来的 SFID 回执接入，当前代码仅完成 pending 前半段与 bound 状态承接点

## 8. 后续推荐扩展

- 为用户二维码增加签名与时间戳，避免被篡改
- 为通讯录补充头像同步、备注、删除与搜索
- 增加 SFID 绑定结果轮询或推送回调
- 把用户资料与通讯录迁移到统一本地数据库，便于后续增量同步
