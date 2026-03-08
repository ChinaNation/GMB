# WUMINAPP 技术总文档（当前实现态）

## 1. 项目定位

`wuminapp` 当前为单仓 Flutter 客户端项目（iOS/Android），不再内置独立后端目录。

边界说明：

- 区块链 Runtime/共识逻辑不在本仓库实现（由 `citizenchain` 提供）
- SFID 与链交互由外部服务系统承载
- `wuminapp` 负责端上钱包、登录签名、链上交易入口、绑定指令发起、状态展示

## 2. 当前技术栈

- App：Flutter + Dart
- 手机机密存储：`flutter_secure_storage`（Keychain/Keystore）
- 手机业务存储：Isar
- 外部接口：HTTP API（由 SFID/网关系统提供）

## 3. 当前目录结构

```text
wuminapp/
├── android/
├── ios/
├── assets/
├── lib/
│   ├── main.dart
│   ├── Isar/
│   ├── login/
│   ├── user/
│   ├── wallet/
│   └── trade/
├── test/
└── WUMINAPP_TECHNICAL.md
```

说明：

- 原 `mobile/` 内容已上移到项目根目录
- 原 `backend/` 已移除

## 4. App 当前实现

### 4.1 主导航

底部 5 Tab：

- `广场`
- `治理`
- `消息`
- `金融`
- `我的`

### 4.2 治理页

- 机构分类卡片已内置：
  - 国储会 1
  - 省储会 43
  - 省储行 43

### 4.3 金融页

- 入口页标题为“金融”
- 已接入“链上交易”页面
- 链下交易仍为开发中占位

### 4.4 钱包与签名

钱包能力收口在 `lib/wallet/`：

- `core/`：钱包生命周期、Isar、机密 key 规范、生物识别守卫
- `capabilities/`：登录签名、链上交易编排、余额/API、管理员目录、绑定、证明态
- `ui/`：钱包页面

签名算法：`sr25519`。

签名前守卫：`UserIdentificationService.confirmBeforeSign()`。

调用点：

- 登录扫码签名前
- 链上交易签名前

### 4.5 登录模块

登录模块在 `lib/login/`，负责：

- 扫码识别挑战码
- 协议校验
- `aud` 白名单校验
- 防重放（`request_id`）
- 展示回执二维码

关键口径：

- 协议：`WUMINAPP_LOGIN_V1`
- 当前系统白名单：`cpms`、`sfid`
- 签名串：

```text
WUMINAPP_LOGIN_V1|system|aud|request_id|challenge|nonce|expires_at
```

## 5. 手机端三层存储（当前）

### 5.1 机密层（Secure Storage）

仅存高敏感数据：

- `wallet.secret.<wallet_id>.mnemonic.v1`
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`（预留）

### 5.2 业务层（Isar）

钱包域核心集合：

- `WalletProfileEntity`
- `WalletSettingsEntity`
- `TxRecordEntity`
- `AdminRoleCacheEntity`
- `ObservedAccountEntity`
- `LoginReplayEntity`
- `AppKvEntity`

当前 schema 版本：`wallet.data.schema.version = 4`。

### 5.3 偏好层（SharedPreferences）

仍有少量非机密配置使用（按模块逐步收口）：

- 登录白名单配置：`login.whitelist_config.v1`
- 登录防重放记录：`login.used_request_ids`
- SFID 绑定状态：`sfid.bind.*`
- 用户资料：
  - `user.profile.nickname`
  - `user.profile.avatar_path`

## 6. 外部 API 对接（当前）

App 通过 `ApiClient` 访问外部服务，当前已使用接口：

- `GET /api/v1/health`
- `GET /api/v1/wallet/balance`
- `POST /api/v1/tx/prepare`
- `POST /api/v1/tx/submit`
- `GET /api/v1/tx/status/:tx_hash`
- `POST /api/v1/chain/bind/request`
- `GET /api/v1/admins/catalog`

## 7. 安全基线（当前）

- 私钥/助记词不落 Isar 与远端服务
- 登录与交易签名前有设备侧身份确认能力（可开关）
- 登录白名单配置有本地 HMAC 完整性保护
- 绑定请求与交易状态依赖外部服务返回

## 8. 已知限制

- 登录防重放当前仍在 `SharedPreferences`，尚未切到 Isar 的 `LoginReplayEntity`
- `SfidBindingService` 状态仍在 `SharedPreferences`（`sfid.bind.*`）
- 链下交易模块仍为占位

## 9. 本地开发

```bash
cd /Users/rhett/GMB/wuminapp
flutter pub get
flutter run \
  --dart-define=WUMINAPP_API_BASE_URL=http://<外部服务地址> \
  --dart-define=WUMINAPP_API_TOKEN=<token>
```

真机调试时 `WUMINAPP_API_BASE_URL` 需为手机可达地址，不可用 `127.0.0.1`。

## 10. 关联模块文档

- 登录模块：`lib/login/LOGIN_TECHNICAL.md`
- 用户模块：`lib/user/USER_TECHNICAL.md`
- 钱包模块：`lib/wallet/WALLET_TECHNICAL.md`
